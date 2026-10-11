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
4. **Board geometry becomes real parts** (kernel bodies, named faces), so everything that works
   on parts works on the board and its components: mates, mate connectors, Measure, sketch on a
   face, Use edges, interference, section view, STEP export, drawings, BOM.
5. **Stable ids from the layout's ids.** The board part's id comes from (Derived feature,
   "board"); each component part's from (Derived feature, footprint `Uuid`, copy). Moving,
   rotating or re-annotating a footprint keeps its part id and face names, so a mate to J1's
   face, or a sketch that Uses an edge of the board, survives layout edits.
6. **Move the board → B-rep builder into `cadrs_core`** (`rebuild/kernel_ops/board.rs`), with
   `cadrs_pcb::step` calling it for STEP export. `cadrs_core` already depends on `cadrs_eda`
   and the kernel, so no new dependency.
7. **The layout tools become host-agnostic.** They already work in board coordinates through
   `EdaClick { at: Pt }`; we lift "where am I and where do I commit" into a `LayoutHost`
   (2D PCB Studio view or a board plane in the 3D viewport) instead of copying them.
8. **Dragging never touches the kernel.** During a drag the component's display meshes are
   moved by the delta transform; the kernel rebuild happens once, on release, and is cheap
   because model bodies are cached per model file + model transform.

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
- **Components**: per footprint with a visible model: the STEP file as kernel bodies, VRML/OBJ
  as sewn solids (as Import does for STL), else the generated `model3d` body. One part per
  footprint, named "`<ref> <value>`" (e.g. "J1 AC inlet"), with per-face appearances from the
  model's colours. Placed with `placement_motion` (board frame: bottom face z = 0) then the
  Derived location motion.
- **Mesh-only fallback**: a model that won't sew becomes a part with a `Solid` but no kernel
  body (`PartState::body = None`, already allowed): visible, pickable, measurable by
  triangles, excluded from booleans; flagged in the feature's warnings.
- **Caching** (decision 8): `Rebuilder` keeps `model bodies` keyed by
  (blob hash, model offset/rotation/scale) and the board body keyed by (outline + holes hash,
  thickness). Moving a footprint then costs one `transform_motion` + `solid_of` for that part.
  Parts whose footprint didn't change are reused from the previous output (diff by footprint
  `Uuid` + placement).
- **Skins**: generalise `SolidImage` to "a texture on a planar face region" (face name + UV
  frame) and attach the top/bottom `board_surface` renders (existing `skins()` code, moved to a
  non-app helper that produces RGBA + UV frame). Rendered at a resolution that follows board
  size, capped (e.g. 4k px), regenerated only when the copper/silk/mask hash changes, not when
  footprints move (pads are on the skin, models are parts).
- **Ids** (decision 5): faces `FaceOrigin::Imported { body, face }` under
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
- The **layout tool strip** of the PCB Studio appears at the top (select/move, rotate, flip,
  outline, keep-out, ruler; routing and zones too — they work, but the plan's focus is
  placement), with the layers panel.
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
   setting. Computed on display meshes (BVH) per frame; exact (kernel distance) on release.
4. **Collision highlight**: a component whose model intersects another part turns red during
   the drag; on release, an exact interference check (kernel boolean/distance) is cached and
   shown as a warning on the Derived feature ("J1 intersects End plate by 0.4 mm").
5. **Align footprint** tool: pick an entity of the component (face, edge, cylinder axis, mate
   connector of its model), then a target in the Part Studio (face, edge, axis, mate
   connector). The footprint moves **in the board plane only** (and optionally turns by the
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
- Move the board → kernel builder from `cadrs_pcb::step` into `cadrs_core::rebuild::kernel_ops::board`;
  `cadrs_pcb::step` calls it. Test: STEP export unchanged (existing tests).
- `Design` hash (stable, serialised) and `Board::origin`.
- `LayoutHost` refactor of `eda::{mod, layout_tools, layers_panel}`; the PCB Studio keeps
  working through `Host::PcbStudio2d`. All existing EDA scenarios/goldens must still pass.
- Move `pcb::view::skins` texture generation into a non-app helper (RGBA + UV frame).

**P1 — Derived board in a Part Studio (read-only 3D)**
- `DerivedBoard`, refresh, check, Debug/hash; source picker lists PCB Studio boards
  ("Boards" section under Current document; versions and other documents through the existing
  browser). Board options in the dialog (components filter, skins, seat).
- Rebuild: board + component parts with stable ids, caching, mesh-only fallback, warnings.
- Feature list children (Board, Components).
- Tests: ids stable across footprint move/rotate/re-annotate; a mate connector on J1's face
  survives a layout move; the refresh follows a `SetDesign` in the PCB Studio; version refs
  frozen; cycle refusals. Scenario `pcb_derived_board`.

**P2 — 3D ↔ Layout toggle**
- `BoardOptions::view`, Feature list toggle, overlay renderer on the board plane, layer
  visibility. Golden `pcb_derived_board_layout` (same camera, both views).

**P3 — Edit layout in the Part Studio**
- Edit mode entry/exit, `navigate3d`, layout tool strip in the Part Studio, hit testing
  (layout first, then model triangles), live drag preview of component meshes, rotate/flip,
  properties dialog, N/Shift+N. Commit via `SetDesign`; undo from the Part Studio.
- Scenario `pcb_edit_in_part_studio`: move J1 with the mouse in a perspective view, check its
  new board coordinates and that its part kept its id; undo restores both.

**P4 — Fitting aids**
- Projected context outlines, snapping, gap readouts, collision highlight + exact check on
  release, **Align footprint** tool, Move by…, keep-outs from parts, mechanical-context DRC.
- Tests (pure core): slab section/projection of a box at known positions; align solves the
  right in-plane translation/rotation for face–face, axis–axis and connector–connector picks.
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
- STEP export of a Part Studio includes the board parts (free once they are parts).
- IDF boards (non-native) derive as board + package boxes, read-only, no layout view.

## 11. Risks and how the plan handles them

- **Heavy STEP models** (connectors with thousands of faces): cache per blob; option to show
  only the bounding box per component while editing; mesh-only fallback; build component
  bodies on the kernel thread like other rebuilds (the Part Studio stays responsive and shows
  the previous output).
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

## 12. Open questions for you

1. **Component models as kernel solids, or display-only?** Recommended: solids (mates, measure,
   interference, cuts in the end plate all need them), with the mesh-only fallback. Costs build
   time for big STEP files.
2. **Routing/zones in the Part Studio edit mode**: allow everything the PCB Studio layout can
   do, or limit it to placement + outline + keep-outs to keep the strip simple?
3. **Outline link**: live (recommended) or one-shot "Update outline from sketch" button?
4. **Seat default**: board's bottom face on the sketch plane (board grows along +normal), or
   top face on it?
5. **Assembly insert of boards** (P6) directly, or only via a Part Studio with a Derived board?
