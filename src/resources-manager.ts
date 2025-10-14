import { getSharedDbService } from './shared-db-service.js';
import type { SharedDatabaseService } from './shared-db-service.js';

interface Resource {
    id: number;
    name: string;
    status: string;
    description?: string;
    type?: string;
    category?: string;
    created_at?: string;
    updated_at?: string;
}

export class ResourcesManager {
    private sharedDbService: SharedDatabaseService | null = null;
    private selectedResourceId: number | null = null;

    constructor() {
        console.log('🏗️ ResourcesManager constructor called');
    }

    async initialize(): Promise<void> {
        try {
            console.log('🏗️ Initializing ResourcesManager...');
            this.sharedDbService = getSharedDbService();

            // Wait for the shared database service to be ready
            if (!this.sharedDbService.getConnectionStatus().isConnected) {
                console.log('⏳ Waiting for database connection...');
                await this.sharedDbService.waitForConnection();
            }

            console.log('✅ ResourcesManager initialized');
            await this.loadResources();
        } catch (error) {
            console.error('❌ Failed to initialize ResourcesManager:', error);
            this.showError('Failed to initialize: ' + (error as Error).message);
        }
    }

    async loadResources(): Promise<void> {
        try {
            console.log('📚 Loading resources...');
            if (!this.sharedDbService) {
                throw new Error('Database service not initialized');
            }

            const resources = await this.sharedDbService.getResources();
            console.log('✅ Resources loaded:', resources);

            this.updateResourcesList(resources);
        } catch (error) {
            console.error('❌ Failed to load resources:', error);
            this.showError('Failed to load resources: ' + (error as Error).message);
        }
    }

    updateResourcesList(resources: Resource[]): void {
        const itemsList = document.getElementById('itemsList');
        if (!itemsList) return;

        if (!resources || resources.length === 0) {
            itemsList.innerHTML = `
                <div class="empty-state">
                    <h3>No Resources Found</h3>
                    <p>Create your first resource to get started.</p>
                </div>
            `;
            return;
        }

        itemsList.innerHTML = resources.map(resource => `
            <div class="item" onclick="window.resourcesManager?.selectResource(${resource.id})" data-resource-id="${resource.id}">
                <div class="item-name">${this.escapeHtml(resource.name || 'Unnamed Resource')}</div>
                <div class="item-id">ID: ${resource.id}</div>
                <div class="item-status status-${resource.status || 'inactive'}">${resource.status || 'inactive'}</div>
            </div>
        `).join('');
    }

    selectResource(resourceId: number): void {
        try {
            console.log('📚 Selected resource:', resourceId);
            this.selectedResourceId = resourceId;

            // Update UI to show selected state
            document.querySelectorAll('.item').forEach(item => {
                item.classList.remove('selected');
                if (item.getAttribute('data-resource-id') === resourceId.toString()) {
                    item.classList.add('selected');
                }
            });

            // Show resource details
            this.showResourceDetails(resourceId);
        } catch (error) {
            console.error('❌ Failed to select resource:', error);
            this.showError('Failed to select resource: ' + (error as Error).message);
        }
    }

    async showResourceDetails(resourceId: number): Promise<void> {
        try {
            if (!this.sharedDbService) {
                throw new Error('Database service not initialized');
            }

            // Hide welcome screen and show details
            const welcomeScreen = document.getElementById('welcomeScreen');
            const itemDetails = document.getElementById('itemDetails');

            if (welcomeScreen) welcomeScreen.style.display = 'none';
            if (itemDetails) itemDetails.style.display = 'block';

            // Load and display resource details
            const resources = await this.sharedDbService.getResources();
            const resource = resources.find((r: Resource) => r.id === resourceId);

            if (resource) {
                this.renderResourceOverview(resource);
            }
        } catch (error) {
            console.error('❌ Failed to show resource details:', error);
            this.showError('Failed to load resource details: ' + (error as Error).message);
        }
    }

    renderResourceOverview(resource: Resource): void {
        const overviewContent = document.getElementById('overviewContent');
        if (!overviewContent) return;

        overviewContent.innerHTML = `
            <div class="form-group">
                <label>Resource Name</label>
                <div class="readonly-field">${this.escapeHtml(resource.name || 'N/A')}</div>
            </div>
            <div class="form-row">
                <div class="form-group">
                    <label>Resource ID</label>
                    <div class="readonly-field">${resource.id}</div>
                </div>
                <div class="form-group">
                    <label>Status</label>
                    <div class="readonly-field">
                        <span class="status-badge status-${resource.status || 'inactive'}">${resource.status || 'inactive'}</span>
                    </div>
                </div>
            </div>
            <div class="form-group">
                <label>Description</label>
                <div class="readonly-field">${this.escapeHtml(resource.description || 'No description available')}</div>
            </div>
            <div class="form-row">
                <div class="form-group">
                    <label>Type</label>
                    <div class="readonly-field">${this.escapeHtml(resource.type || 'General')}</div>
                </div>
                <div class="form-group">
                    <label>Category</label>
                    <div class="readonly-field">${this.escapeHtml(resource.category || 'Uncategorized')}</div>
                </div>
            </div>
            <div class="form-row">
                <div class="form-group">
                    <label>Created</label>
                    <div class="readonly-field">${resource.created_at ? new Date(resource.created_at).toLocaleDateString() : 'N/A'}</div>
                </div>
                <div class="form-group">
                    <label>Updated</label>
                    <div class="readonly-field">${resource.updated_at ? new Date(resource.updated_at).toLocaleDateString() : 'N/A'}</div>
                </div>
            </div>
        `;
    }

    showError(message: string): void {
        const itemsList = document.getElementById('itemsList');
        if (itemsList) {
            itemsList.innerHTML = `
                <div class="error">
                    <strong>Error:</strong> ${this.escapeHtml(message)}
                </div>
            `;
        }
    }

    goBackToIDE(): void {
        window.location.href = 'home.html';
    }

    showCreateResourceForm(): void {
        console.log('📝 Showing create resource form...');
        const createForm = document.getElementById('createForm');
        const welcomeScreen = document.getElementById('welcomeScreen');

        if (createForm) createForm.style.display = 'block';
        if (welcomeScreen) welcomeScreen.style.display = 'none';

        // TODO: Implement create form
    }

    switchTab(tabName: string): void {
        // Hide all tabs
        document.querySelectorAll('.tab').forEach(tab => tab.classList.remove('active'));
        document.querySelectorAll('.tab-content').forEach(content => content.classList.remove('active'));

        // Show selected tab
        const tabIndex = tabName === 'overview' ? 1 : tabName === 'properties' ? 2 : 3;
        const tabElement = document.querySelector(`.tab:nth-child(${tabIndex})`);
        const contentElement = document.getElementById(`${tabName}-tab`);

        if (tabElement) tabElement.classList.add('active');
        if (contentElement) contentElement.classList.add('active');
    }

    cancelCreate(): void {
        const createForm = document.getElementById('createForm');
        const welcomeScreen = document.getElementById('welcomeScreen');

        if (createForm) createForm.style.display = 'none';
        if (welcomeScreen) welcomeScreen.style.display = 'block';
    }

    private escapeHtml(text: string): string {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Make it available globally for the HTML onclick handlers
declare global {
    interface Window {
        ResourcesManager: typeof ResourcesManager;
        resourcesManager?: ResourcesManager;
    }
}

window.ResourcesManager = ResourcesManager;