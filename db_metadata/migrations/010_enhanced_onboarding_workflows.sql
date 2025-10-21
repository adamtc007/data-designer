-- Enhanced Onboarding Workflows Migration
-- Capability-driven onboarding system linking CBUs to products and resources

-- Enhanced Onboarding Workflows: Main workflow tracking
CREATE TABLE onboarding_workflows (
    id SERIAL PRIMARY KEY,
    workflow_id VARCHAR(50) UNIQUE NOT NULL,
    cbu_id INTEGER REFERENCES client_business_units(id) ON DELETE CASCADE,
    product_ids INTEGER[] NOT NULL, -- Array of product IDs
    workflow_status VARCHAR(20) DEFAULT 'initiated',
    priority VARCHAR(20) DEFAULT 'medium',
    target_go_live_date DATE,
    business_requirements JSONB DEFAULT '{}'::jsonb,
    compliance_requirements JSONB DEFAULT '{}'::jsonb,
    resource_requirements JSONB DEFAULT '{}'::jsonb, -- Calculated from product→service→resource mapping
    execution_plan JSONB DEFAULT '[]'::jsonb, -- Ordered list of resource templates and capabilities
    current_stage VARCHAR(100),
    completion_percentage INTEGER DEFAULT 0,
    requested_by VARCHAR(100),
    assigned_to VARCHAR(100),
    approval_chain JSONB DEFAULT '[]'::jsonb,
    estimated_duration_days INTEGER,
    actual_duration_days INTEGER,
    created_by VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_by VARCHAR(100),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT workflow_status_check CHECK (workflow_status IN ('initiated', 'in_progress', 'completed', 'failed', 'cancelled')),
    CONSTRAINT priority_check CHECK (priority IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT completion_percentage_check CHECK (completion_percentage >= 0 AND completion_percentage <= 100)
);

-- Onboarding Resource Tasks: Individual capability execution tasks
CREATE TABLE onboarding_resource_tasks (
    id SERIAL PRIMARY KEY,
    workflow_id INTEGER REFERENCES onboarding_workflows(id) ON DELETE CASCADE,
    resource_template_id INTEGER REFERENCES resource_templates(id) ON DELETE CASCADE,
    capability_id INTEGER REFERENCES resource_capabilities(id) ON DELETE CASCADE,
    task_order INTEGER NOT NULL,
    task_status VARCHAR(20) DEFAULT 'pending',
    input_attributes JSONB DEFAULT '{}'::jsonb,
    output_attributes JSONB,
    validation_results JSONB,
    execution_log JSONB DEFAULT '[]'::jsonb,
    assigned_to VARCHAR(100),
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    estimated_hours REAL,
    actual_hours REAL,
    blocking_issues TEXT,
    retry_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT task_status_check CHECK (task_status IN ('pending', 'in_progress', 'completed', 'failed', 'blocked'))
);

-- Onboarding Dependencies: Task dependency management
CREATE TABLE onboarding_dependencies (
    id SERIAL PRIMARY KEY,
    workflow_id INTEGER REFERENCES onboarding_workflows(id) ON DELETE CASCADE,
    source_task_id INTEGER REFERENCES onboarding_resource_tasks(id) ON DELETE CASCADE,
    target_task_id INTEGER REFERENCES onboarding_resource_tasks(id) ON DELETE CASCADE,
    dependency_type VARCHAR(20) DEFAULT 'blocking',
    dependency_condition TEXT,
    is_satisfied BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT dependency_type_check CHECK (dependency_type IN ('blocking', 'informational', 'conditional')),
    CONSTRAINT no_self_dependency CHECK (source_task_id != target_task_id)
);

-- Onboarding Approvals: Approval workflow tracking
CREATE TABLE onboarding_approvals (
    id SERIAL PRIMARY KEY,
    workflow_id INTEGER REFERENCES onboarding_workflows(id) ON DELETE CASCADE,
    approval_stage VARCHAR(50) NOT NULL,
    approver_role VARCHAR(100) NOT NULL,
    approver_user VARCHAR(100),
    approval_status VARCHAR(20) DEFAULT 'pending',
    approval_notes TEXT,
    approved_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT approval_status_check CHECK (approval_status IN ('pending', 'approved', 'rejected', 'delegated'))
);

-- Indexes for performance
CREATE INDEX idx_onboarding_workflows_cbu ON onboarding_workflows(cbu_id);
CREATE INDEX idx_onboarding_workflows_status ON onboarding_workflows(workflow_status);
CREATE INDEX idx_onboarding_workflows_priority ON onboarding_workflows(priority);
CREATE INDEX idx_onboarding_workflows_target_date ON onboarding_workflows(target_go_live_date);
CREATE INDEX idx_onboarding_resource_tasks_workflow ON onboarding_resource_tasks(workflow_id);
CREATE INDEX idx_onboarding_resource_tasks_status ON onboarding_resource_tasks(task_status);
CREATE INDEX idx_onboarding_resource_tasks_order ON onboarding_resource_tasks(workflow_id, task_order);
CREATE INDEX idx_onboarding_dependencies_workflow ON onboarding_dependencies(workflow_id);
CREATE INDEX idx_onboarding_approvals_workflow ON onboarding_approvals(workflow_id);
CREATE INDEX idx_onboarding_approvals_status ON onboarding_approvals(approval_status);

-- Views for reporting and monitoring
CREATE VIEW v_onboarding_workflow_details AS
SELECT
    ow.workflow_id,
    cbu.cbu_name,
    ow.workflow_status,
    ow.priority,
    ow.current_stage,
    ow.completion_percentage,
    COUNT(ort.*) as total_tasks,
    COUNT(CASE WHEN ort.task_status = 'completed' THEN 1 END) as completed_tasks,
    COUNT(CASE WHEN ort.task_status = 'failed' THEN 1 END) as failed_tasks,
    COUNT(CASE WHEN ort.task_status = 'blocked' THEN 1 END) as blocked_tasks,
    ow.target_go_live_date,
    CASE
        WHEN ow.target_go_live_date IS NOT NULL
        THEN EXTRACT(DAY FROM ow.target_go_live_date - CURRENT_DATE)::int
        ELSE NULL
    END as days_remaining,
    ow.assigned_to,
    ow.created_at
FROM onboarding_workflows ow
JOIN client_business_units cbu ON ow.cbu_id = cbu.id
LEFT JOIN onboarding_resource_tasks ort ON ow.id = ort.workflow_id
GROUP BY ow.workflow_id, cbu.cbu_name, ow.workflow_status, ow.priority,
         ow.current_stage, ow.completion_percentage, ow.target_go_live_date,
         ow.assigned_to, ow.created_at
ORDER BY ow.created_at DESC;

CREATE VIEW v_resource_provisioning_status AS
SELECT
    ow.workflow_id,
    rt.template_name as resource_template_name,
    rt.resource_type,
    COUNT(ort.*) as total_capabilities,
    COUNT(CASE WHEN ort.task_status = 'completed' THEN 1 END) as completed_capabilities,
    COUNT(CASE WHEN ort.task_status = 'failed' THEN 1 END) as failed_capabilities,
    (SELECT rc.capability_name FROM resource_capabilities rc
     JOIN onboarding_resource_tasks ort2 ON rc.id = ort2.capability_id
     WHERE ort2.workflow_id = ow.id AND ort2.task_status = 'in_progress'
     LIMIT 1) as current_capability,
    CASE
        WHEN COUNT(CASE WHEN ort.task_status = 'failed' THEN 1 END) > 0 THEN 'failed'
        WHEN COUNT(CASE WHEN ort.task_status = 'completed' THEN 1 END) = COUNT(ort.*) THEN 'completed'
        WHEN COUNT(CASE WHEN ort.task_status IN ('in_progress', 'completed') THEN 1 END) > 0 THEN 'in_progress'
        ELSE 'pending'
    END as provision_status
FROM onboarding_workflows ow
JOIN onboarding_resource_tasks ort ON ow.id = ort.workflow_id
JOIN resource_templates rt ON ort.resource_template_id = rt.id
GROUP BY ow.workflow_id, rt.template_name, rt.resource_type, ow.id
ORDER BY ow.workflow_id, rt.template_name;

CREATE VIEW v_onboarding_task_details AS
SELECT
    ort.id as task_id,
    ow.workflow_id,
    cbu.cbu_name,
    rt.template_name as resource_template_name,
    rt.resource_type,
    rc.capability_name,
    rc.capability_type,
    ort.task_order,
    ort.task_status,
    ort.assigned_to,
    ort.started_at,
    ort.completed_at,
    ort.estimated_hours,
    ort.actual_hours,
    ort.blocking_issues,
    ort.retry_count,
    ort.created_at
FROM onboarding_resource_tasks ort
JOIN onboarding_workflows ow ON ort.workflow_id = ow.id
JOIN client_business_units cbu ON ow.cbu_id = cbu.id
JOIN resource_templates rt ON ort.resource_template_id = rt.id
JOIN resource_capabilities rc ON ort.capability_id = rc.id
ORDER BY ow.workflow_id, ort.task_order;

-- Triggers for automatic updates
CREATE OR REPLACE FUNCTION update_workflow_completion()
RETURNS TRIGGER AS $$
BEGIN
    -- Update completion percentage and current stage
    UPDATE onboarding_workflows SET
        completion_percentage = (
            SELECT ROUND(
                100.0 * COUNT(CASE WHEN task_status = 'completed' THEN 1 END) /
                NULLIF(COUNT(*), 0)
            )::int
            FROM onboarding_resource_tasks
            WHERE workflow_id = NEW.workflow_id
        ),
        current_stage = (
            SELECT CONCAT(rt.template_name, ' - ', rc.capability_name)
            FROM onboarding_resource_tasks ort
            JOIN resource_templates rt ON ort.resource_template_id = rt.id
            JOIN resource_capabilities rc ON ort.capability_id = rc.id
            WHERE ort.workflow_id = NEW.workflow_id
            AND ort.task_status = 'in_progress'
            ORDER BY ort.task_order
            LIMIT 1
        ),
        workflow_status = CASE
            WHEN (SELECT COUNT(*) FROM onboarding_resource_tasks
                  WHERE workflow_id = NEW.workflow_id AND task_status = 'failed') > 0
            THEN 'failed'
            WHEN (SELECT COUNT(*) FROM onboarding_resource_tasks
                  WHERE workflow_id = NEW.workflow_id AND task_status != 'completed') = 0
            THEN 'completed'
            WHEN (SELECT COUNT(*) FROM onboarding_resource_tasks
                  WHERE workflow_id = NEW.workflow_id AND task_status IN ('in_progress', 'completed')) > 0
            THEN 'in_progress'
            ELSE workflow_status
        END,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.workflow_id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_workflow_completion_trigger
    AFTER INSERT OR UPDATE ON onboarding_resource_tasks
    FOR EACH ROW
    EXECUTE FUNCTION update_workflow_completion();

-- Auto-update timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_onboarding_workflows_updated_at BEFORE UPDATE ON onboarding_workflows
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_onboarding_resource_tasks_updated_at BEFORE UPDATE ON onboarding_resource_tasks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE onboarding_workflows IS 'Capability-driven onboarding workflows linking CBUs to products and resources';
COMMENT ON TABLE onboarding_resource_tasks IS 'Individual capability execution tasks within an onboarding workflow';
COMMENT ON TABLE onboarding_dependencies IS 'Task dependency management for onboarding workflows';
COMMENT ON TABLE onboarding_approvals IS 'Approval workflow tracking for onboarding processes';