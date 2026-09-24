//! Model type theory. TODO.
mod binary_signature;
mod composite;
mod display_helpers;
mod hole;

mod ast;
pub mod checker;
mod parser;
pub mod theory;

/// Parse and check a model from source text, preserving the checked context and
/// its recorded derivations for downstream consumers.
pub fn check_with_context(input: &str) -> Result<checker::ProgrammeContext, String> {
    let model = parser::parse_model(input).map_err(|e| format!("parse: {e}"))?;
    let programme = ast::Programme { models: vec![model] };
    checker::check_programme(&programme).map_err(|e| format!("check: {e}"))
}

/// Parse and check a model from source text.
pub fn check(input: &str) -> Result<(), String> {
    check_with_context(input).map(|_| ())
}

#[cfg(test)]
mod test_models;
