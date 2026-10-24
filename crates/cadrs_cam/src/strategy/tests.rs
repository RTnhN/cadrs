//! Tests of the strategies' shared settings: patterns, levels and repeats.

use super::*;
use crate::geom::P3;
use crate::machine::MachineKind;
use crate::toolpath::FeedKind;

#[test]
fn patterns_by_type_and_tool() {
    let mill = Pattern::available(Type::Edges, ToolKind::EndMill, MachineKind::Mill);
    assert!(mill.contains(&Pattern::Profile2D));
    assert!(!mill.contains(&Pattern::Cut));
    let laser = Pattern::available(Type::Edges, ToolKind::Laser, MachineKind::Laser);
    assert_eq!(laser, vec![Pattern::Cut, Pattern::Etch]);
    assert!(
        Pattern::available(Type::Holes, ToolKind::Drill, MachineKind::Mill)
            .contains(&Pattern::Drill)
    );
    assert!(Pattern::available(Type::Edges, ToolKind::EndMill, MachineKind::Laser).is_empty());
    // Every pattern has a tool and machine it runs with.
    for p in Pattern::ALL {
        assert!(
            ToolKind::ALL
                .iter()
                .any(|t| MachineKind::ALL.iter().any(|m| p.allowed(*t, *m))),
            "{p:?}"
        );
    }
}

#[test]
fn levels_end_at_bottom_with_floors() {
    let m = testing::pocket_block();
    let stock = testing::stock_for(&m, 5.0, 1.0);
    let tool = Tool::new(ToolKind::EndMill, 6.0);
    let mut p = Params::new(Pattern::Rough2D, &tool, MachineKind::Mill);
    p.step_down = 5.0;
    p.floor_stock = 0.0;
    let input = Input {
        model: &m,
        stock: &stock,
        tool: &tool,
        machine: MachineKind::Mill,
        params: &p,
        rest: None,
        fixtures: &[],
    };
    let l = input.levels(21.0, 0.0, true);
    assert_eq!(*l.last().unwrap(), 0.0);
    assert!(l.iter().any(|z| (z - 12.0).abs() < 1e-9), "{l:?}");
    assert!(l.windows(2).all(|w| w[0] > w[1]));
}

#[test]
fn repeats_place_copies() {
    let tp = Toolpath {
        steps: vec![
            Step::Spindle {
                rpm: 1000.0,
                cw: true,
            },
            Step::Rapid([10.0, 0.0, 5.0]),
            Step::Arc {
                to: [10.0, 0.0, -1.0],
                center: [5.0, 0.0],
                cw: false,
                feed: 100.0,
                kind: crate::toolpath::FeedKind::Cut,
            },
        ],
        warnings: vec![],
    };
    let lin = Repeat::Linear {
        count: 3,
        step: [20.0, 0.0],
    }
    .apply(tp.clone());
    assert_eq!(lin.steps.len(), 1 + 2 * 3, "the spindle once");
    assert!(matches!(lin.steps[5], Step::Rapid(p) if (p[0] - 50.0).abs() < 1e-9));
    let circ = Repeat::Circular {
        count: 4,
        center: [0.0, 0.0],
        angle: 90.0,
    }
    .apply(tp);
    // The second copy turned 90°: (10, 0) to (0, 10), its arc's centre with it.
    assert!(
        matches!(circ.steps[3], Step::Rapid(p) if p[0].abs() < 1e-9 && (p[1] - 10.0).abs() < 1e-9)
    );
    assert!(
        matches!(circ.steps[4], Step::Arc { center, .. } if center[0].abs() < 1e-9 && (center[1] - 5.0).abs() < 1e-9)
    );
}

/// A profile along two picked bottom edges of a block (an L, open): the tool runs a radius off
/// them, outside the block, on whichever side the material isn't, and cuts nothing of it.
/// Inside with nothing picked means the part's through holes; a block has none.
#[test]
fn a_profile_on_open_edges_keeps_off_the_part() {
    use crate::model::{Edge, Model, box_triangles};
    use crate::sim::{SimOp, simulate_mill_on};
    let mut m = Model::from_triangles(
        "block",
        box_triangles(&Aabb::new([0.0, 0.0, 0.0], [60.0, 40.0, 12.0])),
    );
    for (name, a, b) in [
        ("e1", [0.0, 0.0, 0.0], [60.0, 0.0, 0.0]),
        ("e2", [60.0, 0.0, 0.0], [60.0, 40.0, 0.0]),
    ] {
        m.edges.push(Edge {
            name: name.into(),
            body: 0,
            points: vec![a, b],
            circle: None,
        });
    }
    let tool = Tool::new(ToolKind::EndMill, 6.0);
    let stock = Stock::block(Aabb::new([-5.0, -5.0, 0.0], [65.0, 45.0, 12.0]));
    for (side, picked) in [
        (Side::Auto, true),
        (Side::Outside, true),
        (Side::Auto, false),
    ] {
        let mut p = Params::new(Pattern::Profile2D, &tool, MachineKind::Mill);
        p.side = side;
        // (Both orders of the picks: the side comes from the material, not the order.)
        if picked {
            p.drive.edges = vec!["e2".into(), "e1".into()];
        }
        let input = Input {
            model: &m,
            stock: &stock,
            tool: &tool,
            machine: MachineKind::Mill,
            params: &p,
            rest: None,
            fixtures: &[],
        };
        let tp = generate(&input).unwrap();
        let cuts: Vec<P3> = tp
            .steps
            .iter()
            .filter_map(|s| match s {
                Step::Feed {
                    to,
                    kind: FeedKind::Cut,
                    ..
                } => Some(*to),
                _ => None,
            })
            .collect();
        assert!(!cuts.is_empty());
        if picked {
            // Along the bottom edge a radius below it, along the right one a radius right.
            assert!(cuts.iter().any(|q| (q[1] + 3.0).abs() < 0.05), "{side:?}");
            assert!(cuts.iter().any(|q| (q[0] - 63.0).abs() < 0.05), "{side:?}");
        }
        let segs = tp.motion([0.0, 0.0, 60.0], 5_000.0);
        let r = simulate_mill_on(
            stock.height_map(0.25),
            &m,
            &[],
            &[SimOp {
                tool: &tool,
                segments: &segs,
            }],
        );
        assert_eq!(r.gouge_cells, 0, "{side:?} {picked}: {}", r.gouge_depth);
    }
    let mut p = Params::new(Pattern::Profile2D, &tool, MachineKind::Mill);
    p.side = Side::Inside;
    let input = Input {
        model: &m,
        stock: &stock,
        tool: &tool,
        machine: MachineKind::Mill,
        params: &p,
        rest: None,
        fixtures: &[],
    };
    let e = generate(&input).unwrap_err();
    assert!(e.contains("no through holes"), "{e}");
}

/// A laser cut along a picked open edge: the beam runs half its kerf off it, outside the part.
#[test]
fn a_cut_on_an_open_edge_keeps_its_kerf_off_the_part() {
    use crate::model::{Edge, Model, box_triangles};
    let mut m = Model::from_triangles(
        "plate",
        box_triangles(&Aabb::new([0.0, 0.0, 0.0], [60.0, 40.0, 3.0])),
    );
    m.edges.push(Edge {
        name: "e1".into(),
        body: 0,
        points: vec![[60.0, 0.0, 3.0], [0.0, 0.0, 3.0]],
        circle: None,
    });
    let laser = Tool::new(ToolKind::Laser, 0.2);
    let stock = Stock::block(Aabb::new([-5.0, -5.0, 0.0], [65.0, 45.0, 3.0]));
    let mut p = Params::new(Pattern::Cut, &laser, MachineKind::Laser);
    p.drive.edges = vec!["e1".into()];
    let input = Input {
        model: &m,
        stock: &stock,
        tool: &laser,
        machine: MachineKind::Laser,
        params: &p,
        rest: None,
        fixtures: &[],
    };
    let tp = generate(&input).unwrap();
    let ys: Vec<f64> = tp
        .steps
        .iter()
        .filter_map(|s| match s {
            Step::Feed {
                to,
                kind: FeedKind::Cut,
                ..
            } => Some(to[1]),
            _ => None,
        })
        .collect();
    assert!(!ys.is_empty());
    for y in ys {
        assert!((y + 0.1).abs() < 1e-6, "{y}");
    }
}
