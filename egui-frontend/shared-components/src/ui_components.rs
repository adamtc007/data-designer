use crate::financial_taxonomy::*;
use egui::{Ui, Color32};

/// Shared UI components for rendering financial taxonomy data

pub struct ProductDisplay;

impl ProductDisplay {
    pub fn render_product_list(ui: &mut Ui, products: &[Product]) {
        ui.heading("📦 Products");
        ui.separator();

        if products.is_empty() {
            ui.label("No products loaded. Click refresh to load data.");
        } else {
            for product in products {
                Self::render_product_item(ui, product);
            }
        }
    }

    pub fn render_product_item(ui: &mut Ui, product: &Product) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(&product.product_name);
                ui.label(format!("({})", product.product_id));
            });
            ui.label(format!("Business: {}", product.line_of_business));
            if let Some(desc) = &product.description {
                ui.label(format!("Description: {}", desc));
            }
            ui.horizontal(|ui| {
                ui.label("Status:");
                match product.status.as_str() {
                    "active" => ui.colored_label(Color32::GREEN, "✅ Active"),
                    _ => ui.colored_label(Color32::YELLOW, &product.status),
                };
            });
        });
    }
}

pub struct ProductOptionDisplay;

impl ProductOptionDisplay {
    pub fn render_product_options_list(ui: &mut Ui, options: &[ProductOption]) {
        ui.heading("⚙️ Product Options");
        ui.separator();

        if options.is_empty() {
            ui.label("No product options loaded.");
        } else {
            for option in options {
                Self::render_product_option_item(ui, option);
            }
        }
    }

    pub fn render_product_option_item(ui: &mut Ui, option: &ProductOption) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(&option.option_name);
                ui.label(format!("({})", option.option_category));
            });
            if let Some(display_name) = &option.display_name {
                ui.label(format!("Display: {}", display_name));
            }
            if let Some(pricing_impact) = option.pricing_impact {
                ui.label(format!("Pricing Impact: ${:.2}", pricing_impact));
            }
            ui.horizontal(|ui| {
                ui.label("Type:");
                match option.option_type.as_str() {
                    "required" => ui.colored_label(Color32::RED, "Required"),
                    "optional" => ui.colored_label(Color32::BLUE, "Optional"),
                    "premium" => ui.colored_label(Color32::GOLD, "Premium"),
                    _ => ui.label(&option.option_type),
                };
            });
        });
    }
}

pub struct ServiceDisplay;

impl ServiceDisplay {
    pub fn render_services_list(ui: &mut Ui, services: &[Service]) {
        ui.heading("🔧 Services");
        ui.separator();

        if services.is_empty() {
            ui.label("No services loaded.");
        } else {
            for service in services {
                Self::render_service_item(ui, service);
            }
        }
    }

    pub fn render_service_item(ui: &mut Ui, service: &Service) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(&service.service_name);
                if let Some(category) = &service.service_category {
                    ui.label(format!("({})", category));
                }
            });
            if let Some(service_type) = &service.service_type {
                ui.label(format!("Type: {}", service_type));
            }
            if let Some(delivery_model) = &service.delivery_model {
                ui.label(format!("Delivery: {}", delivery_model));
            }
            if let Some(billable) = service.billable {
                ui.horizontal(|ui| {
                    ui.label("Billable:");
                    if billable {
                        ui.colored_label(Color32::GREEN, "✅ Yes");
                    } else {
                        ui.colored_label(Color32::GRAY, "❌ No");
                    }
                });
            }
        });
    }
}

pub struct CbuDisplay;

impl CbuDisplay {
    pub fn render_cbu_mandate_structure_list(ui: &mut Ui, structures: &[CbuInvestmentMandateStructure]) {
        ui.heading("🏢 CBU Investment Structure");
        ui.separator();

        if structures.is_empty() {
            ui.label("No CBU mandate structure loaded. Click refresh to load data.");
        } else {
            for structure in structures {
                Self::render_cbu_mandate_structure_item(ui, structure);
            }
        }
    }

    pub fn render_cbu_mandate_structure_item(ui: &mut Ui, structure: &CbuInvestmentMandateStructure) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(&structure.cbu_name);
                ui.label(format!("({})", structure.cbu_id));
            });

            if let Some(mandate_id) = &structure.mandate_id {
                ui.horizontal(|ui| {
                    ui.label("📋 Mandate:");
                    ui.strong(mandate_id);
                });

                if let Some(asset_owner) = &structure.asset_owner_name {
                    ui.horizontal(|ui| {
                        ui.label("💰 Asset Owner:");
                        ui.label(asset_owner);
                    });
                }

                if let Some(investment_manager) = &structure.investment_manager_name {
                    ui.horizontal(|ui| {
                        ui.label("📊 Investment Manager:");
                        ui.label(investment_manager);
                    });
                }

                if let Some(currency) = &structure.base_currency {
                    ui.horizontal(|ui| {
                        ui.label("💱 Currency:");
                        ui.label(currency);
                    });
                }

                if let Some(instruments) = structure.total_instruments {
                    ui.horizontal(|ui| {
                        ui.label("🎪 Instruments:");
                        ui.label(format!("{}", instruments));
                    });
                }

                if let Some(families) = &structure.families {
                    ui.horizontal(|ui| {
                        ui.label("📁 Families:");
                        ui.label(families);
                    });
                }

                if let Some(exposure) = structure.total_exposure_pct {
                    ui.horizontal(|ui| {
                        ui.label("📈 Total Exposure:");
                        ui.label(format!("{:.1}%", exposure));
                    });
                }
            } else {
                ui.colored_label(Color32::GRAY, "No mandate assigned");
            }
        });
    }

    pub fn render_cbu_member_roles_list(ui: &mut Ui, roles: &[CbuMemberInvestmentRole]) {
        ui.heading("👥 CBU Member Roles");
        ui.separator();

        if roles.is_empty() {
            ui.label("No CBU member roles loaded.");
        } else {
            for role in roles {
                Self::render_cbu_member_role_item(ui, role);
            }
        }
    }

    pub fn render_cbu_member_role_item(ui: &mut Ui, role: &CbuMemberInvestmentRole) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(&role.entity_name);
                ui.label(format!("({})", role.role_name));
            });

            ui.horizontal(|ui| {
                ui.label("🏢 CBU:");
                ui.label(&role.cbu_name);
            });

            ui.horizontal(|ui| {
                ui.label("🎭 Role:");
                ui.strong(&role.role_code);
            });

            ui.horizontal(|ui| {
                ui.label("💼 Responsibility:");
                ui.label(&role.investment_responsibility);
            });

            if let Some(mandate_id) = &role.mandate_id {
                ui.horizontal(|ui| {
                    ui.label("📋 Mandate:");
                    ui.label(mandate_id);
                });
            }

            ui.horizontal(|ui| {
                ui.label("Authorities:");
                if role.has_trading_authority.unwrap_or(false) {
                    ui.colored_label(Color32::GREEN, "🔄 Trading");
                }
                if role.has_settlement_authority.unwrap_or(false) {
                    ui.colored_label(Color32::BLUE, "💱 Settlement");
                }
            });
        });
    }
}

pub struct TaxonomyHierarchyDisplay;

impl TaxonomyHierarchyDisplay {
    pub fn render_taxonomy_hierarchy_list(ui: &mut Ui, items: &[TaxonomyHierarchyItem]) {
        ui.heading("🏗️ Taxonomy Hierarchy");
        ui.separator();

        if items.is_empty() {
            ui.label("No taxonomy hierarchy items loaded.");
        } else {
            for item in items {
                Self::render_taxonomy_hierarchy_item(ui, item);
            }
        }
    }

    pub fn render_taxonomy_hierarchy_item(ui: &mut Ui, item: &TaxonomyHierarchyItem) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(&item.item_name);
                ui.label(format!("Level {}: {}", item.level, item.item_type));
            });
            if let Some(desc) = &item.item_description {
                ui.label(desc);
            }
        });
    }
}

pub struct AiSuggestionDisplay;

impl AiSuggestionDisplay {
    pub fn render_ai_suggestions_list(ui: &mut Ui, suggestions: &[AiSuggestion]) {
        ui.heading("✨ AI Suggestions");
        ui.separator();

        if suggestions.is_empty() {
            ui.label("No AI suggestions available.");
        } else {
            for suggestion in suggestions {
                Self::render_ai_suggestion_item(ui, suggestion);
            }
        }
    }

    pub fn render_ai_suggestion_item(ui: &mut Ui, suggestion: &AiSuggestion) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(&suggestion.title);
                ui.label(format!("({:.0}% confidence)", suggestion.confidence * 100.0));
            });
            ui.label(&suggestion.description);
            ui.horizontal(|ui| {
                ui.label("Category:");
                ui.label(&suggestion.category);
            });
            if !suggestion.applicable_contexts.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("Contexts:");
                    ui.label(suggestion.applicable_contexts.join(", "));
                });
            }
        });
    }
}