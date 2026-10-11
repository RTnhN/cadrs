# Plan: PCB boards in Part Studios and Assemblies (place, fit, edit layout in 3D)

Status: plan only, nothing built. Written 2026-10-11 after reading `cadrs_core::pcb`,
`cadrs_pcb`, `cadrs_eda`, `cadrs_app::{pcb, eda, derived_ui, assembly::in_context}`,
`cadrs_core::{derived, rebuild/kernel_ops/derived, import, picture, assembly::context}`,
`docs/PLAN.md` and `reference/onshape/training/{pcb-studio,derived-and-linking,managed-in-context-design}.md`.

## 1. The goal, in the user's words

The green board in the enclosure (extrusion + end plate + mains plug) has to be a real PCB
Studio board, not a stand-in extrude. In the Part Studio (or Assembly) we want to:

1. **Place a board**: extend **Derived** so it can bring in a PCB Studio board, not only a
   Part Studio.
2. **Switch the board between its 3D model and its 2D layout**, as a sketch and the part
   extruded from it are both visible things.
3. **Make or update the board's outline from a Part Studio sketch** (so it follows the
   enclosure).
4. **Line components up with the 3D world** (the connector with the plug and the end plate,
   large parts in the space they have): move footprints in XY of the board **in the 3D
   view, seeing the 3D result while moving**, i.e. the PCB Studio's layout editing brought
   into the Part Studio.

## 2. What exists today (the parts we build on)

| Piece | Where | What it gives us | Gap |
|---|---|---|---|
| Native board model | `cadrs_eda::Design` (schematic + `board::Board`), nm, Y up; outline = shapes on `Layer::Outline`; `PlacedFootprint { id: Uuid, placement, footprint (models) }` | The single source of truth for a board | Nothing links its outline to anything outside the PCB Studio |
| PCB Studio element | `cadrs_core::pcb::{PcbStudio, StudioBoard, BoardSource::Native, SetDesign}` | Boards with stable `BoardId`; every layout edit is one `SetDesign` (undoable) | `SetDesign` is only issued from the PCB Studio tab |
| Board → mechanical | `pcb::design::pcb_board` (IDF-style board: outline loops, placements, courtyard boxes) | Used by the studio's 3D view, Create assembly, Sync | Boxes, not models |
| Board → kernel B-rep with real models | `cadrs_pcb::step` (board body with cut-outs/holes; each footprint's STEP as is, VRML meshed, else generated `model3d` body) | Exactly the geometry a Part Studio needs | Lives in `cadrs_pcb`, which **depends on** `cadrs_core`; the rebuild can't call it |
| Board 3D display | `cadrs_app::pcb::view` (meshes + `skins()`: mask/copper/silk rendered to a texture on the top and bottom faces) | The "as made" look | Display-only meshes in the PCB Studio, not parts |
| Layout 2D view + tools | `cadrs_app::eda::{mod, layout_tools}`; `cadrs_eda::render` → `DrawList` (lines + triangles); tools take `EdaClick { at: Pt }` | All layout editing (move/rotate/flip, route, zones, outline, keep-outs, ruler…) | Hard-wired to the 2D camera of the PCB Studio (`Eda2d`, `navigate`) |
| Derived feature | `cadrs_core::derived` + `rebuild/kernel_ops/derived.rs` + `derived_ui.rs` | Source picker (this doc / other docs / versions), live workspace refresh after every command, locations (mate connectors) + base placement, namespaced ids, frozen version copies, cycle rules | Sources are Part Studios only |
| Picture on a part | `cadrs_core::picture` → `solid::SolidImage` | A texture carried by a part, transformed with it | Rectangle only |
| In-context editing | `assembly::context`, `cadrs_app::assembly::in_context` | Snapshot of the assembly around a Part Studio, ghosted, pickable | — |
| IDF round trip | Sync / Create assembly (`cadrs_pcb::{sync, create_assembly}`) | The Onshape course flow for IDF boards | Stays as is for IDF boards; the new flow replaces it for native boards |

The important consequence: **all four goals reuse existing machinery**. The new work is
mostly (a) a board-aware Derived rebuild, (b) making the layout tools run on a board plane in
the 3D viewport, and (c) a sketch → outline link.

## 3. Key decisions (recommended)

1. **The PCB Studio board stays the single source of truth.** A Part Studio never owns a copy
   it can diverge from. The Derived feature embeds a snapshot of the board's design (as it
   embeds a source Part Studio's features today) only so any rebuild works without lookups and
   the cache key changes exactly when the board does.
2. **Editing the layout from a Part Studio issues the same `SetDesign` on the PCB Studio
   element.** It is one undo step in the document's history; Ctrl+Z in the Part Studio undoes a
   footprint move. `derived::refresh` (already run after every command, undo and redo) brings
   the embedded snapshot up to date, so the Part Studio rebuilds.
3. **Only workspace references in this document are editable.** A Derived board from a version
   or another document is read-only (as derived parts are), with "Open linked document" to edit
   it at the source.
4. **The board is a real part; components are display-mesh parts** (answer Q1). The board body
   is a kernel solid (cheap: an extruded outline), so sketch on its face, Use its edges, mate to
   it and Measure work. Each component is a part with a `Solid` built from its model's
   triangles and **no kernel body** (`PartState::body = None`, already allowed): visible,
   selectable, hideable, in the BOM, with mate connectors on its planar faces (found from the
   mesh, 5), measurable by triangles, but left out of booleans and exact interference.
   Rebuilds stay fast whatever the STEP files weigh.
5. **Stable ids from the layout's ids.** The board part's id comes from (Derived feature,
   "board"); each component part's from (Derived feature, footprint `Uuid`, copy). Moving,
   rotating or re-annotating a footprint keeps its part id and face names, so a mate to J1's
   face, or a sketch that Uses an edge of the board, survives layout edits.
6. **Move the board builder into `cadrs_core`** (`rebuild/kernel_ops/board.rs`: board solid +
   component meshes), with `cadrs_pcb::step` keeping the STEP-export path (models as solids,
   for export only) on top of it. `cadrs_core` already depends on `cadrs_eda` and the kernel,
   so no new dependency.
7. **The layout tools become host-agnostic.** They already work in board coordinates through
   `EdaClick { at: Pt }`; we lift "where am I and where do I commit" into a `LayoutHost`
   (2D PCB Studio view or a board plane in the 3D viewport) instead of copying them.
8. **Dragging never touches the kernel.** During a drag the component's display meshes are
   moved by the delta transform; the rebuild happens once, on release, and is cheap because
   component meshes are cached per model file + model transform and only re-placed.

### 3.1 Answers to the questionnaire (2026-10-11)

| # | Question | Answer | Where it changes the plan |
|---|---|---|---|
| Q1 | Component models: kernel solids or display meshes? | **Display meshes only** | Decision 4, sections 5 and 7.3 (mesh-based clearance, align), P1/P4, risks |
| Q2 | What the Part Studio edit mode allows | **Everything** the PCB Studio layout does (route, vias, zones, text, graphics, outline, keep-outs, placement) | 7.1: the full tool strip and layers panel |
| Q3 | Sketch-linked outline: live or one-shot | **Live** (Break link to edit by hand) | 8.3 as written |
| Q4 | Default seat on the sketch plane / location | **Bottom face** (board grows along the normal) | `BoardSeat::Bottom` default, as written |
| Q5 | Boards in Assemblies | **Direct insert and via a Part Studio** | Section 9 / P6 as written |

### 3.2 Built so far, and where it differs from the plan (2026-10-11)

- **P0–P2 done.** Core: `pcb::model` (model meshes, cached per process; footprint placement),
  `pcb::mesh_solid` (model meshes as mesh parts: planar faces with frames, cylinders with axes,
  coarse 8-sided pins included), `pcb::board3d` (board solid with holes and cut-outs, faces named
  by the plan curve they come from — outline segment geometry, or footprint + pad — so a new hole
  renames nothing), `derived::DerivedBoard`. The STEP export and the PCB Studio's 3D view use the
  same board and placement code.
- **Board point instead of `Board::origin` + base connector.** A derived board is placed by
  `BoardAnchor`: *Fixed* (the outline's centre when derived — the default, so outline edits never
  move the parts), *Origin* (layout 0, 0 — boards made from a sketch), *Centre* (follows the
  outline) or *Footprint* (a footprint's origin, e.g. a connector, lands on the location). Base
  mate connector is refused for boards; the Footprint point covers its use.
- **Parts placed off the board** (a module waiting to be placed) stay out unless asked for.
- **Layout view as a picture.** The layout on the board face is rasterised (32 px/mm) onto one
  quad per face (`cadrs_eda::skin::raster`), not a triangle mesh: a 300 k-triangle overlay made
  every frame take a second in Bevy.
- **P3 done.** `LayoutHost` became a second field of `eda::Eda2d` (`PlaneHost`); all Layout tools
  work on the board in the 3D view; drags show the component's part as a ghost.
- **P4.** Parts around, snapping (pads onto pin axes, shadow edges onto outlines), room readouts,
  "In the way", Align (incl. a pin's axis mark as target), keep-outs from the parts around (per
  side; the PCB Studio's DRC checks footprints against keep-outs, which is the mechanical
  DRC of §7.3 item 8), collision warnings on the Derived feature (named as the Parts list does).
  Exact distances are mesh-based (Q1).
- **Second review's fixes.** One `pcb::frames` module converts layout ↔ board ↔ Part Studio
  (with round-trip tests); the layout hash is bincode-streamed, kept by `StudioBoard::set_design`
  and worked out on reading. Collisions are checked in height bands split at faces flat to the
  board (a part over a low step of a tall component is clear), per triangle culled, cached per
  pair of shapes, and say "(0.80 mm in plan, 2.40 mm high)"; gaps are exact polygon distances
  plus "↑ room" over the part. Snapping goes to edges running beside the part's shadow. Align
  turns a footprint (face to face, at most a quarter turn), says what is still off (out of the
  board's plane, overlapping courtyards or pads). Keep-outs keep holes (cut into hole-free
  pieces), replace their own on a second run, carry the room under the part in their name and
  toast what they did. The rebuild keeps the board's solid per outline and the placed component
  meshes per place; modelling features leave components alone with a warning (failing only when
  nothing else was picked); a Transform moves the board's frame too. In edit mode components are
  see-through (Solid parts turns that off), the parts around can be faded, Move by… (Shift+M)
  takes typed distances, the PCB Studio's DRC lists the components running into Part Studio
  parts, viewport hover is off (except for Align) and the bar is `cadrs_ui::FloatBar`. The
  Feature list's view row is a 3D | Layout `cadrs_ui::Segmented`.
- **P5 done.** `pcb::outline_link`: Create PCB board from a sketch's regions (a new PCB Studio
  or an existing one; the board on the sketch's plane, its origin on the sketch's, one undo
  step), Link outline to a sketch / Break outline link from the Derived feature's menu; the
  linked shapes are locked (the outline tool and Delete refuse them, saying why); a gone region
  keeps the last outline with a problem note; a link to a sketch after the board's Derived
  feature is refused (it would be a cycle).
- **Third review's fixes.** `StudioBoard::design` is private (only `set_design` changes it, so the
  hash can't go stale); a drag moves only its footprint in the preview (no per-frame copy of the
  design) and the stored layout's picture leaves the moved footprints out; a refused drop drops
  its ghost; room readouts are worked out only when the place changes, follow a turn, and cull
  triangles; snapping also goes to other footprints' courtyards; keep-outs from the parts around
  carry the room under them (`Keepout::max_height`: the DRC lets a footprint whose model is low
  enough under); component meshes have volume and mass (closed-mesh integration); collisions are
  cached per pair of solids (by `Arc`); Composite, loft, fill and split name a board component
  they can't use. P5: Create PCB board… is a dialog (name, PCB Studio or a new one, thickness,
  which face lies on the sketch); a PCB Studio board's menu links its outline to any Part Studio
  sketch, or breaks the link, and its Boards row shows the link (a warning when broken); a link
  whose sketch moved below the board's Derived feature says so (refresh re-checks the order), as
  the Derived feature's warning too; the Derived dialog says when a board lies on its sketch (a
  picked location takes over); the Outline tool draws cut-outs on a linked board.
- **P6 done (direct insert).** A rigid instance can be a PCB Studio's board
  (`InstanceSource::Studio { element, board: Some(..) }` rather than a new variant): the studio
  supplies its boards as Derived features (`PcbStudio::part_features`, cached, cleared on any
  change), so the rebuild, the occurrences, BOM rows (the board and each component) and in-context
  snapshots work as for a Part Studio. Insert ▸ Boards lists the document's boards. Edit layout
  on the instance opens it in its PCB Studio; the Part Studio route remains for fitting against
  parts. A board instance's parts are worked out from its layout each time
  (`PcbStudio::board_parts`, the ids follow from the footprints'), so a footprint added or
  removed later comes and goes in the assembly; the structured BOM lists the board with its
  components under it. Export (STEP, IGES, meshes), Check interference and section caps make
  components' solids from their models (`Rebuilder::placed_body`), and in-context snapshots go
  stale when a board's layout changes. The studio places its boards at their layout's origin
  (outline edits move no component).
- **P6: Edit layout in context.** A board instance's menu: Edit layout ▸ In context (here) or
  In its PCB Studio. In context the assembly hosts the Part Studio's edit mode: each top-level
  board instance is given to the view as a derived board's output keyed by the instance's id
  (its frame where the instance puts it, its parts the occurrences), and the layout is the PCB
  Studio's own (`board_in_studio::instance_board`); the parts around are the other instances;
  a drop is one `SetDesign` in the PCB Studio and the instance follows. Instance drags, box
  selection and the triad keep off while editing. (Subassemblies' boards: in their own tab.)
- **P7 done.** MCP tools (`cadrs_mcp`, logic in `cadrs_core::pcb::ops`): `get_board` (outline,
  footprints with pads and nets, DRC errors, where it is derived), `get_board_fit` (the pins and
  holes of the parts square to the board, collisions), `derive_board`, `create_board_from_sketch`,
  `move_footprint` (to a place, or a pad onto a part's pin), `add_footprint`, `remove_footprint`
  (and its symbol), `link_outline` (keeping the parts' place on the board). A Part Studio's STEP
  export writes components as their models' solids (`pcb::model_solids`, shared with the PCB
  Studio's own STEP export), where the board copy has them. Drawing views project components the
  same way; a board shown as its layout appears alone. A view square to a board draws the
  picture on the face it sees (`ViewGeometry::pictures`, any part's pictures: board skins and
  Image features) under its lines, faded toward the paper in a line view so the lines read, in
  a shaded view among the faces by depth (the components cover it); left out at a slant, in a
  section, cropped, detail or broken view, or when a solid part lies in front of any of it.
  Sheet pictures (`cadrs_drawing::export::Picture`) carry their edges, so PDF and PNG exports
  draw them turned or mirrored. IDF boards derive as their outline, drilled holes and a box per package
  (`design::idf_layout`, stable ids), read-only (Edit layout says why), in the Derived picker,
  the assembly Insert dialog and direct inserts.
- **Since.** A board's skins and the layout overlay show only from outside their face (the far
  face's picture showed through the board at an angle); a Transform of a board's part moves its
  components; the Parts list shows a board as a folder of its components; a wire pad whose
  footprint cuts a hole round itself counts as on the board.
- **Since (2).** Assembly instances carry their parts' pictures through their pose (a mated
  board's skins move with it); Create PCB board can make the sketch's circles NPTH holes; an
  in-context snapshot of a board goes stale when its layout changes (tested).

## 4. Data model changes

### 4.1 Derived: a board source (`cadrs_core::derived`)

Extend `DerivedFeature` rather than adding a new feature kind (one source picker, one set of
rules, one refresh, one Reference manager path):

```rust
pub struct DerivedFeature {
    pub source: Option<SourceRef>,          // the Part Studio **or the PCB Studio** tab
    // … existing fields …
    /// A PCB Studio source: which board, its design as at the reference, and the options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub board: Option<Box<DerivedBoard>>,
}

pub struct DerivedBoard {
    pub board: BoardId,
    pub board_name: String,
    /// The board's design at the reference, without the schematic (tracks, zones, silk stay:
    /// the skins need them). Its hash is kept beside it (see 4.4).
    pub design: Box<cadrs_eda::Design>,
    pub design_hash: u64,
    pub options: BoardOptions,
}

pub struct BoardOptions {
    /// What the copies show: the board's 3D model or its flat layout (section 6).
    pub view: BoardView,                    // ThreeD | Layout
    /// Which components come in: all, top only, bottom only, none; plus a list left out.
    pub components: ComponentFilter,
    pub exclude: Vec<Uuid>,                 // footprint ids
    /// Mask/copper/silk textures on the board faces.
    pub skins: bool,
    /// Which board face goes onto the location: the bottom face (default; the board grows
    /// along the location's +Z) or the top face.
    pub seat: BoardSeat,                    // Bottom | Top
}
```

- `DerivedSelection` for a board: `all` = board + components; or picked: the board, components
  by footprint id, keep-outs (as thin parts, optional).
- `DerivedPlacement::BaseOrigin` = the board's **place origin** (new `Board::origin: Pt`,
  default (0,0), set in the layout like KiCad's place/drill origin). `BaseConnector` works
  unchanged: any implicit connector on the board or a component part (e.g. J1's mating face),
  because those are now real faces with names.
- `refresh()` gains the board case: for each same-document workspace board reference, compare
  `design_hash` with the studio board's and copy the design in when it changed.
- `check()` gains: the board's outline link (4.2) must not come from a sketch that is after
  this Derived feature in the same Part Studio, or that Uses its geometry (cycle).
- Deriving the same board twice in one Part Studio: allowed through several locations of one
  feature (as today), refused as two features (same rule as Part Studios, DV3.7).

### 4.2 The board's outline link (`cadrs_core::pcb`)

```rust
pub struct StudioBoard {
    // … existing fields …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outline_link: Option<OutlineLink>,
}

pub struct OutlineLink {
    pub element: ElementId,          // the Part Studio
    pub sketch: FeatureId,
    /// The sketch regions that make the board: their outer loops are outlines, inner loops
    /// cut-outs. Resolved with `RegionRef::resolve`, so they follow sketch edits.
    pub regions: Vec<RegionRef>,
    /// Sketch circles fully inside a region become: cut-outs (default) or NPTH holes.
    pub circles_as: CircleRole,
    /// Last applied, so refresh only writes when something changed.
    pub applied_hash: u64,
}
```

- The outline shapes it writes are ordinary `BoardShape`s on `Layer::Outline`, with a new
  `BoardShape::linked: bool` (serde default `false`): locked in the layout, shown with a link
  badge; the outline tools refuse to edit them ("Driven by Sketch 3 in Part Studio 1 — edit the
  sketch or Break link").
- Mapping: **board coordinates = sketch-plane coordinates** (mm → nm, both Y up / +v up).
  Lines → `Geom::Line`, arcs (centre/start/end) → `Geom::Arc` (start/mid/end), circles →
  `Geom::Circle`, ellipses and splines → `Geom::Polyline` within 1 µm (or `Bezier` for cubic
  spline spans). Construction geometry is ignored.
- `pcb::outline_link::refresh(doc)` runs in the same post-command hook as `derived::refresh`,
  **before** it (sketch → board → derived copies), so one command updates everything.
- Because board coordinates are sketch coordinates, a Derived board located on that sketch's
  plane sits exactly on the sketch: the outline and the 3D board coincide without any offset.

### 4.3 New Derived location: a sketch plane

A `ConnectorRef` variant (or an implicit connector) for "the plane and origin of sketch S".
"Create board from sketch" uses it so the board stays glued to the sketch when its plane
moves (e.g. the sketch is on an offset plane driven by a variable).

### 4.4 Cache keys

`rebuild::chain_key` hashes `{:?}` of the feature kind. A board design (zones with fills,
hundreds of tracks) would make that slow and huge. Give `DerivedBoard` a manual `Debug` that
prints `board`, `design_hash` and `options` only. `design_hash` = stable hash of the
serialised design (computed in `SetDesign::apply` and in refresh).

### 4.5 Assemblies (phase 6)

`InstanceSource::Board { element, board }`: a rigid instance of a board (board + components
as one, like `InstanceSource::Studio`), built by the same core builder. BOM rows: the board,
and optionally its components (from the layout's BOM, `cadrs_eda::bom`).

## 5. Building the board in a rebuild (`rebuild/kernel_ops/board.rs`)

Moved from `cadrs_pcb::step` and extended:

- **Board body**: outline loops (`pcb::design::outline_loops`) extruded by the stackup
  thickness, less cut-outs and plated/NPTH holes (option: drill holes ≥ 0.8 mm only, to keep
  face counts sane). One part, "Board [<board>]", PCB green appearance.
- **Components (display meshes, Q1)**: per footprint with a visible model, its meshes exactly
  as the PCB Studio's 3D view makes them (`cadrs_pcb::mesh::{file_bodies, generated_bodies}`,
  moved down to core: STEP tessellated, VRML/OBJ as is, else the generated `model3d` body).
  One part per footprint, named "`<ref> <value>`" (e.g. "J1 AC inlet"), a `Solid` with no
  kernel body; its triangles grouped into **faces** by model shape and colour, and each group
  split further into planar patches (coplanar triangles within 0.01°/1 µm) so a planar patch
  gets a `PlaneFrame` (sketchable, implicit mate connector) and a circular-looking patch gets
  a fitted axis (cylinder fit within 5 µm; implicit connector on its axis). Per-face colours
  from the model. Placed with `placement_motion` (board frame: bottom face z = 0) then the
  Derived location motion.
- **What mesh parts can't do** (shown as a tooltip, not an error): booleans, fillets and other
  modelling features skip them with a message; mass properties use the closed mesh volume when
  it is closed, else are left out; STEP export of the Part Studio writes them as faceted
  shells (or, better, re-runs the `cadrs_pcb::step` solid path for export only).
- **Caching** (decision 8): `Rebuilder` keeps component meshes keyed by
  (blob hash, model offset/rotation/scale) and the board body keyed by (outline + holes hash,
  thickness). Moving a footprint then costs one mesh transform for that part. Parts whose
  footprint didn't change are reused from the previous output (diff by footprint `Uuid` +
  placement).
- **Skins**: generalise `SolidImage` to "a texture on a planar face region" (face name + UV
  frame) and attach the top/bottom `board_surface` renders (existing `skins()` code, moved to a
  non-app helper that produces RGBA + UV frame). Rendered at a resolution that follows board
  size, capped (e.g. 4k px), regenerated only when the copper/silk/mask hash changes, not when
  footprints move (pads are on the skin, models are parts).
- **Ids** (decision 5): faces `FaceOrigin::Imported { body: shape index, face: patch index }` under
  `derived_entity(feature, footprint id, copy)`, so per-component naming is independent of
  other components and of order.
- **DerivedOutput** lists the board part, the component parts (grouped under "Components" in
  the Feature list children) and their footprint ids, which the app uses to map a picked part
  back to a footprint.

## 6. Board view toggle: 3D model ↔ 2D layout

Per Derived board (and per copy), like a sketch and its extrude:

- **Feature list**: the Derived row's children show "Board", "Components (n)", and a small
  **3D | Layout** segmented toggle (also in the dialog as `BoardOptions::view`, and the right-
  click menu "Show layout" / "Show 3D model"). It is a document setting (saved, undoable),
  because drawings and thumbnails should show what was chosen; the eye toggles stay per part.
- **Layout view in 3D** (no camera change): the board part is kept (so it can be selected,
  mated, measured) but drawn with its skin off; the component parts are hidden; the layout's
  `DrawList` (`cadrs_eda::render`, the same colours as the PCB Studio's Layout view) is drawn
  **on the board's top face plane** (and the bottom layers on the bottom face when viewed from
  below): triangles as a mesh with a small depth bias, lines as gizmos in world space. Only the
  visible layers from the layers panel state.
- **Ghost mode** (useful while fitting): 3D models drawn translucent over the layout. This is
  the default look of the edit mode (section 7).

Implementation: a `BoardOverlay` system in the app keyed by (Part Studio, Derived feature,
copy), using the Derived copy's world frame (from the rebuild output) to transform the
`DrawList` points (board nm → board frame mm → world). Rebuilt only when the design hash or
the frame changes; reuses `eda::rebuild_scene`'s triangle/line split.

## 7. Edit layout in the Part Studio (the core of the request)

Like editing a sketch: **double-click the board** (or right-click → **Edit layout**, or the
Derived row's pencil) enters **board edit mode** for that copy. Esc / ✓ leaves.

### 7.1 What the user sees

- The rest of the Part Studio stays in 3D (enclosure, plug, end plate), optionally translucent
  (the in-context opacity slider, reused).
- The board shows its layout on its plane plus the components' 3D models (ghosted by default,
  a toggle for solid).
- The **full layout tool strip** of the PCB Studio appears at the top (Q2: everything —
  select/move, rotate, flip, route, vias, zones + fill, lines/rects/circles/text/graphics on
  any layer, outline, keep-outs, ruler, DRC), with the layers panel, net highlight and the
  same keys. Copper, silk and zones are drawn on the board plane while editing; the board
  skin texture is regenerated when leaving the tool (or after an idle moment), not per stroke.
- **N** looks normal to the board (top), **Shift+N** from the bottom; the camera keeps
  orbiting otherwise. Drawing in perspective works because the pointer is mapped through a
  ray-plane intersection (the same projective mapping sketches use since `b2a916d`).
- A breadcrumb bar "Editing layout of *Board 1* (PCB Studio 1)" with **Open in PCB Studio**,
  **Done**.

### 7.2 How it works

- **`LayoutHost`** (resource): `{ pcb: ElementId, board: BoardId, host: Host }` where `Host` is
  `PcbStudio2d` (today) or `Plane3d { studio: ElementId, feature: FeatureId, copy: usize,
  frame: PlaneFrame }`. `Eda2d::board()` becomes `LayoutHost`-backed; `layout_tools` reads
  the board and commits through the host (always `SetDesign` on `pcb`/`board`).
- **Pointer**: in `Plane3d`, a new `navigate3d` system turns viewport pointer events into the
  same `EdaClick { Press / Release / Double }` and `EdaPointer.at` by intersecting the camera
  ray with the board's top plane (or bottom, when the bottom side is active), then mapping to
  board nm. Wheel/orbit/pan stay the 3D camera's. Pick tolerance = N px converted at the
  intersection depth.
- **Picking**: a click first tries the layout hit test (`board_edit::hit` at the plane point);
  if nothing, it tries the component parts' triangles (so clicking a tall model's top picks
  its footprint), mapped back to the footprint via `DerivedOutput`.
- **Live 3D while moving**: during `Moving`, the moved footprint's part meshes (and its
  layout drawing) are re-transformed every frame by `board frame · Δ(placement) · board
  frame⁻¹`; nothing else is rebuilt. On release `SetDesign` commits; refresh + rebuild then
  replace the preview with the real part (same id, so selection and highlights persist).
- **Rotate (R), flip (F)** preview the same way; flip mirrors the model to the other face as
  `placement_motion` does for bottom parts.

### 7.3 Fitting aids (what makes "line up" and "fit exactly" easy)

1. **Context on the board plane**: the other parts of the Part Studio, sectioned by the slab
   the board's components occupy (bottom face − max bottom height … top face + max top height)
   and projected onto the board plane, drawn as cyan reference outlines in the layout (as
   Use/project geometry looks in a sketch). Optional fill for "solid here".
2. **Snapping**: while moving, the footprint's courtyard, pads, model bounding box and model
   silhouette snap to those reference outlines, to their midpoints and to other footprints
   (same inference style as sketches; hold Alt to disable).
3. **Live gap readouts**: for the moving component, the free distance to the nearest part
   along ±X/±Y of the board and above it (Z), as dimension labels; red when < the clearance
   setting. Computed mesh-to-mesh (component triangles against the other parts' display
   triangles, BVH) per frame and again on release; the precision is the tessellation's
   (chord tolerance shown in the readout's tooltip; refine the other parts' tessellation
   locally near the component when the gap is under 1 mm).
4. **Collision highlight**: a component whose model's triangles intersect another part's turn
   red during the drag; on release the result is cached and shown as a warning on the Derived
   feature ("J1 intersects End plate by ≈0.4 mm"). Mesh-based (Q1), so no kernel booleans.
5. **Align footprint** tool: pick an entity of the component (a planar patch, a fitted
   cylinder axis, an edge of its mesh, a mate connector on it), then a target in the Part
   Studio (exact face, edge, axis or mate connector). The footprint moves **in the board plane only** (and optionally turns by the
   in-plane angle) so the two coincide in the plane, with an offset field. This is the
   "connector to plug" operation in one step. Out-of-plane mismatch is reported (it is the
   board's placement that must change).
6. **Exact values**: the existing footprint properties dialog (E: X/Y, angle, side) works in
   edit mode; plus **Move by…** (dx, dy) and typed values in the drag label (Tab to type).
7. **Keep-outs from parts**: right-click a reference outline → "Create keep-out" (layout
   keep-out on chosen layers, height = the part's distance from the board), so the PCB designer
   working later in the PCB Studio sees the mechanical limits; the 3D DRC (8) checks them.
8. **3D clearance in DRC**: new `cadrs_eda::drc` rule input "mechanical context" (projected
   outlines + heights) so the PCB Studio's DRC flags component-to-enclosure violations after
   the fact, with the Part Studio named as the source.

## 8. Sketch → board outline

### 8.1 Create a board from a sketch (Part Studio)

- Select one or more closed sketch regions (or a sketch in the Feature list) → right-click →
  **Create PCB board…**. Dialog: PCB Studio (existing, or "New PCB Studio"), board name,
  thickness / stack-up preset (default 1.6 mm, 2 layers), seat (bottom face on the sketch
  plane, or top), circles as cut-outs / NPTH holes.
- One command (`Scope::Whole`, one undo step): adds the native board (`AddBoard` with the
  outline from the regions and `outline_link` set), and inserts a **Derived** board feature in
  the Part Studio after the sketch, located on the sketch plane (4.3), view = 3D.
- The sketch is hidden afterwards (as an extrude hides its sketch); the board is shown.

### 8.2 Link an existing board's outline

- From the Part Studio: right-click a Derived board → **Link outline to sketch…** → pick
  regions. From the PCB Studio layout: **Outline ▸ Link to Part Studio sketch…** (pick tab,
  sketch, regions).
- What happens to the old outline: replaced by the linked one (old shapes deleted in the same
  step); footprints keep their coordinates. If the new outline is far from the old, offer
  "keep components where they are relative to the outline's lower-left corner".
- **Break link** keeps the current shapes as ordinary, editable outline shapes.

### 8.3 Live or one-shot

Live (refreshed after every command, like a workspace Derived reference) is recommended,
because it is what makes "the board follows the enclosure" work. A link to a sketch whose
regions no longer resolve shows an error on the board (PCB Studio Boards list and the Derived
feature) and leaves the last good outline.

## 9. Assemblies

- **Insert ▸ PCB boards** tab in the Insert dialog lists this document's boards (and other
  documents' at versions) → `InstanceSource::Board` (4.5), rigid, mateable through its
  component faces (e.g. mate J1's face to the plug).
- **Edit layout in context**: the same edit mode (section 7) with the assembly as the 3D
  context: the host frame is the instance's occurrence transform; the context outlines come
  from the other instances. Commits `SetDesign`; the instance follows.
- The Part Studio route (Derived, then insert the Part Studio) also works, and is the one to
  use when the board must interact with modelling features (cut the end plate around J1).

## 10. Phases

Each phase ends with unit tests (no Bevy, `cargo test -r`), a headless scenario and a golden
where it is visual. The acceptance model is the user's screenshot: a stand-in enclosure sample
(`samples::pcb_enclosure`: extrusion, end plate with an inlet cut-out, a plug part) plus a
native board with J1 (inlet connector), a large capacitor and a few SMD parts.

**P0 — groundwork (no user-visible change)**
- Move the board builder into `cadrs_core::rebuild::kernel_ops::board` (board solid from
  `cadrs_pcb::step`, component meshes from `cadrs_pcb::mesh::{file_bodies, generated_bodies}`
  and the VRML/OBJ/STEP model mesh loading from `cadrs_app::pcb::view::PcbMeshCache`);
  `cadrs_pcb::step` keeps its solid-model export on top. Test: STEP export unchanged.
- `Design` hash (stable, serialised) and `Board::origin`.
- `LayoutHost` refactor of `eda::{mod, layout_tools, layers_panel}`; the PCB Studio keeps
  working through `Host::PcbStudio2d`. All existing EDA scenarios/goldens must still pass.
- Move `pcb::view::skins` texture generation into a non-app helper (RGBA + UV frame).

**P1 — Derived board in a Part Studio (read-only 3D)**
- `DerivedBoard`, refresh, check, Debug/hash; source picker lists PCB Studio boards
  ("Boards" section under Current document; versions and other documents through the existing
  browser). Board options in the dialog (components filter, skins, seat).
- Rebuild: board solid + component mesh parts (planar patches and fitted axes as faces) with
  stable ids, caching, warnings; modelling features skip mesh parts with a message.
- Feature list children (Board, Components).
- Tests: ids stable across footprint move/rotate/re-annotate; a mate connector on a planar
  patch of J1's mesh survives a layout move; patch/axis detection on a generated chip and pin
  header body (known planes and pin axes); the refresh follows a `SetDesign` in the PCB Studio; version refs
  frozen; cycle refusals. Scenario `pcb_derived_board`.

**P2 — 3D ↔ Layout toggle**
- `BoardOptions::view`, Feature list toggle, overlay renderer on the board plane, layer
  visibility. Golden `pcb_derived_board_layout` (same camera, both views).

**P3 — Edit layout in the Part Studio**
- Edit mode entry/exit, `navigate3d`, the full layout tool strip and layers panel in the Part
  Studio (route, vias, zones, drawing, text, outline, keep-outs, DRC), hit testing
  (layout first, then model triangles), live drag preview of component meshes, rotate/flip,
  properties dialog, N/Shift+N. Commit via `SetDesign`; undo from the Part Studio.
- Scenario `pcb_edit_in_part_studio`: move J1 with the mouse in a perspective view, check its
  new board coordinates and that its part kept its id; undo restores both; route one track and
  fill a zone on the board plane, check the design.

**P4 — Fitting aids**
- Projected context outlines, snapping, mesh-to-mesh gap readouts and collision highlight
  (BVH), **Align footprint** tool, Move by…, keep-outs from parts, mechanical-context DRC.
- Tests (pure core): slab section/projection of a box at known positions; align solves the
  right in-plane translation/rotation for face–face, axis–axis and connector–connector picks
  (component side from mesh patches, target side exact); mesh gap of two boxes within the
  tessellation tolerance.
- Scenario `pcb_fit_connector`: align J1 to the plug, then fit the large capacitor between the
  wall and J1 with 0.5 mm gaps (readouts asserted).

**P5 — Sketch-driven outline**
- `OutlineLink`, curve conversion, refresh ordering, locked linked shapes, Break link;
  Create PCB board… from regions (one undo step: board + Derived on the sketch plane); Link
  outline to sketch from both sides.
- Tests: rectangle with rounded corners and a round hole → exact outline loops; editing the
  sketch dimension updates the board outline and the derived part in one command; a cycle
  (sketch Uses the board's edge) is refused.
- Scenario `pcb_board_from_sketch` (sketch on an offset plane inside the enclosure using the
  enclosure's inner edges with an offset).

**P6 — Assemblies**
- `InstanceSource::Board`, Insert ▸ PCB boards, BOM rows, Edit layout in context.
- Scenario `pcb_board_in_assembly` (mate J1 to the plug, then nudge a component in context).

**P7 — Rounding off**
- Drawings: a Derived board in a drawing view shows the 3D or the layout as chosen.
- MCP tools (`cadrs_mcp`): `derive_board`, `create_board_from_sketch`, `move_footprint`
  (board coords or align-to target), `get_board` — so the agent workflow can do the fitting.
- STEP export of a Part Studio writes the board solid and, for the component mesh parts,
  re-runs the `cadrs_pcb::step` solid path (real STEP models), not faceted shells.
- IDF boards (non-native) derive as board + package boxes, read-only, no layout view.

## 11. Risks and how the plan handles them

- **Heavy STEP models**: Q1 (display meshes) removes the kernel cost from rebuilds; the
  one-off tessellation is cached per blob and done off the main thread (the previous output
  stays shown meanwhile).
- **Mesh parts are second-class** (Q1): no booleans, no exact interference, gaps only as
  precise as the tessellation. Mitigations: planar patches and fitted axes make mating and
  aligning exact enough; local tessellation refinement near tight gaps; the STEP-export path
  still has the real solids if an exact check is ever needed (a later "Check interference
  exactly" command could build them on demand).
- **Rebuild cost per layout edit**: diff by footprint id; only moved components re-transform;
  skins don't change on moves; board body only on outline/hole change.
- **Embedded design size** in the document file: strip the schematic; consider storing the
  zone fills not at all (re-fill is deterministic) if size becomes a problem.
- **Two editors of one board** (PCB Studio tab and a Part Studio edit mode): both commit
  `SetDesign` against the same element; the edit mode reads the live design each frame, so
  there is no stale copy. Leaving edit mode is not required to switch tabs.
- **Coordinate conventions**: board nm Y-up ↔ sketch mm +v ↔ board frame (bottom face z=0) ↔
  world; bottom-side mirroring (IDF vs cadrs, see `design::pcb_board`). One module
  (`pcb::frames`) owns every conversion, with round-trip tests.
- **Undo scope**: commands that touch the PCB Studio and the Part Studio together (Create PCB
  board…, Link outline) use `Scope::Whole`; plain layout moves stay `Scope::Element(pcb)`.

## 12. Open questions

All five were answered on 2026-10-11 (section 3.1). New questions go here.
