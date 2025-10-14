    // New Taxonomy UI Methods
    fn show_product_taxonomy_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("📦 Product Taxonomy");
        ui.separator();

        // Refresh button
        if ui.button("🔄 Refresh Data").clicked() {
            self.load_product_taxonomy();
        }

        ui.separator();

        // Products section
        ui.collapsing("🏪 Products", |ui| {
            if self.products.is_empty() {
                ui.label("No products loaded. Click refresh to load data.");
            } else {
                for product in &self.products {
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
                                "active" => ui.colored_label(egui::Color32::GREEN, "✅ Active"),
                                _ => ui.colored_label(egui::Color32::YELLOW, &product.status),
                            };
                        });
                    });
                }
            }
        });

        // Product Options section
        ui.collapsing("⚙️ Product Options", |ui| {
            if self.product_options.is_empty() {
                ui.label("No product options loaded.");
            } else {
                for option in &self.product_options {
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
                                "required" => ui.colored_label(egui::Color32::RED, "Required"),
                                "optional" => ui.colored_label(egui::Color32::BLUE, "Optional"),
                                "premium" => ui.colored_label(egui::Color32::GOLD, "Premium"),
                                _ => ui.label(&option.option_type),
                            };
                        });
                    });
                }
            }
        });

        // Services section
        ui.collapsing("🔧 Services", |ui| {
            if self.services.is_empty() {
                ui.label("No services loaded.");
            } else {
                for service in &self.services {
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
                                    ui.colored_label(egui::Color32::GREEN, "✅ Yes");
                                } else {
                                    ui.colored_label(egui::Color32::GRAY, "❌ No");
                                }
                            });
                        }
                    });
                }
            }
        });
    }

    fn show_investment_mandates_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎯 Investment Mandates");
        ui.separator();

        // Refresh button
        if ui.button("🔄 Refresh Data").clicked() {
            self.load_investment_mandates();
        }

        ui.separator();

        // CBU Investment Mandate Structure
        ui.collapsing("🏢 CBU Investment Structure", |ui| {
            if self.cbu_mandate_structure.is_empty() {
                ui.label("No CBU mandate structure loaded. Click refresh to load data.");
            } else {
                for structure in &self.cbu_mandate_structure {
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
                            ui.colored_label(egui::Color32::GRAY, "No mandate assigned");
                        }
                    });
                }
            }
        });

        // CBU Member Investment Roles
        ui.collapsing("👥 CBU Member Roles", |ui| {
            if self.cbu_member_roles.is_empty() {
                ui.label("No CBU member roles loaded.");
            } else {
                for role in &self.cbu_member_roles {
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
                                ui.colored_label(egui::Color32::GREEN, "🔄 Trading");
                            }
                            if role.has_settlement_authority.unwrap_or(false) {
                                ui.colored_label(egui::Color32::BLUE, "💱 Settlement");
                            }
                        });
                    });
                }
            }
        });
    }

    // Database Loading Methods
    fn load_product_taxonomy(&mut self) {
        if let Some(ref pool) = self.db_pool {
            let pool = pool.clone();
            let rt = self.runtime.clone();

            // Load products
            match rt.block_on(async {
                sqlx::query_as!(Product,
                    "SELECT id, product_id, product_name, line_of_business, description, status,
                     contract_type, commercial_status, pricing_model, target_market
                     FROM products WHERE status = 'active' ORDER BY product_name")
                .fetch_all(&pool)
                .await
            }) {
                Ok(products) => {
                    self.products = products;
                    self.status_message = format!("Loaded {} products", self.products.len());
                }
                Err(e) => {
                    self.status_message = format!("Error loading products: {}", e);
                }
            }

            // Load product options
            match rt.block_on(async {
                sqlx::query_as!(ProductOption,
                    "SELECT id, option_id, product_id, option_name, option_category, option_type,
                     option_value, display_name, description, pricing_impact, status
                     FROM product_options WHERE status = 'active' ORDER BY option_name")
                .fetch_all(&pool)
                .await
            }) {
                Ok(options) => {
                    self.product_options = options;
                }
                Err(e) => {
                    eprintln!("Error loading product options: {}", e);
                }
            }

            // Load services
            match rt.block_on(async {
                sqlx::query_as!(Service,
                    "SELECT id, service_id, service_name, service_category, description,
                     service_type, delivery_model, billable, status
                     FROM services WHERE status = 'active' ORDER BY service_name")
                .fetch_all(&pool)
                .await
            }) {
                Ok(services) => {
                    self.services = services;
                }
                Err(e) => {
                    eprintln!("Error loading services: {}", e);
                }
            }
        }
    }

    fn load_investment_mandates(&mut self) {
        if let Some(ref pool) = self.db_pool {
            let pool = pool.clone();
            let rt = self.runtime.clone();

            // Load CBU investment mandate structure
            match rt.block_on(async {
                sqlx::query_as!(CbuInvestmentMandateStructure,
                    "SELECT cbu_id, cbu_name, mandate_id, asset_owner_name, investment_manager_name,
                     base_currency, total_instruments, families, total_exposure_pct
                     FROM cbu_investment_mandate_structure ORDER BY cbu_id")
                .fetch_all(&pool)
                .await
            }) {
                Ok(structure) => {
                    self.cbu_mandate_structure = structure;
                    self.status_message = format!("Loaded {} CBU mandate structures", self.cbu_mandate_structure.len());
                }
                Err(e) => {
                    self.status_message = format!("Error loading CBU mandate structure: {}", e);
                }
            }

            // Load CBU member investment roles
            match rt.block_on(async {
                sqlx::query_as!(CbuMemberInvestmentRole,
                    "SELECT cbu_id, cbu_name, entity_name, entity_lei, role_name, role_code,
                     investment_responsibility, mandate_id, has_trading_authority, has_settlement_authority
                     FROM cbu_member_investment_roles ORDER BY cbu_id, role_code")
                .fetch_all(&pool)
                .await
            }) {
                Ok(roles) => {
                    self.cbu_member_roles = roles;
                }
                Err(e) => {
                    eprintln!("Error loading CBU member roles: {}", e);
                }
            }
        }
    }

    fn show_taxonomy_hierarchy_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🏗️ Taxonomy Hierarchy");
        ui.separator();

        ui.label("Complete Products → Options → Services → Resources hierarchy view");
        ui.separator();

        if ui.button("🔄 Load Sample Hierarchy").clicked() {
            self.load_taxonomy_hierarchy();
        }

        if !self.taxonomy_hierarchy.is_empty() {
            ui.collapsing("📊 Hierarchy Data", |ui| {
                for item in &self.taxonomy_hierarchy {
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
            });
        }
    }

    fn load_taxonomy_hierarchy(&mut self) {
        // For now, create a sample hierarchy
        self.taxonomy_hierarchy = vec![
            TaxonomyHierarchyItem {
                level: 1,
                item_type: "product".to_string(),
                item_id: 1,
                item_name: "Institutional Custody Plus".to_string(),
                item_description: Some("Comprehensive custody services".to_string()),
                parent_id: None,
                configuration: None,
                metadata: None,
            },
            TaxonomyHierarchyItem {
                level: 2,
                item_type: "product_option".to_string(),
                item_id: 2,
                item_name: "US Market Settlement".to_string(),
                item_description: Some("Settlement in US markets".to_string()),
                parent_id: Some(1),
                configuration: None,
                metadata: None,
            },
        ];
        self.status_message = "Loaded sample taxonomy hierarchy".to_string();
    }