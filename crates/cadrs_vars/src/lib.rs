//! Variables without a document (Onshape's `variables-and-expressions` course and the Variable
//! Studio): a variable's type, expression and value ([`VariableFeature`]), and the table of a
//! Variable Studio ([`studio`]).
//!
//! `cadrs_core` puts these in documents: the Variable feature of a Part Studio or Assembly
//! (`cadrs_core::variables`), and the Variable Studio tab whose variables every Part Studio and
//! Assembly of the document can name (`cadrs_core::variable_studio`). The expressions
//! themselves (`#name`, units, functions, `? :`) are `cadrs_sketch::units`'.

pub mod studio;
mod variable;

pub use studio::{RowEval, StudioVariable, TableEval};
pub use variable::{VariableFeature, VariableKind, VariableType, display, eval_variable};

#[cfg(test)]
mod tests;
