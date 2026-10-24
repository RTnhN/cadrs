//! Tests of the strategies' shared settings: patterns, levels and repeats.

use super::*;
use crate::machine::MachineKind;

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
