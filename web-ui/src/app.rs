use eframe::egui;
use crate::{WebRouter, wasm_utils};
use crate::grpc_client::GrpcClient;
use crate::cbu_dsl_ide::CbuDslIDE;
use crate::cbu_state_manager::CbuStateManager;
use crate::resource_dsl_ide::ResourceDslIDE;
use crate::resource_state_manager::ResourceStateManager;
use crate::onboarding_ide::OnboardingIDE;
use crate::onboarding_state_manager::OnboardingStateManager;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ActiveView {
    Cbu,
    Resource,
    Onboarding,
}

/// Data Designer Application - CBU, Resource DSL, and Onboarding Workflow Management
pub struct DataDesignerWebApp {
    router: WebRouter,
    active_view: ActiveView,

    // Central state managers - single source of truth
    cbu_state: CbuStateManager,
    resource_state: ResourceStateManager,
    onboarding_state: OnboardingStateManager,

    // IDE components - UI only, references state
    cbu_dsl_ide: CbuDslIDE,
    resource_dsl_ide: ResourceDslIDE,
    onboarding_ide: OnboardingIDE,
}

impl DataDesignerWebApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        wasm_utils::set_panic_hook();
        wasm_utils::console_log("🚀 Starting Data Designer App with CBU, Resource DSL, and Onboarding Workflow management");

        let grpc_client = GrpcClient::new("http://localhost:8080");

        Self {
            router: WebRouter::new(),
            active_view: ActiveView::Cbu,
            cbu_state: CbuStateManager::new(Some(grpc_client.clone())),
            resource_state: ResourceStateManager::new(Some(grpc_client.clone())),
            onboarding_state: OnboardingStateManager::new(Some(grpc_client)),
            cbu_dsl_ide: CbuDslIDE::new(),
            resource_dsl_ide: ResourceDslIDE::new(),
            onboarding_ide: OnboardingIDE::new(),
        }
    }
}

impl eframe::App for DataDesignerWebApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Force continuous repainting to ensure responsiveness
        ctx.request_repaint();

        // Update state from async operations (polling pattern - will be improved)
        self.cbu_state.update_from_async();
        self.resource_state.update_from_async();
        self.onboarding_state.update_from_async();

        // Top panel with title and view tabs
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🏢 Data Designer");
                ui.separator();

                // View tabs
                ui.selectable_value(&mut self.active_view, ActiveView::Cbu, "📋 CBU DSL");
                ui.selectable_value(&mut self.active_view, ActiveView::Resource, "🔧 Resource DSL");
                ui.selectable_value(&mut self.active_view, ActiveView::Onboarding, "🚀 Onboarding Workflows");
            });
            ui.separator();
        });

        // Main content panel - show active view
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_view {
                ActiveView::Cbu => {
                    self.cbu_dsl_ide.render(ui, &mut self.cbu_state);
                }
                ActiveView::Resource => {
                    self.resource_dsl_ide.render(ui, &mut self.resource_state);
                }
                ActiveView::Onboarding => {
                    self.onboarding_ide.render(ui, &mut self.onboarding_state);
                }
            }
        });
    }
}