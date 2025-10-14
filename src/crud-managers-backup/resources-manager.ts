import { invoke } from '@tauri-apps/api/core';

// Types based on the Rust backend structures
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

export interface CreateResourceRequest {
    resource_id: string;
    resource_name: string;
    resource_type: string;
    description?: string;
    location?: string;
    created_by?: string;
}

export class ResourcesManager {
    private selectedResource: Resource | null = null;
    private resources: Resource[] = [];

    async initialize() {
        console.log('🚀 Initializing Resources Manager');
        await this.loadResourcesData();
        this.setupEventListeners();
    }

    async loadResourcesData() {
        try {
            const resources = await invoke<Resource[]>('list_resources', { resourceType: null });
            this.resources = resources || [];
            console.log('✅ Loaded resources:', this.resources.length);
            this.renderResourcesList();
        } catch (error) {
            console.error('❌ Error loading resources:', error);
            this.showError('Failed to load resources: ' + (error as Error).message);
        }
    }

    renderResourcesList() {
        const container = document.getElementById('resourcesList');
        if (!container) return;

        let html = '';

        if (this.resources.length > 0) {
            this.resources.forEach(resource => {
                html += `
                    <div class="resource-item" onclick="selectResource('${resource.id}')">
                        <div class="resource-name">${resource.resource_name}</div>
                        <div class="resource-id">ID: ${resource.resource_id}</div>
                        <div class="resource-type">${resource.resource_type}</div>
                        <div class="resource-status ${resource.status === 'active' ? 'status-active' : 'status-inactive'}">
                            ${resource.status}
                        </div>
                    </div>
                `;
            });
        } else {
            html = '<div class="empty-state">No resources found</div>';
        }

        container.innerHTML = html;

        // Make selectResource globally available
        (window as any).selectResource = (id: string) => {
            this.selectResource(parseInt(id));
        };
    }

    async selectResource(id: number) {
        this.selectedResource = this.resources.find(r => r.id === id) || null;

        if (this.selectedResource) {
            // Update UI selection
            document.querySelectorAll('.resource-item').forEach(el => el.classList.remove('selected'));
            if (event && event.currentTarget) {
                (event.currentTarget as HTMLElement).classList.add('selected');
            }

            // Show details
            this.showResourceDetails();
        }
    }

    showResourceDetails() {
        const welcomeScreen = document.getElementById('welcomeScreen');
        const resourceDetails = document.getElementById('resourceDetails');

        if (welcomeScreen) welcomeScreen.style.display = 'none';
        if (resourceDetails) resourceDetails.style.display = 'block';

        this.renderOverviewTab();
        this.switchTab('overview');
    }

    renderOverviewTab() {
        const container = document.getElementById('overviewContent');
        if (!container || !this.selectedResource) return;

        const resource = this.selectedResource;
        let html = `
            <div class="form-group">
                <h3>🛠️ ${resource.resource_name}</h3>
                <div class="form-row" style="margin-top: 20px;">
                    <div><strong>Resource ID:</strong> ${resource.resource_id}</div>
                    <div><strong>Type:</strong> ${resource.resource_type}</div>
                </div>
                <div class="form-row">
                    <div><strong>Status:</strong> <span class="resource-status ${resource.status === 'active' ? 'status-active' : 'status-inactive'}">${resource.status}</span></div>
                    <div><strong>Location:</strong> ${resource.location || 'Not specified'}</div>
                </div>
                ${resource.description ? `<div style="margin-top: 15px;"><strong>Description:</strong><br>${resource.description}</div>` : ''}
                <div style="margin-top: 15px;">
                    <strong>Created:</strong> ${new Date(resource.created_at).toLocaleString()}
                    ${resource.created_by ? ` by ${resource.created_by}` : ''}
                </div>
                <div style="margin-top: 5px;">
                    <strong>Updated:</strong> ${new Date(resource.updated_at).toLocaleString()}
                    ${resource.updated_by ? ` by ${resource.updated_by}` : ''}
                </div>
            </div>
        `;

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
        if (tabName === 'configuration') {
            this.renderConfigurationTab();
        } else if (tabName === 'edit') {
            this.renderEditTab();
        }
    }

    renderConfigurationTab() {
        const container = document.getElementById('configurationContent');
        if (!container || !this.selectedResource) return;

        const resource = this.selectedResource;
        let html = '<h3>Resource Configuration</h3>';

        // Show configuration if available
        if (resource.configuration) {
            html += `
                <div class="form-group">
                    <label>Configuration Data:</label>
                    <div class="config-display">
                        <pre>${JSON.stringify(resource.configuration, null, 2)}</pre>
                    </div>
                </div>
            `;
        } else {
            html += '<p>No configuration data available for this resource.</p>';
        }

        // Show access policies if available
        if (resource.access_policies) {
            html += `
                <div class="form-group">
                    <label>Access Policies:</label>
                    <div class="config-display">
                        <pre>${JSON.stringify(resource.access_policies, null, 2)}</pre>
                    </div>
                </div>
            `;
        }

        // Add configuration form for editing
        html += `
            <div class="form-group" style="margin-top: 30px;">
                <h4>Update Configuration</h4>
                <form id="configForm">
                    <div class="form-group">
                        <label for="configData">Configuration JSON:</label>
                        <textarea id="configData" name="configuration" rows="10" placeholder="Enter JSON configuration...">${resource.configuration ? JSON.stringify(resource.configuration, null, 2) : ''}</textarea>
                    </div>
                    <div class="form-group">
                        <label for="accessPolicies">Access Policies JSON:</label>
                        <textarea id="accessPolicies" name="access_policies" rows="6" placeholder="Enter access policies JSON...">${resource.access_policies ? JSON.stringify(resource.access_policies, null, 2) : ''}</textarea>
                    </div>
                    <button type="submit" class="btn btn-success">Update Configuration</button>
                </form>
            </div>
        `;

        container.innerHTML = html;

        // Add event listener for config form
        const form = document.getElementById('configForm') as HTMLFormElement;
        if (form) {
            form.addEventListener('submit', (e) => this.handleUpdateConfiguration(e));
        }
    }

    async handleUpdateConfiguration(event: Event) {
        event.preventDefault();
        this.showMessage('Configuration update functionality not yet implemented', 'info');
    }

    renderEditTab() {
        const container = document.getElementById('editForm');
        if (!container || !this.selectedResource) return;

        const resource = this.selectedResource;
        let html = `
            <form id="editResourceForm">
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
                            <option value="Cache" ${resource.resource_type === 'Cache' ? 'selected' : ''}>Cache</option>
                            <option value="External Service" ${resource.resource_type === 'External Service' ? 'selected' : ''}>External Service</option>
                            <option value="Storage" ${resource.resource_type === 'Storage' ? 'selected' : ''}>Storage</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label for="editLocation">Location</label>
                        <input type="text" id="editLocation" name="location" value="${resource.location || ''}" placeholder="e.g., us-east-1, localhost:5432">
                    </div>
                </div>
                <div class="form-group">
                    <label for="editResourceDescription">Description</label>
                    <textarea id="editResourceDescription" name="description" rows="3" placeholder="Describe this resource...">${resource.description || ''}</textarea>
                </div>
                <div class="form-row">
                    <div class="form-group">
                        <label for="editStatus">Status</label>
                        <select id="editStatus" name="status">
                            <option value="active" ${resource.status === 'active' ? 'selected' : ''}>Active</option>
                            <option value="inactive" ${resource.status === 'inactive' ? 'selected' : ''}>Inactive</option>
                            <option value="maintenance" ${resource.status === 'maintenance' ? 'selected' : ''}>Maintenance</option>
                            <option value="deprecated" ${resource.status === 'deprecated' ? 'selected' : ''}>Deprecated</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label for="editCreatedBy">Updated By</label>
                        <input type="text" id="editCreatedBy" name="updated_by" value="system" placeholder="Updated by">
                    </div>
                </div>
                <div style="margin-top: 30px;">
                    <button type="submit" class="btn btn-success">Update Resource</button>
                    <button type="button" class="btn btn-danger" onclick="deleteCurrentResource()">Delete</button>
                </div>
            </form>
        `;

        container.innerHTML = html;

        // Add event listeners
        const form = document.getElementById('editResourceForm') as HTMLFormElement;
        if (form) {
            form.addEventListener('submit', (e) => this.handleUpdateResource(e));
        }

        // Make deleteCurrentResource globally available
        (window as any).deleteCurrentResource = () => this.deleteCurrentResource();
    }

    async handleUpdateResource(event: Event) {
        event.preventDefault();
        // Implementation would go here - update the resource
        this.showMessage('Update functionality not yet implemented', 'info');
    }

    async deleteCurrentResource() {
        if (!this.selectedResource) return;

        if (confirm(`Are you sure you want to delete the resource "${this.selectedResource.resource_name}"?`)) {
            try {
                await invoke('delete_resource', { resourceId: this.selectedResource.id });
                this.resources = this.resources.filter(r => r.id !== this.selectedResource?.id);
                this.selectedResource = null;
                this.renderResourcesList();
                this.showWelcomeScreen();
                this.showMessage('Resource deleted successfully', 'success');
            } catch (error) {
                console.error('Error deleting resource:', error);
                this.showError('Failed to delete resource: ' + (error as Error).message);
            }
        }
    }

    showCreateResourceForm() {
        const welcomeScreen = document.getElementById('welcomeScreen');
        const resourceDetails = document.getElementById('resourceDetails');
        const createForm = document.getElementById('createForm');

        if (welcomeScreen) welcomeScreen.style.display = 'none';
        if (resourceDetails) resourceDetails.style.display = 'none';
        if (createForm) createForm.style.display = 'block';

        this.renderCreateForm();
    }

    renderCreateForm() {
        const container = document.getElementById('createForm');
        if (!container) return;

        const html = `
            <h3>Create New Resource</h3>
            <form id="resourceCreateForm">
                <div class="form-row">
                    <div class="form-group">
                        <label for="resourceId">Resource ID *</label>
                        <input type="text" id="resourceId" name="resource_id" required placeholder="e.g., db-primary, api-gateway">
                    </div>
                    <div class="form-group">
                        <label for="resourceName">Resource Name *</label>
                        <input type="text" id="resourceName" name="resource_name" required placeholder="e.g., Primary Database">
                    </div>
                </div>
                <div class="form-row">
                    <div class="form-group">
                        <label for="resourceType">Resource Type *</label>
                        <select id="resourceType" name="resource_type" required>
                            <option value="">Select Type</option>
                            <option value="Database">Database</option>
                            <option value="API Endpoint">API Endpoint</option>
                            <option value="Message Queue">Message Queue</option>
                            <option value="File System">File System</option>
                            <option value="Cache">Cache</option>
                            <option value="External Service">External Service</option>
                            <option value="Storage">Storage</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label for="location">Location</label>
                        <input type="text" id="location" name="location" placeholder="e.g., us-east-1, localhost:5432">
                    </div>
                </div>
                <div class="form-group">
                    <label for="description">Description</label>
                    <textarea id="description" name="description" rows="3" placeholder="Describe this resource and its purpose..."></textarea>
                </div>
                <div class="form-group">
                    <label for="createdBy">Created By</label>
                    <input type="text" id="createdBy" name="created_by" value="system" placeholder="Created by">
                </div>
                <div style="margin-top: 30px;">
                    <button type="submit" class="btn btn-success">Create Resource</button>
                    <button type="button" class="btn btn-primary" onclick="cancelCreate()">Cancel</button>
                </div>
            </form>
        `;

        container.innerHTML = html;

        const form = document.getElementById('resourceCreateForm') as HTMLFormElement;
        if (form) {
            form.addEventListener('submit', (e) => this.handleCreateResource(e));
        }
    }

    async handleCreateResource(event: Event) {
        event.preventDefault();
        const form = event.target as HTMLFormElement;
        const formData = new FormData(form);

        const request: CreateResourceRequest = {
            resource_id: formData.get('resource_id') as string,
            resource_name: formData.get('resource_name') as string,
            resource_type: formData.get('resource_type') as string,
            description: formData.get('description') as string || undefined,
            location: formData.get('location') as string || undefined,
            created_by: formData.get('created_by') as string || undefined
        };

        try {
            const resource = await invoke<Resource>('create_resource', { request });
            this.resources.push(resource);
            this.renderResourcesList();
            this.cancelCreate();
            this.showMessage(`Resource "${resource.resource_name}" created successfully!`, 'success');
        } catch (error) {
            console.error('Error creating resource:', error);
            this.showError('Failed to create resource: ' + (error as Error).message);
        }
    }

    cancelCreate() {
        const welcomeScreen = document.getElementById('welcomeScreen');
        const resourceDetails = document.getElementById('resourceDetails');
        const createForm = document.getElementById('createForm');

        if (createForm) createForm.style.display = 'none';

        if (this.selectedResource) {
            if (resourceDetails) resourceDetails.style.display = 'block';
        } else {
            this.showWelcomeScreen();
        }
    }

    showWelcomeScreen() {
        const welcomeScreen = document.getElementById('welcomeScreen');
        const resourceDetails = document.getElementById('resourceDetails');

        if (welcomeScreen) welcomeScreen.style.display = 'block';
        if (resourceDetails) resourceDetails.style.display = 'none';
    }

    goBackToIDE() {
        window.location.href = 'index.html';
    }

    setupEventListeners() {
        const searchBox = document.getElementById('searchBox') as HTMLInputElement;
        if (searchBox) {
            searchBox.addEventListener('input', (e) => {
                const target = e.target as HTMLInputElement;
                this.filterResources(target.value);
            });
        }
    }

    filterResources(searchTerm: string) {
        const items = document.querySelectorAll('.resource-item');
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
}