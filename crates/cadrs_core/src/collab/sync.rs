//! Editing one document together: every peer edits its own copy at once, and the owner puts
//! the edits in one order.
//!
//! - A [`Change`] is what one edit did ([`Delta`]) and the orders it was made from, so it can
//!   be applied to a copy others changed meanwhile ([`Change::rebase_onto`]): what they added
//!   keeps its place, what they removed stays removed.
//! - Its [`Footprint`] is what it writes: the document's header, whole elements, a Part
//!   Studio's shell and its features one by one. Two edits **conflict** when their footprints
//!   overlap; orders never do (they are merged).
//! - The owner's [`Sequencer`] numbers the edits, first come first served: one made without
//!   seeing a conflicting edit by someone else is refused.
//! - Every other peer keeps a [`Replica`]: the copy as the owner has it (`confirmed`), at most
//!   one edit of its own on its way to the owner, and its own document, which is the confirmed
//!   copy plus whatever it has not had confirmed yet, rebased each time the confirmed copy
//!   moves.

use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::document::{Document, Element, ElementKind};
use crate::history_log::{Delta, document_hash};
use crate::ids::{ElementId, FeatureId};

/// How many numbered edits the sequencer remembers: an edit made before the oldest of them is
/// refused as stale (its peer catches up from a fresh copy).
pub const KEEP: usize = 512;

/// One part of a document an edit can write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    /// Everything but the elements: name, units, libraries, …
    Head,
    /// A whole element (a tab): its contents, or its being there at all.
    Element(ElementId),
    /// A Part Studio without its features: its name, parts, appearances, folders, …
    Shell(ElementId),
    /// One feature of a Part Studio.
    Feature(ElementId, FeatureId),
}

impl Key {
    fn element(self) -> Option<ElementId> {
        match self {
            Key::Head => None,
            Key::Element(e) | Key::Shell(e) | Key::Feature(e, _) => Some(e),
        }
    }

    /// Whether writing `self` and `other` touches the same thing. A whole element covers its
    /// shell and its features.
    pub fn overlaps(self, other: Key) -> bool {
        match (self, other) {
            (Key::Element(e), k) | (k, Key::Element(e)) => k.element() == Some(e),
            (a, b) => a == b,
        }
    }
}

/// What an edit writes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Footprint(pub Vec<Key>);

impl Footprint {
    /// The first thing both write, if any.
    pub fn overlap(&self, other: &Footprint) -> Option<Key> {
        self.0
            .iter()
            .copied()
            .find(|a| other.0.iter().any(|b| a.overlaps(*b)))
    }

    pub fn covers(&self, key: Key) -> bool {
        self.0.iter().any(|k| k.overlaps(key))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// One edit as it travels between peers.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Change {
    pub delta: Delta,
    /// The element order it was made from, when it changes the order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_order: Option<Vec<ElementId>>,
    /// The feature order of each Part Studio whose order it changes, as it was.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub base_features: Vec<(ElementId, Vec<FeatureId>)>,
}

fn ids(doc: &Document) -> Vec<ElementId> {
    doc.elements.iter().map(|e| e.id).collect()
}

fn feature_ids(e: &Element) -> Vec<FeatureId> {
    e.features().iter().map(|f| f.id).collect()
}

/// `mine` (made from `base`) merged with `theirs` (also made from `base`): mine, less what
/// they removed, plus what they added, each after the item before it in theirs.
pub fn merge_order<T: Copy + PartialEq>(base: &[T], mine: &[T], theirs: &[T]) -> Vec<T> {
    let mut out: Vec<T> = mine
        .iter()
        .copied()
        .filter(|id| theirs.contains(id) || !base.contains(id))
        .collect();
    let mut prev: Option<T> = None;
    for &id in theirs {
        if !base.contains(&id) && !out.contains(&id) {
            let at = prev
                .and_then(|p| out.iter().position(|x| *x == p))
                .map_or(0, |i| i + 1);
            out.insert(at, id);
        }
        if out.contains(&id) {
            prev = Some(id);
        }
    }
    out
}

/// `order` (made from `base`) with the items `drop` says leaving it, and the items it removed
/// that `back` says returning to their place in `base`.
fn take_back<T: Copy + PartialEq>(
    base: &[T],
    order: &[T],
    drop: impl Fn(T) -> bool,
    back: impl Fn(T) -> bool,
) -> Vec<T> {
    let mut out: Vec<T> = order.iter().copied().filter(|id| !drop(*id)).collect();
    let mut prev: Option<T> = None;
    for &id in base {
        if !order.contains(&id) && back(id) {
            let at = prev
                .and_then(|p| out.iter().position(|x| *x == p))
                .map_or(0, |i| i + 1);
            out.insert(at, id);
        }
        if out.contains(&id) {
            prev = Some(id);
        }
    }
    out
}

impl Change {
    /// The edit that turns `before` into `after`.
    pub fn between(before: &Document, after: &Document) -> Self {
        let delta = Delta::between(before, after);
        let base_order = delta.order.as_ref().map(|_| ids(before));
        let base_features = delta
            .studios
            .iter()
            .filter(|p| p.order.is_some())
            .filter_map(|p| Some((p.id, feature_ids(before.element(p.id)?))))
            .collect();
        Self {
            delta,
            base_order,
            base_features,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.delta.is_empty()
    }

    fn base_features_of(&self, studio: ElementId) -> Option<&[FeatureId]> {
        self.base_features
            .iter()
            .find(|(id, _)| *id == studio)
            .map(|(_, f)| f.as_slice())
    }

    /// What it writes.
    pub fn footprint(&self) -> Footprint {
        let d = &self.delta;
        let mut keys = Vec::new();
        if d.head.is_some() {
            keys.push(Key::Head);
        }
        keys.extend(d.elements.iter().map(|e| Key::Element(e.id)));
        if let (Some(order), Some(base)) = (&d.order, &self.base_order) {
            keys.extend(
                base.iter()
                    .filter(|id| !order.contains(id))
                    .map(|id| Key::Element(*id)),
            );
        }
        for p in &d.studios {
            if p.shell.is_some() {
                keys.push(Key::Shell(p.id));
            }
            keys.extend(p.features.iter().map(|f| Key::Feature(p.id, f.id)));
            if let (Some(order), Some(base)) = (&p.order, self.base_features_of(p.id)) {
                keys.extend(
                    base.iter()
                        .filter(|f| !order.contains(f))
                        .map(|f| Key::Feature(p.id, *f)),
                );
            }
        }
        Footprint(keys)
    }

    /// Makes it apply to `current`, a copy others have changed since it was made: the element
    /// and feature orders are merged with theirs, and a Part Studio they removed is let go.
    pub fn rebase_onto(&mut self, current: &Document) {
        if let (Some(order), Some(base)) = (&mut self.delta.order, &self.base_order) {
            *order = merge_order(base, order, &ids(current));
        }
        let base_features = std::mem::take(&mut self.base_features);
        self.delta.studios.retain(|p| {
            current
                .element(p.id)
                .is_some_and(|e| matches!(e.kind, ElementKind::PartStudio { .. }))
        });
        for p in &mut self.delta.studios {
            let (Some(order), Some(studio)) = (&mut p.order, current.element(p.id)) else {
                continue;
            };
            if let Some((_, base)) = base_features.iter().find(|(id, _)| *id == p.id) {
                *order = merge_order(base, order, &feature_ids(studio));
            }
        }
        self.base_features = base_features;
    }

    /// Applies it to `doc`, rebased first (`doc` may have moved on since it was made).
    pub fn apply_to(&self, doc: &mut Document) {
        let mut c = self.clone();
        c.rebase_onto(doc);
        c.delta.apply(doc);
    }

    /// Applies what it does to one element to `element` (an undo step's copy of it): its
    /// features and shell, with the feature order merged; a whole new copy if it replaces it.
    pub fn apply_to_element(&self, element: &mut Element) {
        if let Some(e) = self.delta.elements.iter().find(|e| e.id == element.id) {
            *element = e.clone();
            return;
        }
        let Some(p) = self.delta.studios.iter().find(|p| p.id == element.id) else {
            return;
        };
        let mut doc = Document::empty("");
        doc.elements.push(element.clone());
        let c = Change {
            delta: Delta {
                studios: vec![p.clone()],
                ..Delta::default()
            },
            base_order: None,
            base_features: self
                .base_features
                .iter()
                .filter(|(id, _)| *id == element.id)
                .cloned()
                .collect(),
        };
        c.apply_to(&mut doc);
        if let Some(e) = doc.elements.pop() {
            *element = e;
        }
    }

    /// It, less everything that overlaps `fp`: an edit the owner refused, taken back out of a
    /// peer's own unconfirmed edits (this change, made from the confirmed copy). What it added
    /// there goes, and what it removed there comes back.
    pub fn without(&self, fp: &Footprint) -> Self {
        let mut d = self.delta.clone();
        if fp.covers(Key::Head) {
            d.head = None;
        }
        d.elements.retain(|e| !fp.covers(Key::Element(e.id)));
        if let (Some(order), Some(base)) = (&mut d.order, &self.base_order) {
            let covered = |id| fp.covers(Key::Element(id));
            *order = take_back(
                base,
                order,
                |id| !base.contains(&id) && covered(id),
                covered,
            );
        }
        for p in &mut d.studios {
            let id = p.id;
            if fp.covers(Key::Shell(id)) {
                p.shell = None;
            }
            p.features.retain(|f| !fp.covers(Key::Feature(id, f.id)));
            if let (Some(order), Some(base)) = (&mut p.order, self.base_features_of(id)) {
                let covered = |f| fp.covers(Key::Feature(id, f));
                *order = take_back(base, order, |f| !base.contains(&f) && covered(f), covered);
            }
        }
        Self {
            delta: d,
            base_order: self.base_order.clone(),
            base_features: self.base_features.clone(),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The owner's side

/// Why the sequencer refused an edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// Someone else changed what it changes first.
    Conflict { author: String, key: Key },
    /// It was made from a copy older than the sequencer remembers (or from one it never had).
    Stale,
}

/// The owner's numbering of the edits.
#[derive(Debug, Clone, Default)]
pub struct Sequencer {
    seq: u64,
    log: VecDeque<(u64, String, Footprint)>,
}

impl Sequencer {
    /// Numbering from `seq` (the edits up to it are the document as it is).
    pub fn new(seq: u64) -> Self {
        Self {
            seq,
            log: VecDeque::new(),
        }
    }

    /// The number of the last edit.
    pub fn seq(&self) -> u64 {
        self.seq
    }

    /// Takes `change` by `author`, made from the document as it was after edit `base`: refused
    /// if an edit by someone else since then wrote any of what it writes; otherwise rebased
    /// onto `doc` (the document now), applied to it and numbered. Returns its number and the
    /// change as applied, which is what every peer applies.
    pub fn admit(
        &mut self,
        author: &str,
        base: u64,
        mut change: Change,
        doc: &mut Document,
    ) -> Result<(u64, Change), Refusal> {
        let remembered = base == self.seq || self.log.front().is_some_and(|(s, ..)| *s <= base + 1);
        if base > self.seq || !remembered {
            return Err(Refusal::Stale);
        }
        let fp = change.footprint();
        for (s, a, f) in &self.log {
            if *s > base
                && a != author
                && let Some(key) = f.overlap(&fp)
            {
                return Err(Refusal::Conflict {
                    author: a.clone(),
                    key,
                });
            }
        }
        change.rebase_onto(doc);
        change.delta.apply(doc);
        self.seq += 1;
        self.log.push_back((self.seq, author.to_string(), fp));
        if self.log.len() > KEEP {
            self.log.pop_front();
        }
        Ok((self.seq, change))
    }
}

// ---------------------------------------------------------------------------------------------
// Everyone else's side

/// A numbered edit as a peer receives it.
#[derive(Debug, Clone, PartialEq)]
pub struct Incoming {
    /// The id this peer gave it, when it is this peer's own.
    pub own: Option<u64>,
    pub change: Change,
    /// [`document_hash`] of the owner's document after it.
    pub hash: u64,
}

/// What [`Replica::receive`] did.
#[derive(Debug, Default)]
pub struct Received {
    /// The peer's own document, when it changed.
    pub local: Option<Document>,
    /// The confirmed copy no longer matches the owner's: start again from a fresh copy.
    pub diverged: bool,
}

/// A peer's view of a document the owner sequences.
#[derive(Debug, Clone)]
pub struct Replica {
    seq: u64,
    confirmed: Document,
    /// The edit on its way to the owner: its id and what it writes.
    inflight: Option<(u64, Footprint)>,
    next: u64,
    /// Numbered edits that arrived ahead of one still missing.
    waiting: BTreeMap<u64, Incoming>,
}

impl Replica {
    /// Starting from `doc`, the document after edit `seq`.
    pub fn new(seq: u64, doc: Document) -> Self {
        Self {
            seq,
            confirmed: doc,
            inflight: None,
            next: 1,
            waiting: BTreeMap::new(),
        }
    }

    pub fn seq(&self) -> u64 {
        self.seq
    }

    /// The document as the owner has it.
    pub fn confirmed(&self) -> &Document {
        &self.confirmed
    }

    /// True while an edit of this peer's waits for the owner.
    pub fn in_flight(&self) -> bool {
        self.inflight.is_some()
    }

    /// True while numbered edits wait for one that has not arrived.
    pub fn has_gap(&self) -> bool {
        !self.waiting.is_empty()
    }

    /// The next edit to send the owner: what `local` has that the confirmed copy doesn't. One
    /// at a time: `None` while one is on its way (whatever happens meanwhile goes in the next).
    pub fn propose(&mut self, local: &Document) -> Option<(u64, u64, Change)> {
        if self.inflight.is_some() || *local == self.confirmed {
            return None;
        }
        let change = Change::between(&self.confirmed, local);
        if change.is_empty() {
            return None;
        }
        let id = self.next;
        self.next += 1;
        self.inflight = Some((id, change.footprint()));
        Some((id, self.seq, change))
    }

    /// Edit `seq`, numbered by the owner; `local` is this peer's own document. Edits are
    /// applied in their order, those ahead of a missing one kept until it comes.
    pub fn receive(&mut self, seq: u64, incoming: Incoming, local: &Document) -> Received {
        if seq > self.seq {
            self.waiting.insert(seq, incoming);
        }
        let before = self.confirmed.clone();
        let mut last_hash = None;
        while let Some(next) = self.waiting.remove(&(self.seq + 1)) {
            next.change.apply_to(&mut self.confirmed);
            if let (Some(own), Some((id, _))) = (next.own, &self.inflight)
                && own == *id
            {
                self.inflight = None;
            }
            self.seq += 1;
            last_hash = Some(next.hash);
        }
        let Some(hash) = last_hash else {
            return Received::default();
        };
        let diverged = document_hash(&self.confirmed) != hash;
        // This peer's own edits not confirmed yet, on top of the new confirmed copy.
        let pending = Change::between(&before, local);
        let mut mine = self.confirmed.clone();
        pending.apply_to(&mut mine);
        Received {
            local: (mine != *local).then_some(mine),
            diverged,
        }
    }

    /// The owner refused edit `id`: this peer's document without it. `None` if `id` is not the
    /// edit on its way.
    pub fn refused(&mut self, id: u64, local: &Document) -> Option<Document> {
        let (_, fp) = self.inflight.take_if(|(i, _)| *i == id)?;
        let pending = Change::between(&self.confirmed, local).without(&fp);
        let mut mine = self.confirmed.clone();
        pending.apply_to(&mut mine);
        Some(mine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Feature, FeatureKind, SketchFeature};

    /// A document whose Part Studio has four sketches: "Origin", "Top", "Front", "Right".
    fn shared() -> Document {
        let mut d = Document::new("Shared");
        let fs = d.elements[0].features_mut().unwrap();
        for name in ["Origin", "Top", "Front", "Right"] {
            fs.push(Feature::new(
                FeatureId::new(),
                name,
                FeatureKind::Sketch(SketchFeature::new(None)),
            ));
        }
        d
    }

    /// The owner's document and sequencer, and the edits it numbered (to deliver).
    struct Owner {
        doc: Document,
        seq: Sequencer,
        out: Vec<(u64, String, Option<u64>, Change, u64)>,
    }

    struct Peer {
        name: &'static str,
        doc: Document,
        replica: Replica,
        /// Delivered so far.
        seen: usize,
    }

    impl Owner {
        fn new(doc: Document) -> Self {
            Self {
                doc,
                seq: Sequencer::new(0),
                out: Vec::new(),
            }
        }

        fn peer(&self, name: &'static str) -> Peer {
            Peer {
                name,
                doc: self.doc.clone(),
                replica: Replica::new(self.seq.seq(), self.doc.clone()),
                seen: self.out.len(),
            }
        }

        /// The owner's own edit.
        fn edit(&mut self, f: impl FnOnce(&mut Document)) {
            let mut after = self.doc.clone();
            f(&mut after);
            let change = Change::between(&self.doc, &after);
            let base = self.seq.seq();
            let (n, c) = self
                .seq
                .admit("owner", base, change, &mut self.doc)
                .unwrap();
            assert_eq!(self.doc, after);
            self.out
                .push((n, "owner".into(), None, c, document_hash(&self.doc)));
        }

        /// A peer's next edit reaches the owner. `Err` is the refusal, sent back to it.
        fn take(&mut self, p: &mut Peer) -> Result<(), Refusal> {
            let Some((id, base, change)) = p.replica.propose(&p.doc) else {
                return Ok(());
            };
            match self.seq.admit(p.name, base, change, &mut self.doc) {
                Ok((n, c)) => {
                    let hash = document_hash(&self.doc);
                    self.out.push((n, p.name.into(), Some(id), c, hash));
                    Ok(())
                }
                Err(r) => {
                    if let Some(doc) = p.replica.refused(id, &p.doc) {
                        p.doc = doc;
                    }
                    Err(r)
                }
            }
        }

        /// Everything numbered so far reaches `p`.
        fn deliver(&self, p: &mut Peer) {
            for (n, author, id, c, hash) in &self.out[p.seen..] {
                let own = if author == p.name { *id } else { None };
                let r = p.replica.receive(
                    *n,
                    Incoming {
                        own,
                        change: c.clone(),
                        hash: *hash,
                    },
                    &p.doc,
                );
                assert!(!r.diverged, "{} diverged at {n}", p.name);
                if let Some(d) = r.local {
                    p.doc = d;
                }
            }
            p.seen = self.out.len();
        }
    }

    impl Peer {
        fn edit(&mut self, f: impl FnOnce(&mut Document)) {
            f(&mut self.doc);
        }
    }

    fn studio(d: &Document) -> ElementId {
        d.elements[0].id
    }

    fn rename(d: &mut Document, i: usize, name: &str) {
        let s = studio(d);
        d.element_mut(s).unwrap().features_mut().unwrap()[i].name = name.into();
    }

    fn add_feature(d: &mut Document, name: &str) -> FeatureId {
        let s = studio(d);
        let fs = d.element_mut(s).unwrap().features_mut().unwrap();
        let id = FeatureId::new();
        let mut f: Feature = fs[0].clone();
        f.id = id;
        f.name = name.into();
        fs.push(f);
        id
    }

    fn names(d: &Document) -> Vec<String> {
        d.elements[0]
            .features()
            .iter()
            .map(|f| f.name.clone())
            .collect()
    }

    fn settle(o: &mut Owner, peers: &mut [&mut Peer]) {
        for _ in 0..4 {
            for p in peers.iter_mut() {
                let _ = o.take(p);
            }
            for p in peers.iter_mut() {
                o.deliver(p);
            }
        }
        for p in peers.iter() {
            assert_eq!(p.doc, o.doc, "{} has not converged", p.name);
            assert!(!p.replica.in_flight());
        }
    }

    #[test]
    fn merge_order_keeps_both_sides_additions_and_removals() {
        let base = [1, 2, 3, 4];
        // Mine removed 2 and added 9 after 3; theirs removed 4 and added 7 after 1.
        let mine = [1, 3, 9, 4];
        let theirs = [1, 7, 2, 3];
        assert_eq!(merge_order(&base, &mine, &theirs), vec![1, 7, 3, 9]);
    }

    #[test]
    fn edits_of_different_features_in_one_studio_both_land() {
        let mut o = Owner::new(shared());
        let mut a = o.peer("a");
        let mut b = o.peer("b");
        a.edit(|d| rename(d, 1, "Top by a"));
        b.edit(|d| rename(d, 2, "Front by b"));
        o.edit(|d| rename(d, 3, "Right by owner"));
        settle(&mut o, &mut [&mut a, &mut b]);
        assert_eq!(
            names(&o.doc)[1..],
            ["Top by a", "Front by b", "Right by owner"]
        );
    }

    #[test]
    fn the_second_edit_of_one_feature_is_refused_and_taken_back() {
        let mut o = Owner::new(shared());
        let mut a = o.peer("a");
        let mut b = o.peer("b");
        a.edit(|d| rename(d, 1, "a's"));
        b.edit(|d| {
            rename(d, 1, "b's");
            rename(d, 2, "b's front")
        });
        assert_eq!(o.take(&mut a), Ok(()));
        let r = o.take(&mut b);
        assert!(
            matches!(r, Err(Refusal::Conflict { ref author, .. }) if author == "a"),
            "{r:?}"
        );
        // b's whole edit went (it was one), back to the confirmed copy.
        assert_eq!(names(&b.doc)[1..3], ["Top", "Front"]);
        settle(&mut o, &mut [&mut a, &mut b]);
        assert_eq!(names(&b.doc)[1..3], ["a's", "Front"]);
    }

    #[test]
    fn features_added_at_once_by_two_peers_are_both_kept() {
        let mut o = Owner::new(shared());
        let mut a = o.peer("a");
        let mut b = o.peer("b");
        let fa = add_feature(&mut a.doc, "Sketch by a");
        let fb = add_feature(&mut b.doc, "Sketch by b");
        settle(&mut o, &mut [&mut a, &mut b]);
        let ids: Vec<_> = o.doc.elements[0].features().iter().map(|f| f.id).collect();
        assert!(ids.contains(&fa) && ids.contains(&fb));
        assert_eq!(ids.len(), 6);
    }

    #[test]
    fn tabs_added_at_once_are_both_kept_and_a_removal_stays_removed() {
        let mut o = Owner::new(shared());
        let mut a = o.peer("a");
        let mut b = o.peer("b");
        a.edit(|d| d.elements.push(Element::part_studio("Part Studio 2")));
        b.edit(|d| d.elements.push(Element::part_studio("Part Studio 3")));
        // The owner removes Assembly 1 meanwhile.
        o.edit(|d| {
            d.elements.remove(1);
        });
        settle(&mut o, &mut [&mut a, &mut b]);
        let tabs: Vec<_> = o.doc.elements.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(tabs.len(), 3);
        assert!(tabs.contains(&"Part Studio 2") && tabs.contains(&"Part Studio 3"));
        assert!(!tabs.contains(&"Assembly 1"));
    }

    #[test]
    fn editing_a_tab_someone_removed_is_refused() {
        let mut o = Owner::new(shared());
        let mut a = o.peer("a");
        let mut b = o.peer("b");
        a.edit(|d| {
            d.elements.remove(0);
        });
        b.edit(|d| rename(d, 1, "b's"));
        assert_eq!(o.take(&mut a), Ok(()));
        assert!(matches!(o.take(&mut b), Err(Refusal::Conflict { .. })));
        settle(&mut o, &mut [&mut a, &mut b]);
        assert_eq!(o.doc.elements.len(), 1);
    }

    #[test]
    fn edits_made_while_one_is_in_flight_follow_it() {
        let mut o = Owner::new(shared());
        let mut a = o.peer("a");
        a.edit(|d| rename(d, 1, "one"));
        assert_eq!(o.take(&mut a), Ok(()));
        // A second edit before the first is confirmed: not sent yet, and kept when the
        // confirmation arrives.
        a.edit(|d| rename(d, 2, "two"));
        assert!(a.replica.propose(&a.doc).is_none());
        o.deliver(&mut a);
        assert_eq!(names(&a.doc)[1..3], ["one", "two"]);
        settle(&mut o, &mut [&mut a]);
    }

    #[test]
    fn edits_that_arrive_out_of_order_wait_for_the_missing_one() {
        let mut o = Owner::new(shared());
        let mut a = o.peer("a");
        o.edit(|d| rename(d, 1, "1"));
        o.edit(|d| rename(d, 2, "2"));
        let deliver = |a: &mut Peer, i: usize| {
            let (n, _, _, c, hash) = &o.out[i];
            let r = a.replica.receive(
                *n,
                Incoming {
                    own: None,
                    change: c.clone(),
                    hash: *hash,
                },
                &a.doc,
            );
            if let Some(d) = r.local {
                a.doc = d;
            }
        };
        deliver(&mut a, 1);
        assert!(a.replica.has_gap());
        assert_eq!(names(&a.doc)[1], "Top");
        deliver(&mut a, 0);
        assert!(!a.replica.has_gap());
        assert_eq!(a.doc, o.doc);
    }

    #[test]
    fn a_stale_edit_is_refused() {
        let mut s = Sequencer::new(10);
        let mut doc = shared();
        let mut after = doc.clone();
        rename(&mut after, 1, "y");
        let c = Change::between(&doc, &after);
        assert_eq!(s.admit("a", 3, c.clone(), &mut doc), Err(Refusal::Stale));
        assert_eq!(s.admit("a", 11, c.clone(), &mut doc), Err(Refusal::Stale));
        assert!(s.admit("a", 10, c, &mut doc).is_ok());
    }

    /// Renames feature `i` of the first Part Studio (an element-scoped command).
    #[derive(Debug)]
    struct Rename(usize, &'static str);

    impl crate::command::Command for Rename {
        fn label(&self) -> String {
            "Rename".into()
        }
        fn scope(&self) -> crate::command::Scope {
            crate::command::Scope::Whole
        }
        fn apply(&self, doc: &mut Document) -> Result<(), crate::command::CommandError> {
            rename(doc, self.0, self.1);
            Ok(())
        }
    }

    #[test]
    fn undo_after_a_remote_edit_takes_back_only_ones_own() {
        for scope_element in [false, true] {
            let mut doc = shared();
            let mut history = crate::command::History::default();
            if scope_element {
                // The same edit as a feature dialog makes it: scoped to the studio.
                let s = studio(&doc);
                #[derive(Debug)]
                struct RenameIn(ElementId);
                impl crate::command::Command for RenameIn {
                    fn label(&self) -> String {
                        "Rename".into()
                    }
                    fn scope(&self) -> crate::command::Scope {
                        crate::command::Scope::Element(self.0)
                    }
                    fn apply(
                        &self,
                        doc: &mut Document,
                    ) -> Result<(), crate::command::CommandError> {
                        rename(doc, 1, "mine");
                        Ok(())
                    }
                }
                history.execute(&mut doc, &RenameIn(s)).unwrap();
            } else {
                history.execute(&mut doc, &Rename(1, "mine")).unwrap();
            }
            // Someone else adds a feature and renames another, and a tab.
            let mut theirs = doc.clone();
            let added = add_feature(&mut theirs, "theirs");
            rename(&mut theirs, 2, "their front");
            theirs.elements.push(Element::part_studio("Their studio"));
            let c = Change::between(&doc, &theirs);
            c.apply_to(&mut doc);
            history.rebase(&c);
            history.undo(&mut doc);
            assert_eq!(names(&doc)[1..3], ["Top", "their front"], "{scope_element}");
            assert!(doc.elements[0].features().iter().any(|f| f.id == added));
            assert_eq!(doc.elements.len(), 3);
            history.redo(&mut doc);
            assert_eq!(names(&doc)[1..3], ["mine", "their front"]);
        }
    }

    #[test]
    fn an_undo_steps_copy_of_a_studio_takes_a_remote_edit() {
        let base = shared();
        let mut theirs = base.clone();
        let added = add_feature(&mut theirs, "Theirs");
        rename(&mut theirs, 1, "renamed by them");
        let c = Change::between(&base, &theirs);
        let mut copy = base.elements[0].clone();
        c.apply_to_element(&mut copy);
        assert_eq!(copy, theirs.elements[0]);
        assert!(copy.features().iter().any(|f| f.id == added));
    }
}
