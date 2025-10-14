import { getSharedDbService } from './shared-db-service.js';
import type { SharedDatabaseService } from './shared-db-service.js';

interface CBU {
    id: number;
    cbu_id: string;
    cbu_name: string;
    status: string;
    description?: string;
    primary_lei?: string;
    domicile_country?: string;
    business_type?: string;
    created_date?: string;
    member_count?: number;
    role_count?: number;
    roles?: string;
    created_at?: string;
    updated_at?: string;
}

export class CbuManager {
    private sharedDbService: SharedDatabaseService | null = null;
    private selectedCbuId: number | null = null;

    constructor() {
        console.log('🏗️ CbuManager constructor called');
    }

    async initialize(): Promise<void> {
        try {
            console.log('🏗️ Initializing CbuManager...');
            this.sharedDbService = getSharedDbService();

            // Wait for the shared database service to be ready
            if (!this.sharedDbService.getConnectionStatus().isConnected) {
                console.log('⏳ Waiting for database connection...');
                await this.sharedDbService.waitForConnection();
            }

            console.log('✅ CbuManager initialized');
            await this.loadCbus();
        } catch (error) {
            console.error('❌ Failed to initialize CbuManager:', error);
            this.showError('Failed to initialize: ' + (error as Error).message);
        }
    }

    async loadCbus(): Promise<void> {
        try {
            console.log('🏢 Loading CBUs...');
            if (!this.sharedDbService) {
                throw new Error('Database service not initialized');
            }

            const cbus = await this.sharedDbService.getCbus();
            console.log('✅ CBUs loaded:', cbus);

            this.updateCbusList(cbus);
        } catch (error) {
            console.error('❌ Failed to load CBUs:', error);
            this.showError('Failed to load CBUs: ' + (error as Error).message);
        }
    }

    updateCbusList(cbus: CBU[]): void {
        const itemsList = document.getElementById('itemsList');
        if (!itemsList) return;

        if (!cbus || cbus.length === 0) {
            itemsList.innerHTML = `
                <div class="empty-state">
                    <h3>No CBUs Found</h3>
                    <p>Create your first CBU to get started.</p>
                </div>
            `;
            return;
        }

        itemsList.innerHTML = cbus.map(cbu => `
            <div class="item" onclick="window.cbuManager?.selectCbu(${cbu.id})" data-cbu-id="${cbu.id}">
                <div class="item-name">${this.escapeHtml(cbu.cbu_name || 'Unnamed CBU')}</div>
                <div class="item-id">ID: ${cbu.cbu_id}</div>
                <div class="item-status status-${cbu.status || 'inactive'}">${cbu.status || 'inactive'}</div>
            </div>
        `).join('');
    }

    selectCbu(cbuId: number): void {
        try {
            console.log('🏢 Selected CBU:', cbuId);
            this.selectedCbuId = cbuId;

            // Update UI to show selected state
            document.querySelectorAll('.item').forEach(item => {
                item.classList.remove('selected');
                if (item.getAttribute('data-cbu-id') === cbuId.toString()) {
                    item.classList.add('selected');
                }
            });

            // Show CBU details
            this.showCbuDetails(cbuId);
        } catch (error) {
            console.error('❌ Failed to select CBU:', error);
            this.showError('Failed to select CBU: ' + (error as Error).message);
        }
    }

    async showCbuDetails(cbuId: number): Promise<void> {
        try {
            if (!this.sharedDbService) {
                throw new Error('Database service not initialized');
            }

            // Hide welcome screen and show details
            const welcomeScreen = document.getElementById('welcomeScreen');
            const cbuDetails = document.getElementById('cbuDetails');

            if (welcomeScreen) welcomeScreen.style.display = 'none';
            if (cbuDetails) cbuDetails.style.display = 'block';

            // Load and display CBU details
            const cbus = await this.sharedDbService.getCbus();
            const cbu = cbus.find((c: CBU) => c.id === cbuId);

            if (cbu) {
                this.renderCbuOverview(cbu);
            }
        } catch (error) {
            console.error('❌ Failed to show CBU details:', error);
            this.showError('Failed to load CBU details: ' + (error as Error).message);
        }
    }

    renderCbuOverview(cbu: CBU): void {
        const overviewContent = document.getElementById('overviewContent');
        if (!overviewContent) return;

        overviewContent.innerHTML = `
            <div class="form-group">
                <label>CBU Name</label>
                <div class="readonly-field">${this.escapeHtml(cbu.cbu_name || 'N/A')}</div>
            </div>
            <div class="form-row">
                <div class="form-group">
                    <label>CBU ID</label>
                    <div class="readonly-field">${cbu.cbu_id}</div>
                </div>
                <div class="form-group">
                    <label>Status</label>
                    <div class="readonly-field">
                        <span class="status-badge status-${cbu.status || 'inactive'}">${cbu.status || 'inactive'}</span>
                    </div>
                </div>
            </div>
            <div class="form-group">
                <label>Description</label>
                <div class="readonly-field">${this.escapeHtml(cbu.description || 'No description available')}</div>
            </div>
            <div class="form-row">
                <div class="form-group">
                    <label>Business Type</label>
                    <div class="readonly-field">${this.escapeHtml(cbu.business_type || 'Standard')}</div>
                </div>
                <div class="form-group">
                    <label>Primary LEI</label>
                    <div class="readonly-field">${this.escapeHtml(cbu.primary_lei || 'N/A')}</div>
                </div>
            </div>
            <div class="form-row">
                <div class="form-group">
                    <label>Members</label>
                    <div class="readonly-field">${cbu.member_count || 0}</div>
                </div>
                <div class="form-group">
                    <label>Roles</label>
                    <div class="readonly-field">${cbu.role_count || 0}</div>
                </div>
            </div>
            <div class="form-row">
                <div class="form-group">
                    <label>Created</label>
                    <div class="readonly-field">${cbu.created_date ? new Date(cbu.created_date).toLocaleDateString() : 'N/A'}</div>
                </div>
                <div class="form-group">
                    <label>Updated</label>
                    <div class="readonly-field">${cbu.updated_at ? new Date(cbu.updated_at).toLocaleDateString() : 'N/A'}</div>
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

    showCreateCbuForm(): void {
        console.log('📝 Showing create CBU form...');
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
        const tabIndex = tabName === 'overview' ? 1 : tabName === 'members' ? 2 : 3;
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
        CbuManager: typeof CbuManager;
        cbuManager?: CbuManager;
    }
}

window.CbuManager = CbuManager;