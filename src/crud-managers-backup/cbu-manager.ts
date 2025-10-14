// CBU Management TypeScript Module
// Handles all Client Business Unit CRUD operations and UI interactions

// Type definitions based on the Rust structs
export interface ClientBusinessUnit {
    id: number;
    cbu_id: string;
    cbu_name: string;
    description?: string;
    primary_entity_id?: string;
    primary_lei?: string;
    domicile_country?: string;
    regulatory_jurisdiction?: string;
    business_type?: string;
    status: string;
    created_date?: string;
    last_review_date?: string;
    next_review_date?: string;
    created_by?: string;
    created_at: string;
    updated_by?: string;
    updated_at: string;
    metadata?: any;
}

export interface CbuSummary {
    id: number;
    cbu_id: string;
    cbu_name: string;
    status: string;
    business_type?: string;
    member_count: number;
}

export interface CbuRole {
    id: number;
    role_code: string;
    role_name: string;
    description?: string;
    role_category?: string;
    display_order: number;
    is_active: boolean;
    created_at: string;
    updated_at: string;
}

export interface CbuMember {
    id: number;
    cbu_id: number;
    role_id: number;
    entity_id: string;
    entity_name?: string;
    entity_lei?: string;
    entity_type?: string;
    jurisdiction?: string;
    ownership_percentage?: number;
    effective_date?: string;
    status: string;
    created_by?: string;
    created_at: string;
    updated_by?: string;
    updated_at: string;
}

export interface CbuMemberDetail {
    member: CbuMember;
    role: CbuRole;
    entity_name?: string;
}

export interface CreateCbuRequest {
    cbu_id: string;
    cbu_name: string;
    description?: string;
    primary_entity_id?: string;
    primary_lei?: string;
    domicile_country?: string;
    regulatory_jurisdiction?: string;
    business_type?: string;
    created_by?: string;
}

export interface AddCbuMemberRequest {
    cbu_id: string;
    role_code: string;
    entity_id: string;
    entity_name?: string;
    entity_lei?: string;
    entity_type?: string;
    jurisdiction?: string;
    ownership_percentage?: number;
    effective_date?: string;
    created_by?: string;
}

export interface ValidationError {
    field: string;
    message: string;
}

// Tauri API interface
declare global {
    interface Window {
        __TAURI_INVOKE__?: (command: string, args?: any) => Promise<any>;
    }
}

export class CbuManager {
    private selectedCbu: ClientBusinessUnit | null = null;
    private cbuList: CbuSummary[] = [];
    private roles: CbuRole[] = [];
    private currentTab: string = 'overview';

    constructor() {
        this.setupEventListeners();
    }

    async initialize(): Promise<void> {
        try {
            await this.loadRoles();
            await this.loadCbuList();
        } catch (error) {
            console.error('Failed to initialize CBU Manager:', error);
            this.showError('Failed to initialize CBU Manager: ' + error);
        }
    }

    private setupEventListeners(): void {
        // Search functionality
        const searchBox = document.getElementById('searchBox') as HTMLInputElement;
        if (searchBox) {
            searchBox.addEventListener('input', (e) => {
                const searchTerm = (e.target as HTMLInputElement).value;
                this.filterCbuList(searchTerm);
            });
        }

        // Form submission
        const createForm = document.getElementById('cbuCreateForm') as HTMLFormElement;
        if (createForm) {
            createForm.addEventListener('submit', (e) => {
                e.preventDefault();
                this.handleCreateCbu();
            });
        }
    }

    // === Tauri API Calls ===

    private async invokeTauri(command: string, args?: any): Promise<any> {
        if (!window.__TAURI_INVOKE__) {
            throw new Error('Tauri API not available');
        }
        return window.__TAURI_INVOKE__(command, args);
    }

    private async loadCbuList(): Promise<void> {
        try {
            this.cbuList = await this.invokeTauri('list_cbus');
            this.renderCbuList();
        } catch (error) {
            console.error('Failed to load CBU list:', error);
            this.showError('Failed to load CBU list: ' + error);
        }
    }

    private async loadRoles(): Promise<void> {
        try {
            this.roles = await this.invokeTauri('get_cbu_roles');
        } catch (error) {
            console.error('Failed to load roles:', error);
            this.showError('Failed to load roles: ' + error);
        }
    }

    private async loadCbuDetails(cbuId: string): Promise<void> {
        try {
            this.selectedCbu = await this.invokeTauri('get_cbu_by_id', { cbu_id: cbuId });
            if (this.selectedCbu) {
                this.showCbuDetails();
                this.renderOverview();
                await this.loadMembers();
            }
        } catch (error) {
            console.error('Failed to load CBU details:', error);
            this.showError('Failed to load CBU details: ' + error);
        }
    }

    private async loadMembers(): Promise<void> {
        if (!this.selectedCbu) return;

        try {
            const members: CbuMemberDetail[] = await this.invokeTauri('get_cbu_members', {
                cbu_id: this.selectedCbu.cbu_id
            });
            this.renderMembers(members);
        } catch (error) {
            console.error('Failed to load members:', error);
            this.showError('Failed to load members: ' + error);
        }
    }

    private async createCbu(request: CreateCbuRequest): Promise<void> {
        try {
            const newCbu = await this.invokeTauri('create_cbu', request);
            this.showSuccess('CBU created successfully!');
            await this.loadCbuList();
            this.selectCbu(newCbu.cbu_id);
            this.cancelCreate();
        } catch (error) {
            console.error('Failed to create CBU:', error);
            this.showError('Failed to create CBU: ' + error);
        }
    }

    private async updateCbu(cbuId: string, updates: Partial<ClientBusinessUnit>): Promise<void> {
        try {
            await this.invokeTauri('update_cbu', {
                cbu_id: cbuId,
                cbu_name: updates.cbu_name,
                description: updates.description,
                business_type: updates.business_type,
                updated_by: updates.updated_by || 'System'
            });
            this.showSuccess('CBU updated successfully!');
            await this.loadCbuList();
            await this.loadCbuDetails(cbuId);
        } catch (error) {
            console.error('Failed to update CBU:', error);
            this.showError('Failed to update CBU: ' + error);
        }
    }

    private async addMember(request: AddCbuMemberRequest): Promise<void> {
        try {
            await this.invokeTauri('add_cbu_member', request);
            this.showSuccess('Member added successfully!');
            await this.loadMembers();
        } catch (error) {
            console.error('Failed to add member:', error);
            this.showError('Failed to add member: ' + error);
        }
    }

    private async removeMember(cbuId: string, entityId: string, roleCode: string): Promise<void> {
        if (!confirm('Are you sure you want to remove this member?')) return;

        try {
            await this.invokeTauri('remove_cbu_member', {
                cbu_id: cbuId,
                entity_id: entityId,
                role_code: roleCode,
                updated_by: 'System'
            });
            this.showSuccess('Member removed successfully!');
            await this.loadMembers();
            await this.loadCbuList(); // Refresh member count
        } catch (error) {
            console.error('Failed to remove member:', error);
            this.showError('Failed to remove member: ' + error);
        }
    }

    private async searchCbus(searchTerm: string): Promise<void> {
        try {
            if (searchTerm.trim() === '') {
                await this.loadCbuList();
            } else {
                this.cbuList = await this.invokeTauri('search_cbus', { search_term: searchTerm });
                this.renderCbuList();
            }
        } catch (error) {
            console.error('Failed to search CBUs:', error);
            this.showError('Failed to search CBUs: ' + error);
        }
    }

    // === UI Rendering ===

    private renderCbuList(): void {
        const listElement = document.getElementById('cbuList');
        if (!listElement) return;

        if (this.cbuList.length === 0) {
            listElement.innerHTML = `
                <div class="empty-state">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
                    </svg>
                    <h4>No CBUs Found</h4>
                    <p>Create your first CBU to get started.</p>
                </div>
            `;
            return;
        }

        const html = this.cbuList.map(cbu => `
            <div class="cbu-item ${this.selectedCbu?.cbu_id === cbu.cbu_id ? 'selected' : ''}"
                 onclick="window.cbuManager.selectCbu('${cbu.cbu_id}')">
                <div class="cbu-name">${this.escapeHtml(cbu.cbu_name)}</div>
                <div class="cbu-id">${this.escapeHtml(cbu.cbu_id)}</div>
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <span class="cbu-status status-${cbu.status.toLowerCase()}">${cbu.status}</span>
                    <span style="font-size: 12px; opacity: 0.7;">${cbu.member_count} members</span>
                </div>
                ${cbu.business_type ? `<div style="font-size: 12px; margin-top: 5px; opacity: 0.8;">${this.escapeHtml(cbu.business_type)}</div>` : ''}
            </div>
        `).join('');

        listElement.innerHTML = html;
    }

    private renderOverview(): void {
        if (!this.selectedCbu) return;

        const overviewElement = document.getElementById('overviewContent');
        if (!overviewElement) return;

        const cbu = this.selectedCbu;
        overviewElement.innerHTML = `
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 30px;">
                <div>
                    <h4>Basic Information</h4>
                    <div style="margin-top: 15px;">
                        <div class="form-group">
                            <strong>CBU ID:</strong> ${this.escapeHtml(cbu.cbu_id)}
                        </div>
                        <div class="form-group">
                            <strong>Name:</strong> ${this.escapeHtml(cbu.cbu_name)}
                        </div>
                        <div class="form-group">
                            <strong>Status:</strong>
                            <span class="cbu-status status-${cbu.status.toLowerCase()}">${cbu.status}</span>
                        </div>
                        ${cbu.description ? `<div class="form-group"><strong>Description:</strong> ${this.escapeHtml(cbu.description)}</div>` : ''}
                        ${cbu.business_type ? `<div class="form-group"><strong>Business Type:</strong> ${this.escapeHtml(cbu.business_type)}</div>` : ''}
                    </div>
                </div>

                <div>
                    <h4>Entity Information</h4>
                    <div style="margin-top: 15px;">
                        ${cbu.primary_entity_id ? `<div class="form-group"><strong>Primary Entity ID:</strong> ${this.escapeHtml(cbu.primary_entity_id)}</div>` : ''}
                        ${cbu.primary_lei ? `<div class="form-group"><strong>Primary LEI:</strong> ${this.escapeHtml(cbu.primary_lei)}</div>` : ''}
                        ${cbu.domicile_country ? `<div class="form-group"><strong>Domicile Country:</strong> ${this.escapeHtml(cbu.domicile_country)}</div>` : ''}
                        ${cbu.regulatory_jurisdiction ? `<div class="form-group"><strong>Regulatory Jurisdiction:</strong> ${this.escapeHtml(cbu.regulatory_jurisdiction)}</div>` : ''}
                    </div>
                </div>
            </div>

            <div style="margin-top: 30px; padding-top: 20px; border-top: 1px solid #e0e6ed;">
                <h4>Audit Information</h4>
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin-top: 15px;">
                    ${cbu.created_by ? `<div><strong>Created By:</strong> ${this.escapeHtml(cbu.created_by)}</div>` : ''}
                    <div><strong>Created At:</strong> ${new Date(cbu.created_at).toLocaleString()}</div>
                    ${cbu.updated_by ? `<div><strong>Updated By:</strong> ${this.escapeHtml(cbu.updated_by)}</div>` : ''}
                    <div><strong>Updated At:</strong> ${new Date(cbu.updated_at).toLocaleString()}</div>
                </div>
            </div>
        `;
    }

    private renderMembers(members: CbuMemberDetail[]): void {
        const membersElement = document.getElementById('membersContent');
        if (!membersElement) return;

        if (members.length === 0) {
            membersElement.innerHTML = `
                <div class="empty-state">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M16 4c0-1.11.89-2 2-2s2 .89 2 2-.89 2-2 2-2-.89-2-2zm4 18v-6h2.5l-2.54-7.63A1.5 1.5 0 0 0 18.54 8H17c-.35 0-.65.22-.76.53L14.46 14H12c-.8 0-1.54.37-2.01.97L8.58 16.5 6.5 15 5 16.5l3.5 3.5 2.38-2.38c.21-.21.33-.5.33-.8V18h1.8l1.33-4h.96l2.54 7.63c.11.31.4.53.76.53H20v-2h-2z"/>
                    </svg>
                    <h4>No Members</h4>
                    <p>Add the first member to this CBU.</p>
                </div>
            `;
            return;
        }

        const html = `
            <div class="members-grid">
                ${members.map(memberDetail => `
                    <div class="member-card">
                        <div class="member-name">${this.escapeHtml(memberDetail.entity_name || memberDetail.member.entity_id)}</div>
                        <div class="member-role">${this.escapeHtml(memberDetail.role.role_name)} (${this.escapeHtml(memberDetail.role.role_code)})</div>
                        <div class="member-entity">Entity ID: ${this.escapeHtml(memberDetail.member.entity_id)}</div>
                        ${memberDetail.member.entity_lei ? `<div class="member-entity">LEI: ${this.escapeHtml(memberDetail.member.entity_lei)}</div>` : ''}
                        ${memberDetail.member.ownership_percentage ? `<div class="member-entity">Ownership: ${memberDetail.member.ownership_percentage}%</div>` : ''}
                        <div style="margin-top: 10px;">
                            <span class="cbu-status status-${memberDetail.member.status.toLowerCase()}">${memberDetail.member.status}</span>
                            <button class="btn btn-danger" style="float: right; font-size: 12px; padding: 6px 12px;"
                                    onclick="window.cbuManager.removeMemberFromUI('${this.selectedCbu?.cbu_id}', '${memberDetail.member.entity_id}', '${memberDetail.role.role_code}')">Remove</button>
                        </div>
                    </div>
                `).join('')}
            </div>
        `;

        membersElement.innerHTML = html;
    }

    private renderEditForm(): void {
        if (!this.selectedCbu) return;

        const editElement = document.getElementById('editForm');
        if (!editElement) return;

        const cbu = this.selectedCbu;
        editElement.innerHTML = `
            <form id="cbuEditForm" onsubmit="event.preventDefault(); window.cbuManager.handleUpdateCbu();">
                <div class="form-row">
                    <div class="form-group">
                        <label for="editCbuName">CBU Name *</label>
                        <input type="text" id="editCbuName" name="cbu_name" value="${this.escapeHtml(cbu.cbu_name)}" required>
                    </div>
                    <div class="form-group">
                        <label for="editBusinessType">Business Type</label>
                        <select id="editBusinessType" name="business_type">
                            <option value="">Select Type</option>
                            <option value="Investment Management" ${cbu.business_type === 'Investment Management' ? 'selected' : ''}>Investment Management</option>
                            <option value="Private Banking" ${cbu.business_type === 'Private Banking' ? 'selected' : ''}>Private Banking</option>
                            <option value="Corporate Banking" ${cbu.business_type === 'Corporate Banking' ? 'selected' : ''}>Corporate Banking</option>
                            <option value="Asset Management" ${cbu.business_type === 'Asset Management' ? 'selected' : ''}>Asset Management</option>
                            <option value="Hedge Fund" ${cbu.business_type === 'Hedge Fund' ? 'selected' : ''}>Hedge Fund</option>
                            <option value="Family Office" ${cbu.business_type === 'Family Office' ? 'selected' : ''}>Family Office</option>
                        </select>
                    </div>
                </div>

                <div class="form-group">
                    <label for="editDescription">Description</label>
                    <textarea id="editDescription" name="description" rows="3">${this.escapeHtml(cbu.description || '')}</textarea>
                </div>

                <div style="margin-top: 30px;">
                    <button type="submit" class="btn btn-success">Save Changes</button>
                    <button type="button" class="btn btn-primary" onclick="window.cbuManager.switchTab('overview')">Cancel</button>
                </div>
            </form>
        `;
    }

    // === Event Handlers ===

    public selectCbu(cbuId: string): void {
        this.loadCbuDetails(cbuId);
    }

    public async handleCreateCbu(): Promise<void> {
        const form = document.getElementById('cbuCreateForm') as HTMLFormElement;
        if (!form) return;

        // Clear previous validation errors
        this.clearValidationErrors();

        const formData = new FormData(form);
        const request: CreateCbuRequest = {
            cbu_id: formData.get('cbu_id') as string,
            cbu_name: formData.get('cbu_name') as string,
            description: formData.get('description') as string || undefined,
            primary_entity_id: formData.get('primary_entity_id') as string || undefined,
            primary_lei: formData.get('primary_lei') as string || undefined,
            domicile_country: formData.get('domicile_country') as string || undefined,
            regulatory_jurisdiction: formData.get('regulatory_jurisdiction') as string || undefined,
            business_type: formData.get('business_type') as string || undefined,
            created_by: formData.get('created_by') as string || undefined
        };

        // Validate the form data
        const validationResult = this.validateCbuRequest(request);
        if (!validationResult.isValid) {
            this.showValidationErrors(validationResult.errors);
            return;
        }

        await this.createCbu(request);
    }

    public async handleUpdateCbu(): Promise<void> {
        if (!this.selectedCbu) return;

        const form = document.getElementById('cbuEditForm') as HTMLFormElement;
        if (!form) return;

        const formData = new FormData(form);
        const updates: Partial<ClientBusinessUnit> = {
            cbu_name: formData.get('cbu_name') as string,
            description: formData.get('description') as string || undefined,
            business_type: formData.get('business_type') as string || undefined,
            updated_by: 'System' // You might want to get this from user context
        };

        await this.updateCbu(this.selectedCbu.cbu_id, updates);
    }

    public removeMemberFromUI(cbuId: string, entityId: string, roleCode: string): void {
        this.removeMember(cbuId, entityId, roleCode);
    }

    public switchTab(tab: string): void {
        this.currentTab = tab;

        // Update tab buttons
        document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
        document.querySelector(`[onclick="switchTab('${tab}')"]`)?.classList.add('active');

        // Update tab content
        document.querySelectorAll('.tab-content').forEach(content => {
            content.classList.remove('active');
        });
        document.getElementById(`${tab}-tab`)?.classList.add('active');

        // Render content based on tab
        if (tab === 'edit') {
            this.renderEditForm();
        } else if (tab === 'members') {
            this.loadMembers();
        }
    }

    public showCreateCbuForm(): void {
        document.getElementById('welcomeScreen')?.setAttribute('style', 'display: none;');
        document.getElementById('cbuDetails')?.setAttribute('style', 'display: none;');
        document.getElementById('createForm')?.setAttribute('style', 'display: block;');
    }

    public showAddMemberForm(): void {
        // This would show a modal or form for adding members
        // Implementation depends on your UI preferences
        alert('Add member form would be implemented here');
    }

    public cancelCreate(): void {
        document.getElementById('createForm')?.setAttribute('style', 'display: none;');
        if (this.selectedCbu) {
            this.showCbuDetails();
        } else {
            document.getElementById('welcomeScreen')?.setAttribute('style', 'display: block;');
        }

        // Reset form
        const form = document.getElementById('cbuCreateForm') as HTMLFormElement;
        form?.reset();
    }

    public goBackToIDE(): void {
        // Navigate back to the main IDE
        window.location.href = 'index.html';
    }

    private filterCbuList(searchTerm: string): void {
        if (searchTerm.trim() === '') {
            this.renderCbuList();
        } else {
            this.searchCbus(searchTerm);
        }
    }

    private showCbuDetails(): void {
        document.getElementById('welcomeScreen')?.setAttribute('style', 'display: none;');
        document.getElementById('createForm')?.setAttribute('style', 'display: none;');
        document.getElementById('cbuDetails')?.setAttribute('style', 'display: block;');
    }

    // === Validation Methods ===

    private validateCbuRequest(request: CreateCbuRequest): { isValid: boolean; errors: ValidationError[] } {
        const errors: ValidationError[] = [];

        // Required field validation
        if (!request.cbu_id || request.cbu_id.trim() === '') {
            errors.push({ field: 'cbu_id', message: 'CBU ID is required' });
        } else if (!/^[A-Z0-9_-]+$/.test(request.cbu_id)) {
            errors.push({ field: 'cbu_id', message: 'CBU ID must contain only uppercase letters, numbers, underscores, and hyphens' });
        } else if (request.cbu_id.length < 3 || request.cbu_id.length > 50) {
            errors.push({ field: 'cbu_id', message: 'CBU ID must be between 3 and 50 characters' });
        }

        if (!request.cbu_name || request.cbu_name.trim() === '') {
            errors.push({ field: 'cbu_name', message: 'CBU Name is required' });
        } else if (request.cbu_name.length < 3 || request.cbu_name.length > 200) {
            errors.push({ field: 'cbu_name', message: 'CBU Name must be between 3 and 200 characters' });
        }

        // Optional field validation
        if (request.primary_lei && !/^[A-Z0-9]{18}[0-9]{2}$/.test(request.primary_lei)) {
            errors.push({ field: 'primary_lei', message: 'LEI must be 20 characters (18 alphanumeric + 2 numeric)' });
        }

        if (request.domicile_country && !/^[A-Z]{2}$/.test(request.domicile_country)) {
            errors.push({ field: 'domicile_country', message: 'Country code must be 2 uppercase letters (ISO 3166-1 alpha-2)' });
        }

        if (request.regulatory_jurisdiction && !/^[A-Z]{2}$/.test(request.regulatory_jurisdiction)) {
            errors.push({ field: 'regulatory_jurisdiction', message: 'Jurisdiction code must be 2 uppercase letters (ISO 3166-1 alpha-2)' });
        }

        if (request.description && request.description.length > 1000) {
            errors.push({ field: 'description', message: 'Description must be less than 1000 characters' });
        }

        return {
            isValid: errors.length === 0,
            errors
        };
    }

    private showValidationErrors(errors: ValidationError[]): void {
        // Remove existing error messages
        document.querySelectorAll('.validation-error').forEach(el => el.remove());

        errors.forEach(error => {
            const field = document.querySelector(`[name="${error.field}"]`) as HTMLInputElement;
            if (field) {
                // Add error styling
                field.style.borderColor = '#dc3545';
                field.style.boxShadow = '0 0 5px rgba(220, 53, 69, 0.3)';

                // Create error message element
                const errorDiv = document.createElement('div');
                errorDiv.className = 'validation-error';
                errorDiv.style.color = '#dc3545';
                errorDiv.style.fontSize = '12px';
                errorDiv.style.marginTop = '5px';
                errorDiv.textContent = error.message;

                // Insert error message after the field
                field.parentNode?.insertBefore(errorDiv, field.nextSibling);

                // Remove error styling on focus
                field.addEventListener('focus', () => {
                    field.style.borderColor = '';
                    field.style.boxShadow = '';
                    errorDiv.remove();
                }, { once: true });
            }
        });

        // Show summary error message
        this.showError(`Please fix ${errors.length} validation error${errors.length > 1 ? 's' : ''} below.`);
    }

    private clearValidationErrors(): void {
        // Remove all validation error messages
        document.querySelectorAll('.validation-error').forEach(el => el.remove());

        // Reset field styling
        document.querySelectorAll('input, select, textarea').forEach(field => {
            const inputField = field as HTMLInputElement;
            inputField.style.borderColor = '';
            inputField.style.boxShadow = '';
        });

        // Remove existing error/success messages
        document.querySelectorAll('.error, .success').forEach(el => el.remove());
    }

    // === Utility Methods ===

    private escapeHtml(text: string): string {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    private showError(message: string): void {
        // Remove existing messages
        document.querySelectorAll('.error, .success').forEach(el => el.remove());

        const errorDiv = document.createElement('div');
        errorDiv.className = 'error';
        errorDiv.textContent = message;

        const contentArea = document.querySelector('.content-area');
        contentArea?.insertBefore(errorDiv, contentArea.firstChild);

        setTimeout(() => errorDiv.remove(), 5000);
    }

    private showSuccess(message: string): void {
        // Remove existing messages
        document.querySelectorAll('.error, .success').forEach(el => el.remove());

        const successDiv = document.createElement('div');
        successDiv.className = 'success';
        successDiv.textContent = message;

        const contentArea = document.querySelector('.content-area');
        contentArea?.insertBefore(successDiv, contentArea.firstChild);

        setTimeout(() => successDiv.remove(), 3000);
    }

    // Methods needed for main IDE integration
    async loadCBUData(): Promise<void> {
        await this.loadCbuList();
    }

    getCBUs(): CbuSummary[] {
        return this.cbuList;
    }

    showCreateCBUForm(): void {
        this.showCreateCbuForm();
    }

    showEditCBUForm(id: string): void {
        // Load and show edit form for the specified CBU
        const cbuId = parseInt(id);
        const cbu = this.cbuList.find(c => c.id === cbuId);
        if (cbu) {
            this.loadCbuDetails(cbu.cbu_id);
        }
    }

    async deleteCBU(id: string): Promise<void> {
        try {
            const cbuId = parseInt(id);
            const cbu = this.cbuList.find(c => c.id === cbuId);
            if (!cbu) {
                throw new Error('CBU not found');
            }

            await this.invokeTauri('delete_cbu', { cbuId: cbu.cbu_id });

            // Refresh the CBU list
            await this.loadCBUData();

            console.log(`CBU ${cbu.cbu_name} deleted successfully`);
        } catch (error) {
            console.error('Failed to delete CBU:', error);
            throw error;
        }
    }
}

// Global instance for HTML onclick handlers
declare global {
    interface Window {
        cbuManager: CbuManager;
    }
}