use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDictionary {
    pub entities: HashMap<String, Entity>,
    pub domains: HashMap<String, Domain>,
    pub lookups: HashMap<String, LookupTable>,
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub description: String,
    pub attributes: Vec<Attribute>,
    pub business_rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub data_type: DataType,
    pub description: String,
    pub required: bool,
    pub validation_rules: Vec<String>,
    pub domain: Option<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    String,
    Number,
    Boolean,
    Date,
    DateTime,
    Json,
    Array(Box<DataType>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub name: String,
    pub description: String,
    pub values: Vec<DomainValue>,
    pub validation_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainValue {
    pub code: String,
    pub label: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupTable {
    pub name: String,
    pub description: String,
    pub key_type: DataType,
    pub value_type: DataType,
    pub entries: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from_entity: String,
    pub from_attribute: String,
    pub to_entity: String,
    pub to_attribute: String,
    pub relationship_type: RelationshipType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

impl DataDictionary {
    pub fn new() -> Self {
        DataDictionary {
            entities: HashMap::new(),
            domains: HashMap::new(),
            lookups: HashMap::new(),
            relationships: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string(path)?;
        let dictionary: DataDictionary = serde_json::from_str(&content)?;
        Ok(dictionary)
    }

    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut dictionary = DataDictionary::new();

        // Load entities
        let entities_path = dir.as_ref().join("entities.json");
        if entities_path.exists() {
            let content = fs::read_to_string(&entities_path)?;
            let entities: HashMap<String, Entity> = serde_json::from_str(&content)?;
            dictionary.entities = entities;
        }

        // Load domains
        let domains_path = dir.as_ref().join("domains.json");
        if domains_path.exists() {
            let content = fs::read_to_string(&domains_path)?;
            let domains: HashMap<String, Domain> = serde_json::from_str(&content)?;
            dictionary.domains = domains;
        }

        // Load lookups
        let lookups_path = dir.as_ref().join("lookups.json");
        if lookups_path.exists() {
            let content = fs::read_to_string(&lookups_path)?;
            let lookups: HashMap<String, LookupTable> = serde_json::from_str(&content)?;
            dictionary.lookups = lookups;
        }

        // Load relationships
        let relationships_path = dir.as_ref().join("relationships.json");
        if relationships_path.exists() {
            let content = fs::read_to_string(&relationships_path)?;
            let relationships: Vec<Relationship> = serde_json::from_str(&content)?;
            dictionary.relationships = relationships;
        }

        Ok(dictionary)
    }

    pub fn get_all_attributes(&self) -> Vec<(String, Attribute)> {
        let mut attributes = Vec::new();

        for (entity_name, entity) in &self.entities {
            for attribute in &entity.attributes {
                attributes.push((
                    format!("{}.{}", entity_name, attribute.name),
                    attribute.clone()
                ));
            }
        }

        attributes
    }

    pub fn get_attribute_info(&self, full_name: &str) -> Option<Attribute> {
        let parts: Vec<&str> = full_name.split('.').collect();

        if parts.len() == 2 {
            if let Some(entity) = self.entities.get(parts[0]) {
                return entity.attributes.iter()
                    .find(|a| a.name == parts[1])
                    .cloned();
            }
        } else if parts.len() == 1 {
            // Search all entities for this attribute
            for entity in self.entities.values() {
                if let Some(attr) = entity.attributes.iter()
                    .find(|a| a.name == parts[0]) {
                    return Some(attr.clone());
                }
            }
        }

        None
    }

    pub fn get_domain_values(&self, domain_name: &str) -> Vec<String> {
        self.domains.get(domain_name)
            .map(|d| d.values.iter().map(|v| v.code.clone()).collect())
            .unwrap_or_default()
    }

    pub fn get_lookup_keys(&self, table_name: &str) -> Vec<String> {
        self.lookups.get(table_name)
            .map(|t| t.entries.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn create_default_kyc_dictionary() -> Self {
        let mut dictionary = DataDictionary::new();

        // KYC Client Entity
        let client_entity = Entity {
            name: "Client".to_string(),
            description: "KYC Client information".to_string(),
            attributes: vec![
                Attribute {
                    name: "client_id".to_string(),
                    data_type: DataType::String,
                    description: "Unique client identifier".to_string(),
                    required: true,
                    validation_rules: vec!["LENGTH(client_id) > 0".to_string()],
                    domain: None,
                    examples: vec!["CLT-001".to_string(), "CLT-002".to_string()],
                },
                Attribute {
                    name: "legal_entity_name".to_string(),
                    data_type: DataType::String,
                    description: "Legal entity name".to_string(),
                    required: true,
                    validation_rules: vec!["LENGTH(legal_entity_name) > 0".to_string()],
                    domain: None,
                    examples: vec!["Acme Corp".to_string(), "Global Trading LLC".to_string()],
                },
                Attribute {
                    name: "lei_code".to_string(),
                    data_type: DataType::String,
                    description: "Legal Entity Identifier".to_string(),
                    required: false,
                    validation_rules: vec!["IS_LEI(lei_code)".to_string()],
                    domain: None,
                    examples: vec!["529900T8BM49AURSDO55".to_string()],
                },
                Attribute {
                    name: "email".to_string(),
                    data_type: DataType::String,
                    description: "Primary contact email".to_string(),
                    required: true,
                    validation_rules: vec!["IS_EMAIL(email)".to_string()],
                    domain: None,
                    examples: vec!["contact@example.com".to_string()],
                },
                Attribute {
                    name: "risk_rating".to_string(),
                    data_type: DataType::String,
                    description: "Risk rating level".to_string(),
                    required: true,
                    validation_rules: vec![],
                    domain: Some("RiskLevel".to_string()),
                    examples: vec!["LOW".to_string(), "MEDIUM".to_string(), "HIGH".to_string()],
                },
                Attribute {
                    name: "aum_usd".to_string(),
                    data_type: DataType::Number,
                    description: "Assets under management in USD".to_string(),
                    required: false,
                    validation_rules: vec!["aum_usd >= 0".to_string()],
                    domain: None,
                    examples: vec!["1000000".to_string(), "50000000".to_string()],
                },
                Attribute {
                    name: "kyc_status".to_string(),
                    data_type: DataType::String,
                    description: "KYC completion status".to_string(),
                    required: true,
                    validation_rules: vec![],
                    domain: Some("KYCStatus".to_string()),
                    examples: vec!["PENDING".to_string(), "APPROVED".to_string()],
                },
                Attribute {
                    name: "pep_status".to_string(),
                    data_type: DataType::Boolean,
                    description: "Politically Exposed Person status".to_string(),
                    required: true,
                    validation_rules: vec![],
                    domain: None,
                    examples: vec!["true".to_string(), "false".to_string()],
                },
            ],
            business_rules: vec![
                "IF risk_rating == \"HIGH\" THEN documents_required >= 10".to_string(),
                "IF pep_status == true THEN enhanced_due_diligence = true".to_string(),
            ],
        };

        dictionary.entities.insert("Client".to_string(), client_entity);

        // Risk Level Domain
        let risk_domain = Domain {
            name: "RiskLevel".to_string(),
            description: "Risk rating levels".to_string(),
            values: vec![
                DomainValue {
                    code: "LOW".to_string(),
                    label: "Low Risk".to_string(),
                    description: Some("Standard due diligence required".to_string()),
                },
                DomainValue {
                    code: "MEDIUM".to_string(),
                    label: "Medium Risk".to_string(),
                    description: Some("Enhanced monitoring required".to_string()),
                },
                DomainValue {
                    code: "HIGH".to_string(),
                    label: "High Risk".to_string(),
                    description: Some("Enhanced due diligence required".to_string()),
                },
            ],
            validation_pattern: Some("^(LOW|MEDIUM|HIGH)$".to_string()),
        };

        dictionary.domains.insert("RiskLevel".to_string(), risk_domain);

        // KYC Status Domain
        let kyc_status_domain = Domain {
            name: "KYCStatus".to_string(),
            description: "KYC workflow status".to_string(),
            values: vec![
                DomainValue {
                    code: "PENDING".to_string(),
                    label: "Pending Review".to_string(),
                    description: Some("Awaiting document review".to_string()),
                },
                DomainValue {
                    code: "IN_REVIEW".to_string(),
                    label: "In Review".to_string(),
                    description: Some("Documents under review".to_string()),
                },
                DomainValue {
                    code: "APPROVED".to_string(),
                    label: "Approved".to_string(),
                    description: Some("KYC approved".to_string()),
                },
                DomainValue {
                    code: "REJECTED".to_string(),
                    label: "Rejected".to_string(),
                    description: Some("KYC rejected".to_string()),
                },
            ],
            validation_pattern: Some("^(PENDING|IN_REVIEW|APPROVED|REJECTED)$".to_string()),
        };

        dictionary.domains.insert("KYCStatus".to_string(), kyc_status_domain);

        // Country Lookup Table
        let mut country_lookup = LookupTable {
            name: "countries".to_string(),
            description: "Country code to name mapping".to_string(),
            key_type: DataType::String,
            value_type: DataType::String,
            entries: HashMap::new(),
        };

        country_lookup.entries.insert("US".to_string(), "United States".to_string());
        country_lookup.entries.insert("GB".to_string(), "United Kingdom".to_string());
        country_lookup.entries.insert("FR".to_string(), "France".to_string());
        country_lookup.entries.insert("DE".to_string(), "Germany".to_string());
        country_lookup.entries.insert("JP".to_string(), "Japan".to_string());

        dictionary.lookups.insert("countries".to_string(), country_lookup);

        dictionary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_dictionary() {
        let dict = DataDictionary::create_default_kyc_dictionary();

        assert!(dict.entities.contains_key("Client"));
        assert!(dict.domains.contains_key("RiskLevel"));
        assert!(dict.lookups.contains_key("countries"));

        let client = &dict.entities["Client"];
        assert_eq!(client.attributes.len(), 8);

        let risk_domain = &dict.domains["RiskLevel"];
        assert_eq!(risk_domain.values.len(), 3);
    }

    #[test]
    fn test_get_attribute_info() {
        let dict = DataDictionary::create_default_kyc_dictionary();

        let attr = dict.get_attribute_info("Client.email");
        assert!(attr.is_some());
        assert_eq!(attr.unwrap().name, "email");

        let attr = dict.get_attribute_info("risk_rating");
        assert!(attr.is_some());
        assert_eq!(attr.unwrap().domain, Some("RiskLevel".to_string()));
    }

    #[test]
    fn test_get_domain_values() {
        let dict = DataDictionary::create_default_kyc_dictionary();

        let values = dict.get_domain_values("RiskLevel");
        assert_eq!(values.len(), 3);
        assert!(values.contains(&"LOW".to_string()));
        assert!(values.contains(&"MEDIUM".to_string()));
        assert!(values.contains(&"HIGH".to_string()));
    }
}