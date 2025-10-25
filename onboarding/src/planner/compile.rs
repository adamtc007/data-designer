use crate::{ast::oodl::OnboardIntent, meta::loader::MetaBundle, ir::{Plan, Task, TaskKind, Idd, AttrSchema, Bindings}};
use anyhow::{Result, anyhow};
use std::collections::{BTreeMap, BTreeSet};

pub struct CompileInputs<'a> {
    pub intent: &'a OnboardIntent,
    pub meta: &'a MetaBundle,
    pub team_users: Vec<serde_json::Value>,
    pub cbu_profile: serde_json::Value,
}

pub struct CompileOutputs {
    pub plan: Plan,
    pub idd: Idd,
    pub bindings: Bindings,
}

pub fn compile_onboard(inp: CompileInputs) -> Result<CompileOutputs> {
    let prod = inp.meta.product_catalog.products.iter()
        .find(|p| p.id == inp.intent.products[0])
        .ok_or_else(|| anyhow!("product not found"))?;

    let mut schema: BTreeMap<String, AttrSchema> = BTreeMap::new();
    let mut required_set = BTreeSet::new();
    for rbind in prod.resources.as_ref().unwrap_or(&vec![]) {
        if let Some(rt) = inp.meta.resource_dicts.resource_types.iter().find(|rt| rt.id == rbind.r#type) {
            for a in &rt.dictionary.attrs {
                let entry = schema.entry(a.key.clone()).or_insert(AttrSchema {
                    r#type: a.r#type.clone(),
                    required: a.required,
                    provenance: vec![rt.id.clone()],
                    default: a.default.clone(),
                });
                entry.provenance.push(rt.id.clone());
                if a.required { required_set.insert(a.key.clone()); }
            }
        }
    }
    if required_set.contains("users") {
        schema.entry("users".into()).or_insert(AttrSchema{
            r#type:"array[user]".into(), required:true, provenance:vec!["team_sheet".into()], default:None
        });
    }

    let mut option_ids: Vec<String> = vec![];
    for s in &prod.services {
        for o in &s.options {
            option_ids.push(o.id.clone());
            schema.entry(o.id.clone()).or_insert(AttrSchema{
                r#type: o.kind.clone(),
                required: true,
                provenance: vec![format!("option:{}", s.service_id)],
                default: None,
            });
            required_set.insert(o.id.clone());
        }
    }

    let mut values: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    if required_set.contains("users") {
        values.insert("users".into(), serde_json::Value::Array(inp.team_users));
    }
    if let Some(region) = inp.cbu_profile.get("region").and_then(|v| v.as_str()) {
        values.insert("reporting-gaap".into(), serde_json::Value::String(
            if region == "US" { "USGAAP" } else { "IFRS" }.into()
        ));
    }

    let mut gaps: Vec<String> = vec![];
    for k in required_set {
        if !values.contains_key(&k) {
            gaps.push(k);
        }
    }
    let idd = Idd { schema, values, gaps: gaps.clone() };

    let mut steps: Vec<Task> = vec![];
    if !option_ids.is_empty() {
        steps.push(Task {
            id: "d1".into(),
            kind: TaskKind::SolicitData { options: option_ids.clone(), attrs: vec![], audience: "Client".into() },
            needs: vec![],
            after: vec![],
        });
    }
    let extra_needed: Vec<String> = idd.gaps.iter().filter(|g| !option_ids.contains(g)).cloned().collect();
    if !extra_needed.is_empty() {
        steps.push(Task {
            id: "d2".into(),
            kind: TaskKind::SolicitData { options: vec![], attrs: extra_needed, audience: "Client".into() },
            needs: vec![],
            after: vec![],
        });
    }
    for rbind in prod.resources.as_ref().unwrap_or(&vec![]) {
        let cfg_id = format!("cfg:{}", rbind.r#type);
        let act_id = format!("act:{}", rbind.r#type);
        let mut after = vec![];
        if !option_ids.is_empty() { after.push("d1".into()); }
        if steps.iter().any(|t| t.id=="d2") { after.push("d2".into()); }
        steps.push(Task {
            id: cfg_id.clone(),
            kind: TaskKind::ResourceOp { resource: rbind.r#type.clone(), op: "Configure".into() },
            needs: vec![], after,
        });
        steps.push(Task {
            id: act_id.clone(),
            kind: TaskKind::ResourceOp { resource: rbind.r#type.clone(), op: "Activate".into() },
            needs: vec![], after: vec![cfg_id],
        });
    }

    let plan = Plan {
        instance_id: inp.intent.instance_id.clone(),
        cbu_id: inp.intent.cbu_id.clone(),
        products: inp.intent.products.clone(),
        steps,
    };

    let bindings = Bindings::default();

    Ok(CompileOutputs { plan, idd, bindings })
}
