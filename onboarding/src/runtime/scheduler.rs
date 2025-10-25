use crate::ir::{Plan, TaskKind};
use anyhow::Result;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct ExecutionConfig {}

pub async fn execute_plan(plan: &Plan, _cfg: &ExecutionConfig) -> Result<()> {
    info!(instance=%plan.instance_id, "starting execution");
    for t in &plan.steps {
        match &t.kind {
            TaskKind::SolicitData { options, attrs, audience } => {
                warn!(?options, ?attrs, %audience, "PAUSE: solicit data (stub)");
            }
            TaskKind::ResourceOp { resource, op } => {
                info!(%resource, %op, "execute resource op (stub)");
            }
        }
    }
    Ok(())
}
