use crate::models::{DataDictionary, Value};
use crate::parser;
use crate::evaluator::{self, Facts}; // <-- Import the new evaluator
use anyhow::{bail, Context, Result};

/// The RulesEngine is now an orchestrator that parses rules on demand.
pub struct RulesEngine<'a> {
    dictionary: &'a DataDictionary,
}

impl<'a> RulesEngine<'a> {
    /// Creates a new RulesEngine.
    pub fn new(dict: &'a DataDictionary) -> Result<Self> {
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

        let attr_def = self.dictionary.derived_attributes.iter()
            .find(|a| a.name == attr_name)
            .with_context(|| format!("Definition for '{}' not found", attr_name))?;

        // Recursively ensure all dependencies are met.
        for dep in &attr_def.dependencies {
            self.calculate_attribute_recursive(dep, facts)?;
        }

        // Find the first rule for this attribute
        let rule = attr_def.rules.get(0)
            .with_context(|| format!("No rules found for attribute '{}'", attr_name))?;

        // Parse and evaluate the condition
        let condition_met = if let Some(cond_str) = &rule.condition {
            let cond_ast = parser::parse_rule(cond_str)
                .map_err(|e| anyhow::anyhow!("Failed to parse condition '{}': {:?}", cond_str, e))?
                .1;
            match evaluator::evaluate(&cond_ast, facts)? {
                Value::Boolean(b) => b,
                _ => false,
            }
        } else {
            true // No condition means it's always met.
        };

        // Parse and evaluate the appropriate value expression
        let final_value = if condition_met {
            if let Some(then_str) = &rule.value {
                let then_ast = parser::parse_rule(then_str)
                    .map_err(|e| anyhow::anyhow!("Failed to parse then clause '{}': {:?}", then_str, e))?
                    .1;
                evaluator::evaluate(&then_ast, facts)?
            } else {
                bail!("Rule for '{}' has a condition but no 'then' clause.", attr_name)
            }
        } else {
            if let Some(other_str) = &rule.otherwise_value {
                let other_ast = parser::parse_rule(other_str)
                    .map_err(|e| anyhow::anyhow!("Failed to parse otherwise clause '{}': {:?}", other_str, e))?
                    .1;
                evaluator::evaluate(&other_ast, facts)?
            } else {
                 bail!("Rule condition for '{}' was false, but no 'otherwise' clause was found.", attr_name)
            }
        };

        facts.insert(attr_name.to_string(), final_value);
        Ok(())
    }
}
