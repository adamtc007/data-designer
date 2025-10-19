use crate::models::{DataDictionary, Value};
use crate::evaluator::Facts; // <-- Import the new evaluator
use crate::parser::parse_expression;
use crate::evaluator::evaluate;
use anyhow::{Context, Result, bail};

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

        // Look for derived attribute definition
        let attr_def = self.dictionary.derived_attributes.iter()
            .find(|attr| attr.name == attr_name)
            .with_context(|| format!("Definition for '{}' not found", attr_name))?;

        // Calculate dependencies first
        for dep in &attr_def.dependencies {
            self.calculate_attribute_recursive(dep, facts)?;
        }

        // Calculate the derived value using the first rule
        if let Some(rule) = attr_def.rules.first() {
            let final_value = if let Some(condition_str) = &rule.condition {
                // Evaluate condition
                let (_, condition_ast) = parse_expression(condition_str)?;
                let condition_result = evaluate(&condition_ast, facts)?;

                let condition_met = match condition_result {
                    Value::Boolean(b) => b,
                    _ => false,
                };

                if condition_met {
                    if let Some(then_str) = &rule.value {
                        let (_, then_ast) = parse_expression(then_str)?;
                        evaluate(&then_ast, facts)?
                    } else {
                        bail!("Rule for '{}' has a condition but no 'then' clause", attr_name)
                    }
                } else {
                    if let Some(else_str) = &rule.otherwise_value {
                        let (_, else_ast) = parse_expression(else_str)?;
                        evaluate(&else_ast, facts)?
                    } else {
                        bail!("Rule condition for '{}' was false, but no 'otherwise' clause found", attr_name)
                    }
                }
            } else {
                // No condition, just evaluate the value
                if let Some(value_str) = &rule.value {
                    let (_, value_ast) = parse_expression(value_str)?;
                    evaluate(&value_ast, facts)?
                } else {
                    bail!("Rule for '{}' has no condition and no value", attr_name)
                }
            };

            facts.insert(attr_name.to_string(), final_value);
        } else {
            bail!("No rules found for derived attribute '{}'", attr_name);
        }

        Ok(())
    }
}
