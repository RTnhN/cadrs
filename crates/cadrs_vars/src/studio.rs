//! A Variable Studio's table: rows of variables, each defined by an expression that may name
//! the rows above it and the variables of the tables inserted into the studio.
//!
//! [`evaluate`] works the table out top to bottom, the way Onshape's Variable Studio does:
//!
//! - A row whose name is empty or not a name, or whose value is empty, is in error.
//! - A name is defined once per table: a second row of the same name is in error ("already
//!   defined above"), and the first one keeps the name.
//! - A row may name only what is defined above it (the inserted tables count as above every
//!   row). Naming a row below it is "used before it is defined"; naming itself, or a row that
//!   names it back, is a cycle; anything else is "not defined".
//! - A row in error defines nothing: the rows that name it fail too ("#a is in error").
//!
//! The rows' order is the user's (Move up, Move down, drag), see [`move_row`].

use cadrs_sketch::units::{self, Units, VarValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::variable::{VariableFeature, VariableType, eval_variable};

/// One row of a Variable Studio: a variable and the id it keeps through renames and moves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudioVariable {
    pub id: Uuid,
    pub var: VariableFeature,
}

impl StudioVariable {
    /// A new row with a fresh id.
    pub fn new(var: VariableFeature) -> Self {
        Self {
            id: Uuid::new_v4(),
            var,
        }
    }
}

/// How a row worked out.
#[derive(Debug, Clone, PartialEq)]
pub struct RowEval {
    pub id: Uuid,
    /// Its value, if it defines one.
    pub value: Option<VarValue>,
    /// Why it doesn't.
    pub error: Option<String>,
}

/// A table worked out: each row's result, in order, and the variables it defines (with those
/// of `base` first).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TableEval {
    pub rows: Vec<RowEval>,
    /// The rows that define a value: (row, value), in order.
    pub defined: Vec<(Uuid, VarValue)>,
}

impl TableEval {
    /// The result of a row.
    pub fn row(&self, id: Uuid) -> Option<&RowEval> {
        self.rows.iter().find(|r| r.id == id)
    }

    /// The number of rows in error.
    pub fn errors(&self) -> usize {
        self.rows.iter().filter(|r| r.error.is_some()).count()
    }
}

/// Works the rows out top to bottom (see the module docs). `base` are the variables defined
/// above every row (the inserted tables'); a row's own name shadows one of `base`.
pub fn evaluate(rows: &[StudioVariable], base: &[(String, VarValue)], units: &Units) -> TableEval {
    let mut env: Vec<(String, VarValue)> = base.to_vec();
    let mut out = TableEval::default();
    // Names of the rows seen so far, and of the ones in error.
    let mut seen: Vec<&str> = Vec::new();
    let mut failed: Vec<&str> = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let v = &row.var;
        let error = row_error(rows, i, base, &env, &seen, &failed);
        let result = match error {
            Some(e) => Err(e),
            None => eval_variable(v.var_type, &v.expr, units, &env).map_err(|e| e.to_string()),
        };
        // A duplicate leaves the name to the row above it (in error or not).
        let duplicate = seen.contains(&v.name.as_str());
        if !v.name.is_empty() && !duplicate {
            seen.push(&v.name);
        }
        match result {
            Ok((value, kind)) => {
                let value = VarValue {
                    value,
                    quantity: kind.quantity(),
                };
                env.push((v.name.clone(), value));
                out.defined.push((row.id, value));
                out.rows.push(RowEval {
                    id: row.id,
                    value: Some(value),
                    error: None,
                });
            }
            Err(e) => {
                if !v.name.is_empty() && !duplicate && !failed.contains(&v.name.as_str()) {
                    failed.push(&v.name);
                }
                out.rows.push(RowEval {
                    id: row.id,
                    value: None,
                    error: Some(e),
                });
            }
        }
    }
    out
}

/// The rows with each one's last evaluated value written in (`value` and `kind`), so a saved
/// table shows what it evaluated to. Returns whether anything changed.
pub fn write_values(rows: &mut [StudioVariable], eval: &TableEval) -> bool {
    let mut changed = false;
    for row in rows {
        if let Some(Some(v)) = eval.row(row.id).map(|r| r.value) {
            let kind = crate::VariableKind::of(v.quantity);
            if row.var.value != v.value || row.var.kind != kind {
                row.var.value = v.value;
                row.var.kind = kind;
                changed = true;
            }
        }
    }
    changed
}

/// Moves the row `id` to position `to` (counted without it); false if there is no such row.
pub fn move_row(rows: &mut Vec<StudioVariable>, id: Uuid, to: usize) -> bool {
    let Some(from) = rows.iter().position(|r| r.id == id) else {
        return false;
    };
    let row = rows.remove(from);
    rows.insert(to.min(rows.len()), row);
    true
}

/// A name not yet used by `rows`: `var_1`, `var_2`, …
pub fn fresh_name(rows: &[StudioVariable]) -> String {
    (1..)
        .map(|i| format!("var_{i}"))
        .find(|n| !rows.iter().any(|r| r.var.name == *n))
        .unwrap_or_default()
}

/// A new row as typed in the table's last (empty) row: a Length named `name`, 0 in the
/// workspace unit, as Onshape starts one.
pub fn new_row(name: &str, units: &Units) -> StudioVariable {
    let mut var = VariableFeature {
        name: name.trim().trim_start_matches('#').to_string(),
        var_type: VariableType::Length,
        expr: VariableType::Length.with_unit("0", units),
        ..VariableFeature::default()
    };
    let _ = var.evaluate(units, &[]);
    StudioVariable::new(var)
}

/// Why row `i` can't be worked out before its expression is evaluated: its name, a duplicate,
/// or a name it can't see (`env`: what is defined above it; `seen` and `failed`: the names of
/// the rows above it, and of those in error).
fn row_error(
    rows: &[StudioVariable],
    i: usize,
    base: &[(String, VarValue)],
    env: &[(String, VarValue)],
    seen: &[&str],
    failed: &[&str],
) -> Option<String> {
    let v = &rows[i].var;
    if let Some(p) = v.problem() {
        return Some(p.to_string());
    }
    if seen.contains(&v.name.as_str()) {
        return Some(format!("#{} is already defined above", v.name));
    }
    for n in units::variable_names(&v.expr) {
        if n == v.name {
            // Its own name, unless that is an inserted variable it redefines.
            if base.iter().any(|(k, _)| *k == n) {
                continue;
            }
            return Some(format!("#{n} refers to itself"));
        }
        // Before what is defined: a row in error that redefines an inserted variable hides it.
        if failed.contains(&n.as_str()) {
            return Some(format!("#{n} is in error"));
        }
        if env.iter().any(|(k, _)| *k == n) {
            continue;
        }
        if let Some(j) = rows[i + 1..].iter().position(|r| r.var.name == n) {
            let below = &rows[i + 1 + j].var;
            return Some(if units::variable_names(&below.expr).contains(&v.name) {
                format!("#{n} and #{} refer to each other", v.name)
            } else {
                format!("#{n} is used before it is defined; move it above this variable")
            });
        }
        return Some(format!("#{n} is not defined"));
    }
    None
}

/// The rows' values as expressions see them: (name, value) of each row that defines one.
pub fn env(rows: &[StudioVariable], eval: &TableEval) -> Vec<(String, VarValue)> {
    eval.defined
        .iter()
        .filter_map(|(id, v)| {
            let row = rows.iter().find(|r| r.id == *id)?;
            Some((row.var.name.clone(), *v))
        })
        .collect()
}
