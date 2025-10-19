use crate::models::{DataDictionary, Value};
use crate::evaluator::Facts; // <-- Import the new evaluator
use crate::parser::parse_expression;
use crate::evaluator::evaluate;
use anyhow::{Context, Result, bail};

/// The RulesEngine is now an orchestrator that parses rules on demand.
pub struct RulesEngine {
    dictionary: DataDictionary,
}

impl RulesEngine {
    /// Creates a new RulesEngine.
    pub fn new(dict: DataDictionary) -> Result<Self> {
        Ok(Self { dictionary: dict })
    }

    /// Evaluates a chain of dependencies.
    pub fn evaluate_chain(&self, targets: &[String], initial_facts: &Facts) -> Result<Facts> {
        let mut facts = initial_facts.clone();
        for target in targets {
            self.calculate_attribute_recursive(target, &mut facts)?;
        }
        Ok(facts)
    }

    fn calculate_attribute_recursive(&self, attr_name: &str, facts: &mut Facts) -> Result<()> {
        if facts.contains_key(attr_name) {
            return Ok(()); // Already calculated.
        }

        // Look for derived attribute definition
        let attr_def = self.dictionary.derived_attributes.iter()
            .find(|attr| attr.name == attr_name)
            .with_context(|| format!("Definition for '{}' not found", attr_name))?;

        // Calculate dependencies first
        for dep in &attr_def.dependencies {
            self.calculate_attribute_recursive(dep, facts)?;
        }

        // TODO: Implement proper derived attributes support
        // For now, just insert a placeholder value
        facts.insert(attr_name.to_string(), Value::String("derived_placeholder".to_string()));

        Ok(())
    }
}
