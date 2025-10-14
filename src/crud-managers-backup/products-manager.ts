import { invoke } from '@tauri-apps/api/core';

// Types based on the Rust backend structures
export interface Product {
    id: number;
    product_id: string;
    product_name: string;
    line_of_business: string;
    description?: string;
    status: string;
    pricing_model?: string;
    target_market?: string;
    regulatory_requirements?: any;
    sla_commitments?: any;
    created_by?: string;
    created_at: string;
    updated_by?: string;
    updated_at: string;
}

export interface Service {
    id: number;
    service_id: string;
    service_name: string;
    service_category?: string;
    description?: string;
    is_core_service: boolean;
    configuration_schema?: any;
    dependencies?: string[];
    status: string;
    created_by?: string;
    created_at: string;
    updated_by?: string;
    updated_at: string;
}

export interface Resource {
    id: number;
    resource_id: string;
    resource_name: string;
    resource_type: string;
    description?: string;
    location?: string;
    configuration?: any;
    access_policies?: any;
    status: string;
    created_by?: string;
    created_at: string;
    updated_by?: string;
    updated_at: string;
}

export interface CreateProductRequest {
    product_id: string;
    product_name: string;
    line_of_business: string;
    description?: string;
    pricing_model?: string;
    target_market?: string;
    created_by?: string;
}

export interface CreateServiceRequest {
    service_id: string;
    service_name: string;
    service_category?: string;
    description?: string;
    is_core_service: boolean;
    dependencies?: string[];
    created_by?: string;
}

export interface CreateResourceRequest {
    resource_id: string;
    resource_name: string;
    resource_type: string;
    description?: string;
    location?: string;
    created_by?: string;
}

export class ProductServicesManager {
    private selectedItem: Product | Service | Resource | null = null;
    private selectedItemType: 'product' | 'service' | 'resource' | null = null;
    private products: Product[] = [];
    private services: Service[] = [];
    private resources: Resource[] = [];

    async initialize() {
        console.log('🚀 Initializing Product Services Manager');
        await this.loadAllData();
        this.setupEventListeners();
    }

    async loadAllData() {
        try {
            // Load products
            const products = await invoke<Product[]>('list_products', { lineOfBusiness: null });
            this.products = products || [];
            console.log('✅ Loaded products:', this.products.length);

            // Load services
            const services = await invoke<Service[]>('list_services', { serviceCategory: null });
            this.services = services || [];
            console.log('✅ Loaded services:', this.services.length);

            // Load resources
            const resources = await invoke<Resource[]>('list_resources', { resourceType: null });
            this.resources = resources || [];
            console.log('✅ Loaded resources:', this.resources.length);

            this.renderItemsList();
        } catch (error) {
            console.error('❌ Error loading data:', error);
            this.showError('Failed to load data: ' + (error as Error).message);
        }
    }

    renderItemsList() {
        const container = document.getElementById('itemsList');
        if (!container) return;

        let html = '';

        // Products section
        if (this.products.length > 0) {
            html += '<div style="margin-bottom: 20px;">';
            html += '<h4 style="color: #667eea; margin-bottom: 10px;">📦 Products</h4>';
            this.products.forEach(product => {
                html += `
                    <div class="item" onclick="selectItem('product', '${product.id}')">
                        <div class="item-name">${product.product_name}</div>
                        <div class="item-id">ID: ${product.product_id}</div>
                        <div class="item-status ${product.status === 'active' ? 'status-active' : 'status-inactive'}">
                            ${product.status}
                        </div>
                    </div>
                `;
            });
            html += '</div>';
        }

        // Services section
        if (this.services.length > 0) {
            html += '<div style="margin-bottom: 20px;">';
            html += '<h4 style="color: #56ab2f; margin-bottom: 10px;">⚙️ Services</h4>';
            this.services.forEach(service => {
                html += `
                    <div class="item" onclick="selectItem('service', '${service.id}')">
                        <div class="item-name">${service.service_name}</div>
                        <div class="item-id">ID: ${service.service_id}</div>
                        <div class="item-status ${service.status === 'active' ? 'status-active' : 'status-inactive'}">
                            ${service.status}
                        </div>
                    </div>
                `;
            });
            html += '</div>';
        }

        // Resources section
        if (this.resources.length > 0) {
            html += '<div style="margin-bottom: 20px;">';
            html += '<h4 style="color: #ff6b6b; margin-bottom: 10px;">🛠️ Resources</h4>';
            this.resources.forEach(resource => {
                html += `
                    <div class="item" onclick="selectItem('resource', '${resource.id}')">
                        <div class="item-name">${resource.resource_name}</div>
                        <div class="item-id">ID: ${resource.resource_id}</div>
                        <div class="item-status ${resource.status === 'active' ? 'status-active' : 'status-inactive'}">
                            ${resource.status}
                        </div>
                    </div>
                `;
            });
            html += '</div>';
        }

        if (html === '') {
            html = '<div class="empty-state">No products, services, or resources found</div>';
        }

        container.innerHTML = html;

        // Make selectItem globally available
        (window as any).selectItem = (type: string, id: string) => {
            this.selectItem(type as 'product' | 'service' | 'resource', parseInt(id));
        };
    }

    async selectItem(type: 'product' | 'service' | 'resource', id: number) {
        this.selectedItemType = type;

        if (type === 'product') {
            this.selectedItem = this.products.find(p => p.id === id) || null;
        } else if (type === 'service') {
            this.selectedItem = this.services.find(s => s.id === id) || null;
        } else if (type === 'resource') {
            this.selectedItem = this.resources.find(r => r.id === id) || null;
        }

        if (this.selectedItem) {
            // Update UI selection
            document.querySelectorAll('.item').forEach(el => el.classList.remove('selected'));
            if (event && event.currentTarget) {
                (event.currentTarget as HTMLElement).classList.add('selected');
            }

            // Show details
            this.showItemDetails();
        }
    }

    showItemDetails() {
        const welcomeScreen = document.getElementById('welcomeScreen');
        const itemDetails = document.getElementById('itemDetails');

        if (welcomeScreen) welcomeScreen.style.display = 'none';
        if (itemDetails) itemDetails.style.display = 'block';

        this.renderOverviewTab();
        this.switchTab('overview');
    }

    renderOverviewTab() {
        const container = document.getElementById('overviewContent');
        if (!container || !this.selectedItem) return;

        let html = '<div class="form-group">';

        if (this.selectedItemType === 'product') {
            const product = this.selectedItem as Product;
            html += `
                <h3>📦 ${product.product_name}</h3>
                <div class="form-row" style="margin-top: 20px;">
                    <div><strong>Product ID:</strong> ${product.product_id}</div>
                    <div><strong>Line of Business:</strong> ${product.line_of_business}</div>
                </div>
                <div class="form-row">
                    <div><strong>Status:</strong> <span class="item-status ${product.status === 'active' ? 'status-active' : 'status-inactive'}">${product.status}</span></div>
                    <div><strong>Target Market:</strong> ${product.target_market || 'Not specified'}</div>
                </div>
                ${product.description ? `<div style="margin-top: 15px;"><strong>Description:</strong><br>${product.description}</div>` : ''}
                ${product.pricing_model ? `<div style="margin-top: 15px;"><strong>Pricing Model:</strong> ${product.pricing_model}</div>` : ''}
            `;
        } else if (this.selectedItemType === 'service') {
            const service = this.selectedItem as Service;
            html += `
                <h3>⚙️ ${service.service_name}</h3>
                <div class="form-row" style="margin-top: 20px;">
                    <div><strong>Service ID:</strong> ${service.service_id}</div>
                    <div><strong>Category:</strong> ${service.service_category || 'Not specified'}</div>
                </div>
                <div class="form-row">
                    <div><strong>Status:</strong> <span class="item-status ${service.status === 'active' ? 'status-active' : 'status-inactive'}">${service.status}</span></div>
                    <div><strong>Core Service:</strong> ${service.is_core_service ? 'Yes' : 'No'}</div>
                </div>
                ${service.description ? `<div style="margin-top: 15px;"><strong>Description:</strong><br>${service.description}</div>` : ''}
                ${service.dependencies && service.dependencies.length > 0 ? `<div style="margin-top: 15px;"><strong>Dependencies:</strong> ${service.dependencies.join(', ')}</div>` : ''}
            `;
        } else if (this.selectedItemType === 'resource') {
            const resource = this.selectedItem as Resource;
            html += `
                <h3>🛠️ ${resource.resource_name}</h3>
                <div class="form-row" style="margin-top: 20px;">
                    <div><strong>Resource ID:</strong> ${resource.resource_id}</div>
                    <div><strong>Type:</strong> ${resource.resource_type}</div>
                </div>
                <div class="form-row">
                    <div><strong>Status:</strong> <span class="item-status ${resource.status === 'active' ? 'status-active' : 'status-inactive'}">${resource.status}</span></div>
                    <div><strong>Location:</strong> ${resource.location || 'Not specified'}</div>
                </div>
                ${resource.description ? `<div style="margin-top: 15px;"><strong>Description:</strong><br>${resource.description}</div>` : ''}
            `;
        }

        html += '</div>';
        container.innerHTML = html;
    }

    switchTab(tabName: string) {
        // Update tab states
        document.querySelectorAll('.tab').forEach(tab => {
            tab.classList.remove('active');
        });
        document.querySelectorAll('.tab-content').forEach(content => {
            content.classList.remove('active');
        });

        // Activate selected tab
        const activeTab = document.querySelector(`.tab[onclick="switchTab('${tabName}')"]`);
        const activeContent = document.getElementById(`${tabName}-tab`);

        if (activeTab) activeTab.classList.add('active');
        if (activeContent) activeContent.classList.add('active');

        // Load tab-specific content
        if (tabName === 'services' && this.selectedItemType === 'product') {
            this.renderServicesTab();
        } else if (tabName === 'edit') {
            this.renderEditTab();
        }
    }

    renderServicesTab() {
        const container = document.getElementById('servicesContent');
        if (!container) return;

        // For now, show related services (this could be expanded with actual relationships)
        const relatedServices = this.services.filter(service =>
            this.selectedItemType === 'product' &&
            service.service_category === (this.selectedItem as Product)?.line_of_business
        );

        let html = '';
        if (relatedServices.length > 0) {
            html += '<div class="services-grid">';
            relatedServices.forEach(service => {
                html += `
                    <div class="service-card">
                        <div class="service-name">${service.service_name}</div>
                        <div class="service-category">${service.service_category || 'No category'}</div>
                        <div class="service-description">${service.description || 'No description'}</div>
                    </div>
                `;
            });
            html += '</div>';
        } else {
            html = '<div class="empty-state">No related services found</div>';
        }

        container.innerHTML = html;
    }

    renderEditTab() {
        const container = document.getElementById('editForm');
        if (!container || !this.selectedItem) return;

        let html = '<form id="editItemForm">';

        if (this.selectedItemType === 'product') {
            const product = this.selectedItem as Product;
            html += `
                <div class="form-row">
                    <div class="form-group">
                        <label for="editProductId">Product ID *</label>
                        <input type="text" id="editProductId" name="product_id" value="${product.product_id}" required>
                    </div>
                    <div class="form-group">
                        <label for="editProductName">Product Name *</label>
                        <input type="text" id="editProductName" name="product_name" value="${product.product_name}" required>
                    </div>
                </div>
                <div class="form-row">
                    <div class="form-group">
                        <label for="editLob">Line of Business *</label>
                        <select id="editLob" name="line_of_business" required>
                            <option value="Investment Management" ${product.line_of_business === 'Investment Management' ? 'selected' : ''}>Investment Management</option>
                            <option value="Private Banking" ${product.line_of_business === 'Private Banking' ? 'selected' : ''}>Private Banking</option>
                            <option value="Corporate Banking" ${product.line_of_business === 'Corporate Banking' ? 'selected' : ''}>Corporate Banking</option>
                            <option value="Asset Management" ${product.line_of_business === 'Asset Management' ? 'selected' : ''}>Asset Management</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label for="editPricingModel">Pricing Model</label>
                        <select id="editPricingModel" name="pricing_model">
                            <option value="">Select Model</option>
                            <option value="Flat Fee" ${product.pricing_model === 'Flat Fee' ? 'selected' : ''}>Flat Fee</option>
                            <option value="Percentage" ${product.pricing_model === 'Percentage' ? 'selected' : ''}>Percentage</option>
                            <option value="Tiered" ${product.pricing_model === 'Tiered' ? 'selected' : ''}>Tiered</option>
                        </select>
                    </div>
                </div>
                <div class="form-group">
                    <label for="editProductDescription">Description</label>
                    <textarea id="editProductDescription" name="description" rows="3">${product.description || ''}</textarea>
                </div>
                <div class="form-group">
                    <label for="editTargetMarket">Target Market</label>
                    <input type="text" id="editTargetMarket" name="target_market" value="${product.target_market || ''}">
                </div>
            `;
        } else if (this.selectedItemType === 'service') {
            const service = this.selectedItem as Service;
            html += `
                <div class="form-row">
                    <div class="form-group">
                        <label for="editServiceId">Service ID *</label>
                        <input type="text" id="editServiceId" name="service_id" value="${service.service_id}" required>
                    </div>
                    <div class="form-group">
                        <label for="editServiceName">Service Name *</label>
                        <input type="text" id="editServiceName" name="service_name" value="${service.service_name}" required>
                    </div>
                </div>
                <div class="form-row">
                    <div class="form-group">
                        <label for="editServiceCategory">Service Category</label>
                        <input type="text" id="editServiceCategory" name="service_category" value="${service.service_category || ''}">
                    </div>
                    <div class="form-group">
                        <label for="editCoreService">Core Service</label>
                        <select id="editCoreService" name="is_core_service">
                            <option value="true" ${service.is_core_service ? 'selected' : ''}>Yes</option>
                            <option value="false" ${!service.is_core_service ? 'selected' : ''}>No</option>
                        </select>
                    </div>
                </div>
                <div class="form-group">
                    <label for="editServiceDescription">Description</label>
                    <textarea id="editServiceDescription" name="description" rows="3">${service.description || ''}</textarea>
                </div>
            `;
        } else if (this.selectedItemType === 'resource') {
            const resource = this.selectedItem as Resource;
            html += `
                <div class="form-row">
                    <div class="form-group">
                        <label for="editResourceId">Resource ID *</label>
                        <input type="text" id="editResourceId" name="resource_id" value="${resource.resource_id}" required>
                    </div>
                    <div class="form-group">
                        <label for="editResourceName">Resource Name *</label>
                        <input type="text" id="editResourceName" name="resource_name" value="${resource.resource_name}" required>
                    </div>
                </div>
                <div class="form-row">
                    <div class="form-group">
                        <label for="editResourceType">Resource Type *</label>
                        <select id="editResourceType" name="resource_type" required>
                            <option value="Database" ${resource.resource_type === 'Database' ? 'selected' : ''}>Database</option>
                            <option value="API Endpoint" ${resource.resource_type === 'API Endpoint' ? 'selected' : ''}>API Endpoint</option>
                            <option value="Message Queue" ${resource.resource_type === 'Message Queue' ? 'selected' : ''}>Message Queue</option>
                            <option value="File System" ${resource.resource_type === 'File System' ? 'selected' : ''}>File System</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label for="editLocation">Location</label>
                        <input type="text" id="editLocation" name="location" value="${resource.location || ''}">
                    </div>
                </div>
                <div class="form-group">
                    <label for="editResourceDescription">Description</label>
                    <textarea id="editResourceDescription" name="description" rows="3">${resource.description || ''}</textarea>
                </div>
            `;
        }

        html += `
            <div style="margin-top: 30px;">
                <button type="submit" class="btn btn-success">Update ${this.selectedItemType}</button>
                <button type="button" class="btn btn-danger" onclick="deleteCurrentItem()">Delete</button>
            </div>
        </form>`;

        container.innerHTML = html;

        // Add event listeners
        const form = document.getElementById('editItemForm') as HTMLFormElement;
        if (form) {
            form.addEventListener('submit', (e) => this.handleUpdateItem(e));
        }

        // Make deleteCurrentItem globally available
        (window as any).deleteCurrentItem = () => this.deleteCurrentItem();
    }

    async handleUpdateItem(event: Event) {
        event.preventDefault();
        // Implementation would go here - update the item
        this.showMessage('Update functionality not yet implemented', 'info');
    }

    async deleteCurrentItem() {
        if (!this.selectedItem || !this.selectedItemType) return;

        if (confirm(`Are you sure you want to delete this ${this.selectedItemType}?`)) {
            // Implementation would go here - delete the item
            this.showMessage('Delete functionality not yet implemented', 'info');
        }
    }

    showCreateProductForm() {
        const welcomeScreen = document.getElementById('welcomeScreen');
        const itemDetails = document.getElementById('itemDetails');
        const createForm = document.getElementById('createForm');

        if (welcomeScreen) welcomeScreen.style.display = 'none';
        if (itemDetails) itemDetails.style.display = 'none';
        if (createForm) createForm.style.display = 'block';

        this.renderCreateForm();
    }

    renderCreateForm() {
        const container = document.getElementById('createForm');
        if (!container) return;

        const html = `
            <h3>Create New Product</h3>
            <form id="productCreateForm">
                <div class="form-row">
                    <div class="form-group">
                        <label for="productId">Product ID *</label>
                        <input type="text" id="productId" name="product_id" required>
                    </div>
                    <div class="form-group">
                        <label for="productName">Product Name *</label>
                        <input type="text" id="productName" name="product_name" required>
                    </div>
                </div>
                <div class="form-row">
                    <div class="form-group">
                        <label for="lob">Line of Business *</label>
                        <select id="lob" name="line_of_business" required>
                            <option value="">Select LOB</option>
                            <option value="Investment Management">Investment Management</option>
                            <option value="Private Banking">Private Banking</option>
                            <option value="Corporate Banking">Corporate Banking</option>
                            <option value="Asset Management">Asset Management</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label for="pricingModel">Pricing Model</label>
                        <select id="pricingModel" name="pricing_model">
                            <option value="">Select Model</option>
                            <option value="Flat Fee">Flat Fee</option>
                            <option value="Percentage">Percentage</option>
                            <option value="Tiered">Tiered</option>
                        </select>
                    </div>
                </div>
                <div class="form-group">
                    <label for="description">Description</label>
                    <textarea id="description" name="description" rows="3"></textarea>
                </div>
                <div class="form-group">
                    <label for="targetMarket">Target Market</label>
                    <input type="text" id="targetMarket" name="target_market">
                </div>
                <div style="margin-top: 30px;">
                    <button type="submit" class="btn btn-success">Create Product</button>
                    <button type="button" class="btn btn-primary" onclick="cancelCreate()">Cancel</button>
                </div>
            </form>
        `;

        container.innerHTML = html;

        const form = document.getElementById('productCreateForm') as HTMLFormElement;
        if (form) {
            form.addEventListener('submit', (e) => this.handleCreateProduct(e));
        }
    }

    async handleCreateProduct(event: Event) {
        event.preventDefault();
        const form = event.target as HTMLFormElement;
        const formData = new FormData(form);

        const request: CreateProductRequest = {
            product_id: formData.get('product_id') as string,
            product_name: formData.get('product_name') as string,
            line_of_business: formData.get('line_of_business') as string,
            description: formData.get('description') as string || undefined,
            pricing_model: formData.get('pricing_model') as string || undefined,
            target_market: formData.get('target_market') as string || undefined,
            created_by: 'system'
        };

        try {
            const product = await invoke<Product>('create_product', request as any);
            this.products.push(product);
            this.renderItemsList();
            this.cancelCreate();
            this.showMessage(`Product "${product.product_name}" created successfully!`, 'success');
        } catch (error) {
            console.error('Error creating product:', error);
            this.showError('Failed to create product: ' + (error as Error).message);
        }
    }

    showAddServiceForm() {
        this.showMessage('Add Service functionality not yet implemented', 'info');
    }

    cancelCreate() {
        const welcomeScreen = document.getElementById('welcomeScreen');
        const itemDetails = document.getElementById('itemDetails');
        const createForm = document.getElementById('createForm');

        if (createForm) createForm.style.display = 'none';

        if (this.selectedItem) {
            if (itemDetails) itemDetails.style.display = 'block';
        } else {
            if (welcomeScreen) welcomeScreen.style.display = 'block';
        }
    }

    goBackToIDE() {
        window.location.href = 'index.html';
    }

    setupEventListeners() {
        const searchBox = document.getElementById('searchBox') as HTMLInputElement;
        if (searchBox) {
            searchBox.addEventListener('input', (e) => {
                const target = e.target as HTMLInputElement;
                this.filterItems(target.value);
            });
        }
    }

    filterItems(searchTerm: string) {
        const items = document.querySelectorAll('.item');
        const term = searchTerm.toLowerCase();

        items.forEach(item => {
            const text = item.textContent?.toLowerCase() || '';
            if (text.includes(term)) {
                (item as HTMLElement).style.display = 'block';
            } else {
                (item as HTMLElement).style.display = 'none';
            }
        });
    }

    showMessage(message: string, type: 'success' | 'error' | 'info') {
        const messageDiv = document.createElement('div');
        messageDiv.className = type;
        messageDiv.textContent = message;

        const container = document.querySelector('.content-area');
        if (container) {
            container.insertBefore(messageDiv, container.firstChild);

            setTimeout(() => {
                messageDiv.remove();
            }, 5000);
        }
    }

    showError(message: string) {
        this.showMessage(message, 'error');
    }

    // Getter methods for accessing data arrays
    getProducts(): Product[] {
        return this.products;
    }

    getServices(): Service[] {
        return this.services;
    }

    getResources(): Resource[] {
        return this.resources;
    }

    // CRUD Form methods needed for integration
    showProductForm(mode: 'create' | 'edit', id?: string): void {
        // This would show the product form modal
        console.log(`Show product form: ${mode}`, id);
    }

    showServiceForm(mode: 'create' | 'edit', id?: string): void {
        // This would show the service form modal
        console.log(`Show service form: ${mode}`, id);
    }

    showResourceForm(mode: 'create' | 'edit', id?: string): void {
        // This would show the resource form modal
        console.log(`Show resource form: ${mode}`, id);
    }

    async deleteProduct(id: string): Promise<void> {
        // Delete product implementation
        console.log(`Delete product: ${id}`);
    }

    async deleteService(id: string): Promise<void> {
        // Delete service implementation
        console.log(`Delete service: ${id}`);
    }

    async deleteResource(id: string): Promise<void> {
        // Delete resource implementation
        console.log(`Delete resource: ${id}`);
    }
}