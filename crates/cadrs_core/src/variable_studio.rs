//! The **Variable Studio** (Onshape's Variable Studio tab, `VariableStudio/variable_studio.htm`
//! and the `variables-and-expressions` course): a tab holding a table of variables that every
//! Part Studio and Assembly of the document can name (`#width`), without a Variable feature of
//! their own.
//!
//! - **The table** ([`VariableStudio::variables`]): rows of Name, Type (Length, Angle, Number,
//!   Any), Value (an expression that may name the rows above it) and Description, worked out by
//!   [`cadrs_vars::studio::evaluate`] (errors: used before defined, cycles, duplicates).
//! - **Inserted tables** ([`VariableStudio::inserted`], Insert Variable Studio): another Variable
//!   Studio's variables, all or some of them, shown above the studio's own rows and named by
//!   them. A Variable Studio of this document is followed live; one of another document is a
//!   frozen copy of a version ([`snapshot`]), labelled with the version ("Dimensions V1").
//!   Inserting a studio into one that it inserts (a cycle) is refused ([`would_cycle`]). A
//!   studio inserted live reaches the Part Studios only through itself (so not while it is
//!   unavailable); a version, through the studio it is inserted in.
//! - **One name, two studios** ([`conflicts`]): the later tab's variable is the one Part
//!   Studios see; the table says so on both rows.
//! - **Availability** ([`VariableStudio::available`], the checkbox at the bottom of the tab):
//!   unchecked while the variables are a draft; only checked studios reach the Part Studios and
//!   Assemblies.
//! - **Where the variables go** ([`propagate`], run after every command, undo and redo): each
//!   Part Studio and Assembly keeps the document's studio variables it can name
//!   ([`Element::studio_variables`], [`ImportedVariable`]), as values. They are defined above
//!   its whole list, so its own Variable features (and the mates' Variables) shadow them. A
//!   Part Studio's expressions are refreshed with them ([`crate::variables::refresh`]), and the
//!   ones its features name are handed to the rebuild as Variable features ahead of the list
//!   ([`Element::active_features`]), so a rebuild (in the kernel process too) sees exactly the
//!   values the document has and caches by them.
//!
//! Not built (Onshape has them): the **From import** type (a variable holding rows of a CSV
//! table), **Any** values that aren't numbers (strings, booleans, arrays, maps; `true` and
//! `false` read as 1 and 0), **configured** Variable Studios, and **Release** of a studio.

use cadrs_sketch::units::VarValue;
use cadrs_vars::studio::{self, StudioVariable, TableEval};
use cadrs_vars::{VariableFeature, VariableKind};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::command::{Command, CommandError, Scope};
use crate::document::{Document, Element, ElementKind, Feature, FeatureKind};
use crate::external::{RefAt, SourceRef};
use crate::ids::{ElementId, FeatureId};

fn yes() -> bool {
    true
}

fn is_yes(b: &bool) -> bool {
    *b
}

/// A Variable Studio tab's contents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableStudio {
    /// Tables inserted from other Variable Studios, above the own rows, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inserted: Vec<InsertedTable>,
    /// The studio's own rows, top to bottom.
    #[serde(default)]
    pub variables: Vec<StudioVariable>,
    /// Whether Part Studios and Assemblies can use the variables (the checkbox at the bottom).
    #[serde(default = "yes", skip_serializing_if = "is_yes")]
    pub available: bool,
}

impl Default for VariableStudio {
    fn default() -> Self {
        Self {
            inserted: Vec::new(),
            variables: Vec::new(),
            available: true,
        }
    }
}

impl VariableStudio {
    /// The own row `id`.
    pub fn row(&self, id: Uuid) -> Option<&StudioVariable> {
        self.variables.iter().find(|r| r.id == id)
    }

    pub fn row_mut(&mut self, id: Uuid) -> Option<&mut StudioVariable> {
        self.variables.iter_mut().find(|r| r.id == id)
    }

    /// Change version: `table` takes the place (and the id) of the inserted table `id`.
    /// Returns false if there is no such table.
    pub fn replace_inserted(&mut self, id: Uuid, table: InsertedTable) -> bool {
        let Some(slot) = self.inserted.iter_mut().find(|t| t.id == id) else {
            return false;
        };
        *slot = InsertedTable { id, ..table };
        true
    }
}

/// Another Variable Studio's variables, inserted (Insert Variable Studio).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InsertedTable {
    pub id: Uuid,
    /// The source studio: of this document at its workspace (followed live), or a version.
    pub source: SourceRef,
    /// The source tab's name when it was inserted (shown if the source is gone).
    pub name: String,
    /// The source document's name, for another document.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub document_name: String,
    /// The version's name ("V1"), for a version.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub version_name: String,
    /// The variables picked; empty: the whole table.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub only: Vec<String>,
    /// A version's variables as they were, with their values (a version can't change).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frozen: Vec<StudioVariable>,
}

impl InsertedTable {
    /// A same-document studio, followed live.
    pub fn live(source: ElementId, name: impl Into<String>, only: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            source: SourceRef {
                document: None,
                at: RefAt::Workspace,
                element: source,
                pinned: false,
            },
            name: name.into(),
            document_name: String::new(),
            version_name: String::new(),
            only,
            frozen: Vec::new(),
        }
    }

    /// Whether it follows a studio of this document live.
    pub fn is_live(&self) -> bool {
        self.source.document.is_none() && self.source.at == RefAt::Workspace
    }

    /// How its header reads: "Variable Studio 1", "Dimensions › Variable Studio 1 V1".
    pub fn label(&self, doc: &Document) -> String {
        let name = if self.is_live() {
            doc.element(self.source.element)
                .map_or(self.name.as_str(), |e| e.name.as_str())
        } else {
            self.name.as_str()
        };
        let mut s = String::new();
        if !self.document_name.is_empty() {
            s.push_str(&self.document_name);
            s.push_str(" › ");
        }
        s.push_str(name);
        if !self.version_name.is_empty() {
            s.push(' ');
            s.push_str(&self.version_name);
        }
        s
    }

    fn picks(&self, name: &str) -> bool {
        self.only.is_empty() || self.only.iter().any(|n| n == name)
    }
}

/// A variable a Variable Studio defines, with its value.
#[derive(Debug, Clone, PartialEq)]
pub struct Exported {
    pub id: Uuid,
    /// The variable, its `value` and `kind` as evaluated.
    pub var: VariableFeature,
    /// It comes from a Variable Studio of this document inserted live: that studio gives it to
    /// the Part Studios (or not, while it is unavailable), not this one.
    pub live: bool,
}

impl Exported {
    pub fn var_value(&self) -> VarValue {
        self.var.var_value()
    }
}

/// An inserted table worked out.
#[derive(Debug, Clone, PartialEq)]
pub struct InsertedEval {
    pub table: Uuid,
    pub label: String,
    /// Its variables (the picked ones), in order.
    pub vars: Vec<Exported>,
    /// Why it gives nothing or not all it should (the source is gone, a cycle, a picked name
    /// the source no longer has).
    pub error: Option<String>,
}

/// A Variable Studio worked out.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StudioEval {
    pub inserted: Vec<InsertedEval>,
    /// The own rows.
    pub table: TableEval,
    /// Everything it defines, in order: the inserted tables', then the own rows'.
    pub exported: Vec<Exported>,
}

impl StudioEval {
    /// The inserted and own variables' values, for expressions.
    pub fn env(&self) -> Vec<(String, VarValue)> {
        self.exported
            .iter()
            .map(|e| (e.var.name.clone(), e.var_value()))
            .collect()
    }
}

/// The Variable Studio of an element.
pub fn studio_of(el: &Element) -> Option<&VariableStudio> {
    match &el.kind {
        ElementKind::VariableStudio(s) => Some(s),
        _ => None,
    }
}

/// A Variable Studio's contents standing in for what the document has (an edit not made yet).
type Over<'a> = Option<(ElementId, &'a VariableStudio)>;

/// The contents of the Variable Studio `id`: `over`'s if it is that one.
fn contents<'a>(doc: &'a Document, id: ElementId, over: Over<'a>) -> Option<&'a VariableStudio> {
    match over {
        Some((o, s)) if o == id => Some(s),
        _ => studio_of(doc.element(id)?),
    }
}

/// Works out the Variable Studio `id` of `doc` (its inserted tables, then its rows).
pub fn evaluate(doc: &Document, id: ElementId) -> Option<StudioEval> {
    eval_in(doc, id, None, &mut Vec::new())
}

/// Works out `studio` as if it were the contents of `id` in `doc` (a table being edited).
pub fn evaluate_contents(doc: &Document, id: ElementId, studio: &VariableStudio) -> StudioEval {
    eval_contents(doc, studio, Some((id, studio)), &mut vec![id])
}

fn eval_in(
    doc: &Document,
    id: ElementId,
    over: Over<'_>,
    stack: &mut Vec<ElementId>,
) -> Option<StudioEval> {
    let s = contents(doc, id, over)?;
    stack.push(id);
    let out = eval_contents(doc, s, over, stack);
    stack.pop();
    Some(out)
}

fn eval_contents(
    doc: &Document,
    s: &VariableStudio,
    over: Over<'_>,
    stack: &mut Vec<ElementId>,
) -> StudioEval {
    let units = &doc.units;
    let mut out = StudioEval::default();
    for t in &s.inserted {
        let label = t.label(doc);
        let (all, mut error): (Vec<Exported>, Option<String>) = if t.is_live() {
            if stack.contains(&t.source.element) {
                (
                    Vec::new(),
                    Some(format!("{label} inserts this Variable Studio (a cycle)")),
                )
            } else {
                match eval_in(doc, t.source.element, over, stack) {
                    Some(e) => (
                        e.exported
                            .into_iter()
                            .map(|x| Exported { live: true, ..x })
                            .collect(),
                        None,
                    ),
                    None => (Vec::new(), Some(format!("{} was deleted", t.name))),
                }
            }
        } else {
            let all = t
                .frozen
                .iter()
                .filter(|r| r.var.problem().is_none())
                .map(|r| Exported {
                    id: r.id,
                    var: r.var.clone(),
                    live: false,
                })
                .collect();
            (all, None)
        };
        if error.is_none()
            && let Some(n) = t
                .only
                .iter()
                .find(|n| !all.iter().any(|e| e.var.name == **n))
        {
            error = Some(format!("{label} no longer has #{n}"));
        }
        let vars: Vec<Exported> = all.into_iter().filter(|e| t.picks(&e.var.name)).collect();
        out.exported.extend(vars.iter().cloned());
        out.inserted.push(InsertedEval {
            table: t.id,
            label,
            vars,
            error,
        });
    }
    let base = out.env();
    out.table = studio::evaluate(&s.variables, &base, units);
    for (id, v) in &out.table.defined {
        if let Some(r) = s.variables.iter().find(|r| r.id == *id) {
            let mut var = r.var.clone();
            var.value = v.value;
            var.kind = VariableKind::of(v.quantity);
            out.exported.push(Exported {
                id: *id,
                var,
                live: false,
            });
        }
    }
    out
}

/// Whether inserting `source` into the Variable Studio `into` would make a cycle (`source` is
/// `into`, or inserts it, at any depth).
pub fn would_cycle(doc: &Document, into: ElementId, source: ElementId) -> bool {
    fn reaches(doc: &Document, from: ElementId, target: ElementId, depth: usize) -> bool {
        if from == target {
            return true;
        }
        // Deeper than any real chain of inserts: refused as if it were a cycle.
        if depth > 64 {
            return true;
        }
        doc.element(from).and_then(studio_of).is_some_and(|s| {
            s.inserted
                .iter()
                .filter(|t| t.is_live())
                .any(|t| reaches(doc, t.source.element, target, depth + 1))
        })
    }
    reaches(doc, source, into, 0)
}

/// A frozen copy of the Variable Studio `element` of `source` (another document at a version,
/// or this one at a version), its picked variables (`only`; empty: all) with their values.
pub fn snapshot(
    source: &Document,
    r: SourceRef,
    version_name: &str,
    only: Vec<String>,
) -> Result<InsertedTable, String> {
    let el = source
        .element(r.element)
        .ok_or_else(|| "That Variable Studio isn't in the version".to_string())?;
    let eval = evaluate(source, r.element)
        .ok_or_else(|| format!("{} isn't a Variable Studio", el.name))?;
    let frozen: Vec<StudioVariable> = eval
        .exported
        .into_iter()
        .filter(|e| only.is_empty() || only.contains(&e.var.name))
        .map(|e| StudioVariable {
            id: e.id,
            var: e.var,
        })
        .collect();
    if frozen.is_empty() {
        return Err(format!("{} has no variables to insert", el.name));
    }
    Ok(InsertedTable {
        id: Uuid::new_v4(),
        source: r,
        name: el.name.clone(),
        document_name: if r.document.is_some() {
            source.name.clone()
        } else {
            String::new()
        },
        version_name: version_name.to_string(),
        only,
        frozen,
    })
}

// ---------------------------------------------------------------------------------------------
// The variables Part Studios and Assemblies see

/// A Variable Studio's variable as a Part Studio or Assembly keeps it
/// ([`Element::studio_variables`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportedVariable {
    /// The row's id (the id of the Variable feature a rebuild sees for it).
    pub id: FeatureId,
    pub name: String,
    /// mm, degrees or a plain number.
    pub value: f64,
    pub kind: VariableKind,
    /// The Variable Studio it comes from.
    pub studio: ElementId,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Whether the element's own expressions name it (only those reach the rebuild).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub used: bool,
}

impl ImportedVariable {
    pub fn var_value(&self) -> VarValue {
        VarValue {
            value: self.value,
            quantity: self.kind.quantity(),
        }
    }

    /// The Variable feature a rebuild sees for it: `#name` set to its exact value.
    pub fn feature(&self) -> Feature {
        let var = VariableFeature {
            name: self.name.clone(),
            var_type: self.kind.var_type(),
            expr: self.kind.literal(self.value),
            value: self.value,
            kind: self.kind,
            description: self.description.clone(),
        };
        Feature::new(
            self.id,
            format!("#{}", self.name),
            FeatureKind::Variable(var),
        )
    }
}

/// The variables of the document's available Variable Studios, in tab order (a later studio's
/// variable of the same name wins, as in an expression; [`conflicts`] lists such names). A
/// studio inserted live into another reaches the Part Studios only through itself (so only
/// while it is available); a version inserted from another document, through the studio it is
/// inserted in. `used` is left false.
pub fn document_variables(doc: &Document) -> Vec<ImportedVariable> {
    let mut out: Vec<ImportedVariable> = Vec::new();
    for (el, e) in exported_by_studio(doc) {
        out.retain(|x| x.name != e.var.name);
        out.push(ImportedVariable {
            id: FeatureId(e.id),
            name: e.var.name.clone(),
            value: e.var.value,
            kind: e.var.kind,
            studio: el,
            description: e.var.description.clone(),
            used: false,
        });
    }
    out
}

/// What each available Variable Studio gives the Part Studios, in tab order.
fn exported_by_studio(doc: &Document) -> Vec<(ElementId, Exported)> {
    let mut out = Vec::new();
    for el in &doc.elements {
        let Some(s) = studio_of(el) else { continue };
        if !s.available {
            continue;
        }
        let Some(eval) = evaluate(doc, el.id) else {
            continue;
        };
        out.extend(
            eval.exported
                .into_iter()
                .filter(|e| !e.live)
                .map(|e| (el.id, e)),
        );
    }
    out
}

/// Names that more than one available Variable Studio gives the Part Studios: the name, the
/// studio whose variable is used (the last in tab order) and the others.
pub fn conflicts(doc: &Document) -> Vec<(String, ElementId, Vec<ElementId>)> {
    let mut by_name: Vec<(String, Vec<ElementId>)> = Vec::new();
    for (el, e) in exported_by_studio(doc) {
        match by_name.iter_mut().find(|(n, _)| *n == e.var.name) {
            Some((_, studios)) => {
                studios.retain(|s| *s != el);
                studios.push(el);
            }
            None => by_name.push((e.var.name.clone(), vec![el])),
        }
    }
    by_name
        .into_iter()
        .filter(|(_, s)| s.len() > 1)
        .map(|(n, mut s)| {
            let winner = s.pop().expect("more than one");
            (n, winner, s)
        })
        .collect()
}

/// The names an Assembly's mate features name.
fn assembly_names(asm: &crate::assembly::Assembly) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for f in &asm.mates {
        let texts: Vec<&str> = match &f.kind {
            crate::assembly::mate::MateKind::Variable(v) => vec![v.expr.as_str()],
            _ => f.exprs.iter().map(|(_, e)| e.as_str()).collect(),
        };
        for t in texts {
            for n in cadrs_sketch::units::variable_names(t) {
                if !out.contains(&n) {
                    out.push(n);
                }
            }
        }
    }
    out
}

/// Brings every Part Studio's and Assembly's [`Element::studio_variables`] up to date with the
/// document's Variable Studios and refreshes the expressions that name them (see the module
/// docs). `scope` is what the last command changed: a Part Studio's own edit only needs that
/// Part Studio's uses looked at again. Cheap when the document has no Variable Studio.
pub fn propagate(doc: &mut Document, scope: Option<Scope>) {
    let has_studio = doc
        .elements
        .iter()
        .any(|e| matches!(e.kind, ElementKind::VariableStudio(_)));
    if !has_studio && doc.elements.iter().all(|e| e.studio_variables.is_empty()) {
        return;
    }
    let only = match scope {
        Some(Scope::Element(e) | Scope::ElementName(e))
            if doc
                .element(e)
                .is_some_and(|el| !matches!(el.kind, ElementKind::VariableStudio(_))) =>
        {
            Some(e)
        }
        _ => None,
    };
    let all = if has_studio {
        document_variables(doc)
    } else {
        Vec::new()
    };
    let ids: Vec<ElementId> = doc
        .elements
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                ElementKind::PartStudio { .. } | ElementKind::Assembly
            ) && only.is_none_or(|o| o == e.id)
        })
        .map(|e| e.id)
        .collect();
    for id in ids {
        let Some(el) = doc.element(id) else { continue };
        let names = match el.assembly_model() {
            Some(asm) => assembly_names(asm),
            None => crate::variables::names_used(el.features()),
        };
        let want: Vec<ImportedVariable> = all
            .iter()
            .map(|v| ImportedVariable {
                used: names.contains(&v.name),
                ..v.clone()
            })
            .collect();
        if el.studio_variables == want {
            continue;
        }
        let values_changed = el.studio_variables.len() != want.len()
            || el.studio_variables.iter().zip(&want).any(|(a, b)| {
                (a.name.as_str(), a.value, a.kind) != (b.name.as_str(), b.value, b.kind)
            });
        let units = doc.units;
        let Some(el) = doc.element_mut(id) else {
            continue;
        };
        el.studio_variables = want;
        if !values_changed {
            continue;
        }
        if matches!(el.kind, ElementKind::Assembly) {
            let base = el.variable_base();
            if let Some(asm) = el.assembly_model_mut() {
                crate::assembly::vars::refresh(&base, asm, &units);
            }
        } else {
            crate::commands::refresh_studio(doc, id);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Commands

/// Replaces a Variable Studio's contents (a row added, edited, moved or deleted, a table
/// inserted or removed, the availability checkbox): one undo step. The rows keep the values
/// they evaluate to. `poses` are Assemblies' placements re-solved for the mate offsets that
/// followed the change (the app solves them once the change is made, then repeats the command
/// with them and merges the two undo steps).
#[derive(Debug, Clone)]
pub struct SetVariableStudio {
    pub element: ElementId,
    pub studio: VariableStudio,
    pub label: String,
    pub poses: Vec<(
        ElementId,
        Vec<(crate::assembly::InstanceId, crate::assembly::Pose)>,
    )>,
    /// Whether the change can reach other tabs (see [`SetVariableStudio::new`]): then its undo
    /// step holds the whole document, else only the studio.
    pub reaches_others: bool,
}

impl SetVariableStudio {
    /// A change that moves no assembly instance. It can reach other tabs when a Part Studio or
    /// Assembly uses a variable it gives them (its own, a same-named one, or one of a studio
    /// inserting it), or when the studio's names, inserts or availability change (a name a tab
    /// names, but nothing defined yet, may now be defined).
    pub fn new(
        doc: &Document,
        element: ElementId,
        studio: VariableStudio,
        label: impl Into<String>,
    ) -> Self {
        fn names(s: &VariableStudio) -> Vec<&str> {
            let mut n: Vec<&str> = s.variables.iter().map(|r| r.var.name.as_str()).collect();
            n.sort_unstable();
            n
        }
        let renamed = doc.element(element).and_then(studio_of).is_none_or(|old| {
            names(old) != names(&studio)
                || old.inserted != studio.inserted
                || old.available != studio.available
        });
        // A tab uses one of its variables: from this studio, from a studio that inserts it
        // (whose rows may name its variables), or of a name it defines.
        let defines = |name: &str| studio.variables.iter().any(|r| r.var.name == name);
        let used = doc.elements.iter().any(|e| {
            e.studio_variables.iter().any(|v| {
                v.used
                    && (v.studio == element
                        || defines(&v.name)
                        || would_cycle(doc, element, v.studio))
            })
        });
        Self {
            element,
            studio,
            label: label.into(),
            poses: Vec::new(),
            reaches_others: renamed || used,
        }
    }
}

impl Command for SetVariableStudio {
    fn label(&self) -> String {
        self.label.clone()
    }
    // The whole document: undo puts back exactly what the change re-derived in the Part
    // Studios and Assemblies (sketch solves, placements), rather than deriving it again.
    fn scope(&self) -> Scope {
        if self.reaches_others || !self.poses.is_empty() {
            Scope::Whole
        } else {
            Scope::Element(self.element)
        }
    }
    fn apply(&self, doc: &mut Document) -> Result<(), CommandError> {
        let el = doc
            .element(self.element)
            .ok_or(CommandError::ElementNotFound(self.element))?;
        if studio_of(el).is_none() {
            return Err(CommandError::Invalid("not a Variable Studio".into()));
        }
        for t in &self.studio.inserted {
            if t.is_live() && would_cycle(doc, self.element, t.source.element) {
                return Err(CommandError::Invalid(format!(
                    "{} inserts this Variable Studio",
                    t.label(doc)
                )));
            }
        }
        let mut studio = self.studio.clone();
        let eval = evaluate_contents(doc, self.element, &studio);
        studio::write_values(&mut studio.variables, &eval.table);
        if let Some(el) = doc.element_mut(self.element) {
            el.kind = ElementKind::VariableStudio(Box::new(studio));
        }
        // Inside the step (its snapshot is the whole document), so undo restores what follows.
        propagate(doc, None);
        // The mates have followed; their instances take the solved placements.
        for (asm, poses) in &self.poses {
            crate::assembly::structure::place(doc, *asm, poses)?;
        }
        Ok(())
    }
}
