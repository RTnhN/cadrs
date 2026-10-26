use cadrs_sketch::units::{LengthUnit, Quantity, Units, VarValue};

use crate::studio::{self, StudioVariable};
use crate::{VariableFeature, VariableKind, VariableType};

fn row(t: VariableType, name: &str, expr: &str) -> StudioVariable {
    StudioVariable::new(VariableFeature {
        name: name.into(),
        var_type: t,
        expr: expr.into(),
        ..VariableFeature::default()
    })
}

fn len(name: &str, expr: &str) -> StudioVariable {
    row(VariableType::Length, name, expr)
}

fn value(e: &crate::TableEval, r: &StudioVariable) -> f64 {
    e.row(r.id).and_then(|r| r.value).expect("a value").value
}

fn error(e: &crate::TableEval, r: &StudioVariable) -> String {
    e.row(r.id)
        .and_then(|r| r.error.clone())
        .unwrap_or_default()
}

#[test]
fn a_table_top_from_its_length() {
    // The help page's example: the table top's width from its length.
    let mm = Units::default();
    let l1 = len("L1", "1200 mm");
    let w1 = len("W1", "#L1 / 2");
    let legs = row(VariableType::Number, "legs", "4");
    let splay = row(VariableType::Angle, "splay", "atan2(50 mm, 700 mm)");
    let rows = vec![l1.clone(), w1.clone(), legs.clone(), splay.clone()];
    let e = studio::evaluate(&rows, &[], &mm);
    assert_eq!(e.errors(), 0);
    assert_eq!(value(&e, &w1), 600.0);
    assert_eq!(value(&e, &legs), 4.0);
    assert!((value(&e, &splay) - 4.085_616_7).abs() < 1e-6);
    let env = studio::env(&rows, &e);
    assert_eq!(env.len(), 4);
    assert_eq!(env[1].0, "W1");
    assert_eq!(env[3].1.quantity, Quantity::Angle);
}

#[test]
fn rows_see_only_the_rows_above_them() {
    let mm = Units::default();
    let a = len("a", "#b * 2");
    let b = len("b", "10 mm");
    let c = len("c", "#nowhere + 1 mm");
    let d = len("d", "#c + 1 mm");
    let rows = vec![a.clone(), b.clone(), c.clone(), d.clone()];
    let e = studio::evaluate(&rows, &[], &mm);
    assert_eq!(
        error(&e, &a),
        "#b is used before it is defined; move it above this variable"
    );
    assert_eq!(value(&e, &b), 10.0);
    assert_eq!(error(&e, &c), "#nowhere is not defined");
    // A row in error defines nothing: its users fail too, saying why.
    assert_eq!(error(&e, &d), "#c is in error");
    assert_eq!(e.defined.len(), 1);

    // Moved below its use, `a` works.
    let mut rows = rows;
    assert!(studio::move_row(&mut rows, a.id, 1));
    assert_eq!(rows[1].id, a.id);
    let e = studio::evaluate(&rows, &[], &mm);
    assert_eq!(value(&e, &a), 20.0);
}

#[test]
fn cycles_duplicates_and_bad_names() {
    let mm = Units::default();
    let me = len("me", "#me + 1 mm");
    let x = len("x", "#y");
    let y = len("y", "#x");
    let dup = len("me", "3 mm");
    let bad = len("2fast", "1 mm");
    let empty = len("ok", " ");
    let rows = vec![
        me.clone(),
        x.clone(),
        y.clone(),
        dup.clone(),
        bad.clone(),
        empty.clone(),
    ];
    let e = studio::evaluate(&rows, &[], &mm);
    assert_eq!(error(&e, &me), "#me refers to itself");
    assert_eq!(error(&e, &x), "#y and #x refer to each other");
    assert_eq!(error(&e, &y), "#x is in error");
    assert_eq!(error(&e, &dup), "#me is already defined above");
    assert!(error(&e, &bad).starts_with("A name is"));
    assert_eq!(error(&e, &empty), "Enter a value");
    assert!(e.defined.is_empty());
}

#[test]
fn types_check_what_the_value_measures() {
    let mm = Units::default();
    let l = len("l", "30 deg");
    let a = row(VariableType::Angle, "a", "10 mm");
    let n = row(VariableType::Number, "n", "#ok");
    let ok = len("ok", "5");
    let any = row(VariableType::Any, "any", "#ok * 2");
    let ratio = row(VariableType::Any, "ratio", "#ok / 1 in");
    let rows = vec![
        ok.clone(),
        l.clone(),
        a.clone(),
        n.clone(),
        any.clone(),
        ratio.clone(),
    ];
    let e = studio::evaluate(&rows, &[], &mm);
    assert_eq!(error(&e, &l), "Units do not match");
    assert_eq!(error(&e, &a), "Units do not match");
    assert_eq!(error(&e, &n), "Units do not match");
    // A bare number in a Length is in the workspace unit.
    assert_eq!(value(&e, &ok), 5.0);
    assert_eq!(
        e.row(any.id).unwrap().value.unwrap().quantity,
        Quantity::Length
    );
    assert_eq!(value(&e, &any), 10.0);
    assert_eq!(
        e.row(ratio.id).unwrap().value.unwrap().quantity,
        Quantity::Count
    );
    assert!((value(&e, &ratio) - 5.0 / 25.4).abs() < 1e-12);

    let inches = Units::new(LengthUnit::Inch, 3);
    let e = studio::evaluate(&rows, &[], &inches);
    assert_eq!(value(&e, &ok), 127.0);
}

#[test]
fn inserted_variables_come_first_and_can_be_redefined() {
    let mm = Units::default();
    let base = vec![(
        "width".to_string(),
        VarValue {
            value: 100.0,
            quantity: Quantity::Length,
        },
    )];
    let half = len("half", "#width / 2");
    let width = len("width", "#width + 10 mm");
    let after = len("after", "#width");
    let rows = vec![half.clone(), width.clone(), after.clone()];
    let e = studio::evaluate(&rows, &base, &mm);
    assert_eq!(e.errors(), 0);
    assert_eq!(value(&e, &half), 50.0);
    assert_eq!(value(&e, &width), 110.0);
    assert_eq!(value(&e, &after), 110.0);
}

#[test]
fn conditionals_and_functions_in_a_table() {
    let mm = Units::default();
    let w = len("w", "120 mm");
    let n = row(
        VariableType::Number,
        "holes",
        "#w > 100 mm ? floor(#w / 40 mm) : 2",
    );
    let pitch = len("pitch", "#w / (#holes + 1)");
    let rows = vec![w.clone(), n.clone(), pitch.clone()];
    let e = studio::evaluate(&rows, &[], &mm);
    assert_eq!(value(&e, &n), 3.0);
    assert_eq!(value(&e, &pitch), 30.0);
}

#[test]
fn write_values_keeps_what_each_row_evaluated_to() {
    let mm = Units::default();
    let mut rows = vec![len("a", "2 in"), row(VariableType::Any, "b", "#a / 1 mm")];
    let e = studio::evaluate(&rows, &[], &mm);
    assert!(studio::write_values(&mut rows, &e));
    assert_eq!(rows[0].var.value, 50.8);
    assert_eq!(rows[1].var.kind, VariableKind::Number);
    assert!(!studio::write_values(&mut rows, &e));
}

#[test]
fn new_rows_names_and_units() {
    let inches = Units::new(LengthUnit::Inch, 3);
    let r = studio::new_row("#depth", &inches);
    assert_eq!(r.var.name, "depth");
    assert_eq!(r.var.expr, "0 in");
    assert_eq!(VariableType::Angle.with_unit(" 30 ", &inches), "30 deg");
    assert_eq!(VariableType::Number.with_unit("3", &inches), "3");
    assert_eq!(VariableType::Length.with_unit("#a * 2", &inches), "#a * 2");
    let rows = vec![len("var_1", "1 mm"), len("var_3", "1 mm")];
    assert_eq!(studio::fresh_name(&rows), "var_2");
}

#[test]
fn literals_read_back_exactly() {
    let mm = Units::default();
    let inches = Units::new(LengthUnit::Inch, 3);
    for (kind, v) in [
        (VariableKind::Length, 25.4 / 3.0),
        (VariableKind::Length, -1e-7),
        (VariableKind::Angle, 33.333_333_333_333_336),
        (VariableKind::Number, 7.0),
    ] {
        for u in [&mm, &inches] {
            let (got, k) = crate::eval_variable(kind.var_type(), &kind.literal(v), u, &[]).unwrap();
            assert_eq!(got, v);
            assert_eq!(k, kind);
        }
    }
}

#[test]
fn rows_round_trip_through_ron() {
    let rows = vec![len("a", "2 in"), row(VariableType::Angle, "b", "15 deg")];
    let text = ron::to_string(&rows).unwrap();
    let back: Vec<StudioVariable> = ron::from_str(&text).unwrap();
    assert_eq!(back, rows);
}

#[test]
fn a_row_in_error_hides_the_inserted_variable_it_redefines() {
    let mm = Units::default();
    let base = vec![(
        "a".to_string(),
        VarValue {
            value: 5.0,
            quantity: Quantity::Length,
        },
    )];
    let before = len("before", "#a * 2");
    let a = len("a", "#nowhere");
    let after = len("after", "#a * 2");
    let rows = vec![before.clone(), a.clone(), after.clone()];
    let e = studio::evaluate(&rows, &base, &mm);
    // Above the redefinition, the inserted #a.
    assert_eq!(value(&e, &before), 10.0);
    assert_eq!(error(&e, &a), "#nowhere is not defined");
    // Below it, the redefinition in error: not silently the inserted value.
    assert_eq!(error(&e, &after), "#a is in error");
}

#[test]
fn a_duplicate_leaves_the_name_to_the_first_row() {
    let mm = Units::default();
    let a = len("a", "1 mm");
    let dup = len("a", "2 mm");
    let b = len("b", "#a * 3");
    let rows = vec![a.clone(), dup.clone(), b.clone()];
    let e = studio::evaluate(&rows, &[], &mm);
    assert_eq!(error(&e, &dup), "#a is already defined above");
    // Not "#a is in error": the first #a still defines it.
    assert_eq!(value(&e, &b), 3.0);
}
