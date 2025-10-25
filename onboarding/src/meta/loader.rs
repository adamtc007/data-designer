use super::*;
use anyhow::Result;
use std::path::Path;

pub struct MetaBundle {
    pub cbu_templates: CbuTemplates,
    pub product_catalog: ProductCatalog,
    pub resource_dicts: ResourceDicts,
    pub rules: Rules,
}

pub fn load_from_dir(dir: &Path) -> Result<MetaBundle> {
    let cbu_templates: CbuTemplates = serde_yaml::from_str(&std::fs::read_to_string(dir.join("cbu_templates.yaml"))?)?;
    let product_catalog: ProductCatalog = serde_yaml::from_str(&std::fs::read_to_string(dir.join("product_catalog.yaml"))?)?;
    let mut dicts = vec![];
    for entry in std::fs::read_dir(dir.join("resource_dicts"))? {
        let p = entry?.path();
        if p.extension().and_then(|s| s.to_str()) == Some("yaml") {
            let rd: ResourceType = serde_yaml::from_str(&std::fs::read_to_string(p)?)?;
            dicts.push(rd);
        }
    }
    let resource_dicts = ResourceDicts { resource_types: dicts };
    let rules = Rules {
        rules: vec![ Rule {
            id: "choose-gaap".into(),
            language: "rust-fn".into(),
            expr: "choose_gaap".into()
        }],
    };
    Ok(MetaBundle { cbu_templates, product_catalog, resource_dicts, rules })
}
