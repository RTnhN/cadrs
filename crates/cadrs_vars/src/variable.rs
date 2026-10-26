//! A variable's definition: `#name = expression`, with a type and a description (the Variable
//! feature of a Part Studio or Assembly, and a row of a Variable Studio).

use cadrs_sketch::units::{self, ParseError, Quantity, Units, VarValue};
use serde::{Deserialize, Serialize};

/// A Variable's type (Onshape's Length, Angle, Number and Any).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VariableType {
    #[default]
    Length,
    Angle,
    Number,
    /// Whatever the expression gives.
    Any,
}

impl VariableType {
    pub const ALL: [VariableType; 4] = [
        VariableType::Length,
        VariableType::Angle,
        VariableType::Number,
        VariableType::Any,
    ];

    pub fn label(self) -> &'static str {
        match self {
            VariableType::Length => "Length",
            VariableType::Angle => "Angle",
            VariableType::Number => "Number",
            VariableType::Any => "Any",
        }
    }

    /// A value typed as a bare number, with the type's unit added ("40" → "40 mm" for a
    /// Length in a millimetre workspace, "30" → "30 deg" for an Angle); anything else as typed.
    pub fn with_unit(self, text: &str, units: &Units) -> String {
        let text = text.trim();
        match (text.parse::<f64>().is_ok(), self) {
            (true, VariableType::Length) => format!("{text} {}", units.length.symbol()),
            (true, VariableType::Angle) => format!("{text} deg"),
            _ => text.to_string(),
        }
    }
}

/// What a variable's value measures, once evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VariableKind {
    /// Millimetres.
    #[default]
    Length,
    /// Degrees.
    Angle,
    Number,
}

impl VariableKind {
    pub fn quantity(self) -> Quantity {
        match self {
            VariableKind::Length => Quantity::Length,
            VariableKind::Angle => Quantity::Angle,
            VariableKind::Number => Quantity::Count,
        }
    }

    pub fn of(q: Quantity) -> Self {
        match q {
            Quantity::Length => VariableKind::Length,
            Quantity::Angle => VariableKind::Angle,
            Quantity::Count => VariableKind::Number,
        }
    }

    /// The type that holds exactly this kind of value.
    pub fn var_type(self) -> VariableType {
        match self {
            VariableKind::Length => VariableType::Length,
            VariableKind::Angle => VariableType::Angle,
            VariableKind::Number => VariableType::Number,
        }
    }

    /// An expression that evaluates to exactly `value` (mm, degrees or a number), whatever the
    /// workspace units: `12.5 mm`, `30 deg`, `4`.
    pub fn literal(self, value: f64) -> String {
        // `{:?}` prints the shortest text that reads back as the same f64.
        match self {
            VariableKind::Length => format!("{value:?} mm"),
            VariableKind::Angle => format!("{value:?} deg"),
            VariableKind::Number => format!("{value:?}"),
        }
    }
}

/// A Variable: `#name = expr`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableFeature {
    /// The name, without the `#`.
    pub name: String,
    #[serde(default)]
    pub var_type: VariableType,
    /// The expression as typed ("40 mm", "#piston_d + 0.5 mm").
    pub expr: String,
    /// Its value when last evaluated: mm, degrees or a plain number.
    pub value: f64,
    /// What the value measures.
    #[serde(default)]
    pub kind: VariableKind,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
}

impl Default for VariableFeature {
    fn default() -> Self {
        Self {
            name: String::new(),
            var_type: VariableType::Length,
            expr: "0 mm".into(),
            value: 0.0,
            kind: VariableKind::Length,
            description: String::new(),
        }
    }
}

impl VariableFeature {
    /// A Length variable `#name = expr`, evaluated without other variables.
    pub fn length(name: &str, expr: &str) -> Self {
        Self::typed(VariableType::Length, name, expr)
    }

    /// A variable of type `t`, `#name = expr`, evaluated without other variables.
    pub fn typed(t: VariableType, name: &str, expr: &str) -> Self {
        let mut v = Self {
            name: name.into(),
            var_type: t,
            expr: expr.into(),
            kind: match t {
                VariableType::Angle => VariableKind::Angle,
                VariableType::Number => VariableKind::Number,
                _ => VariableKind::Length,
            },
            ..Self::default()
        };
        let _ = v.evaluate(&Units::default(), &Vec::new());
        v
    }

    /// The value as expressions see it.
    pub fn var_value(&self) -> VarValue {
        VarValue {
            value: self.value,
            quantity: self.kind.quantity(),
        }
    }

    /// Evaluates the expression with the variables above it (`vars`), setting the value.
    pub fn evaluate(
        &mut self,
        units: &Units,
        vars: &[(String, VarValue)],
    ) -> Result<(), ParseError> {
        let (value, kind) = eval_variable(self.var_type, &self.expr, units, vars)?;
        self.value = value;
        self.kind = kind;
        Ok(())
    }

    /// Why it can't be defined, if it can't.
    pub fn problem(&self) -> Option<&'static str> {
        if self.name.is_empty() {
            return Some("Enter a variable name");
        }
        if !units::is_variable_name(&self.name) {
            return Some("A name is letters, digits and _ (starting with a letter)");
        }
        if self.expr.trim().is_empty() {
            return Some("Enter a value");
        }
        None
    }

    /// The value with its unit, as the Variable table shows it ("40.5 mm").
    pub fn display(&self, units: &Units) -> String {
        display(self.value, self.kind, units)
    }
}

/// A value with its unit in the workspace units ("40.5 mm", "30 deg", "4").
pub fn display(value: f64, kind: VariableKind, units: &Units) -> String {
    match kind {
        VariableKind::Length => format!(
            "{} {}",
            units.value(value, Quantity::Length),
            units.length.symbol()
        ),
        VariableKind::Angle => format!("{} deg", units.value(value, Quantity::Angle)),
        VariableKind::Number => {
            let s = format!("{value:.6}");
            let s = s.trim_end_matches('0').trim_end_matches('.');
            if s == "-0" { "0".into() } else { s.into() }
        }
    }
}

/// A variable's expression evaluated as its type asks: its value and what it measures.
pub fn eval_variable(
    t: VariableType,
    expr: &str,
    units: &Units,
    vars: &[(String, VarValue)],
) -> Result<(f64, VariableKind), ParseError> {
    Ok(match t {
        VariableType::Length => (
            units.eval_vars(expr, Quantity::Length, &vars)?,
            VariableKind::Length,
        ),
        VariableType::Angle => (
            units.eval_vars(expr, Quantity::Angle, &vars)?,
            VariableKind::Angle,
        ),
        VariableType::Number => (
            units.eval_vars(expr, Quantity::Count, &vars)?,
            VariableKind::Number,
        ),
        VariableType::Any => {
            let v = units::eval_any(expr, units.length.mm(), &vars)?;
            (v.value, VariableKind::of(v.quantity))
        }
    })
}
