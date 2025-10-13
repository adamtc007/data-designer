// Metadata-Driven UI Engine - The Missing Critical Component
// This engine interprets Resource Objects and generates dynamic UI

declare const window: any;

interface ResourceDictionaryFile {
    metadata: {
        version: string;
        description: string;
        author: string;
        creation_date: string;
        last_modified: string;
    };
    resources: ResourceObjectFile[];
}

interface ResourceObjectFile {
    resourceName: string;
    description: string;
    version: string;
    category: string;
    ownerTeam: string;
    status: string;
    ui: {
        layout: string;
        groupOrder: string[];
        navigation: any;
    };
    attributes: AttributeObjectFile[];
}

interface AttributeObjectFile {
    name: string;
    dataType: string;
    description: string;
    constraints?: any;
    persistence_locator: {
        system: string;
        entity: string;
        column: string;
    };
    ui: {
        group: string;
        displayOrder: number;
        renderHint: string;
        label: string;
        placeholder?: string;
        isRequired: boolean;
        validation?: any;
    };
    perspectives?: {
        [key: string]: {
            label?: string;
            description?: string;
            aiExample?: string;
        };
    };
}

interface ResolvedAttributeUI {
    group: string;
    display_order: number;
    render_hint: string;
    label: string;
    placeholder?: string;
    is_required: boolean;
    validation?: any;
    description: string;
    ai_example?: string;
}

export class MetadataDrivenEngine {
    private resourceDictionary: ResourceDictionaryFile | null = null;
    private currentUserContext: string = 'default';

    constructor() {
        // Initialize context from backend
        this.initializeUserContext();
    }

    // ===== CORE MISSING FUNCTIONALITY: LOAD AND INTERPRET METADATA =====

    async loadResourceDictionary(filePath?: string): Promise<void> {
        try {
            const path = filePath || './sample-resource-dictionary.json';
            this.resourceDictionary = await window.__TAURI__.invoke('load_resource_dictionary_from_file', {
                filePath: path
            });
            console.log('✅ Resource Dictionary loaded:', this.resourceDictionary);
        } catch (error) {
            console.error('❌ Failed to load resource dictionary:', error);
            // Fallback to sample data
            this.resourceDictionary = this.createSampleDictionary();
        }
    }

    // ===== MISSING CORE ENGINE: RENDERING LOGIC =====

    /**
     * THE CRITICAL MISSING PIECE: Renders a Resource Object into actual UI
     */
    async renderResource(resourceName: string, perspective?: string): Promise<HTMLElement> {
        if (!this.resourceDictionary) {
            await this.loadResourceDictionary();
        }

        const resource = this.findResource(resourceName);
        if (!resource) {
            throw new Error(`Resource '${resourceName}' not found`);
        }

        // Set user context if perspective provided
        if (perspective) {
            await this.setUserContext(perspective);
        }

        console.log(`🎨 Rendering ${resourceName} with layout: ${resource.ui.layout}, perspective: ${this.currentUserContext}`);

        // Create the main resource container
        const container = document.createElement('div');
        container.className = `resource-container resource-${resourceName.toLowerCase()}`;
        container.setAttribute('data-resource', resourceName);
        container.setAttribute('data-perspective', this.currentUserContext);

        // Add resource header
        const header = this.createResourceHeader(resource);
        container.appendChild(header);

        // THE CORE LOGIC: Render based on layout type
        const contentArea = await this.renderResourceLayout(resource);
        container.appendChild(contentArea);

        // Add controls and actions
        const actions = this.createResourceActions(resource);
        container.appendChild(actions);

        return container;
    }

    /**
     * THE MISSING LAYOUT ENGINE: Interprets ui.layout and groupOrder
     */
    private async renderResourceLayout(resource: ResourceObjectFile): Promise<HTMLElement> {
        const { layout, groupOrder } = resource.ui;

        // Group attributes by ui.group and sort by displayOrder
        const groupedAttributes = await this.groupAndSortAttributes(resource.attributes);

        switch (layout) {
            case 'wizard':
                return this.renderWizardLayout(groupedAttributes, groupOrder);
            case 'tabs':
                return this.renderTabsLayout(groupedAttributes, groupOrder);
            case 'vertical-stack':
                return this.renderVerticalStackLayout(groupedAttributes, groupOrder);
            case 'horizontal-grid':
                return this.renderHorizontalGridLayout(groupedAttributes, groupOrder);
            case 'accordion':
                return this.renderAccordionLayout(groupedAttributes, groupOrder);
            default:
                console.warn(`Unknown layout type: ${layout}, falling back to vertical-stack`);
                return this.renderVerticalStackLayout(groupedAttributes, groupOrder);
        }
    }

    /**
     * THE MISSING PERSPECTIVE RESOLVER: Applies context-aware attribute resolution
     */
    private async groupAndSortAttributes(attributes: AttributeObjectFile[]): Promise<{ [group: string]: ResolvedAttributeUI[] }> {
        const grouped: { [group: string]: ResolvedAttributeUI[] } = {};

        for (const attr of attributes) {
            // THE CRITICAL MISSING CALL: Resolve attribute with current perspective
            const resolved = await this.resolveAttributeWithPerspective(attr);

            if (!grouped[resolved.group]) {
                grouped[resolved.group] = [];
            }
            grouped[resolved.group].push(resolved);
        }

        // Sort each group by displayOrder
        for (const group in grouped) {
            grouped[group].sort((a, b) => a.display_order - b.display_order);
        }

        return grouped;
    }

    /**
     * THE MISSING PERSPECTIVE RESOLUTION LOGIC
     */
    private async resolveAttributeWithPerspective(attribute: AttributeObjectFile): Promise<ResolvedAttributeUI> {
        try {
            return await window.__TAURI__.invoke('resolve_attribute_with_perspective', {
                attribute: attribute,
                perspective: this.currentUserContext
            });
        } catch (error) {
            console.error('Failed to resolve attribute with perspective:', error);
            // Fallback to default resolution
            return {
                group: attribute.ui.group,
                display_order: attribute.ui.displayOrder,
                render_hint: attribute.ui.renderHint,
                label: attribute.ui.label,
                placeholder: attribute.ui.placeholder,
                is_required: attribute.ui.isRequired,
                validation: attribute.ui.validation,
                description: attribute.description,
                ai_example: undefined
            };
        }
    }

    // ===== LAYOUT IMPLEMENTATIONS =====

    private renderWizardLayout(groupedAttributes: { [group: string]: ResolvedAttributeUI[] }, groupOrder: string[]): HTMLElement {
        const wizardContainer = document.createElement('div');
        wizardContainer.className = 'wizard-layout';

        // Create wizard steps
        const stepsContainer = document.createElement('div');
        stepsContainer.className = 'wizard-steps';

        // Create step indicators
        const stepIndicators = document.createElement('div');
        stepIndicators.className = 'step-indicators';
        groupOrder.forEach((group, index) => {
            const indicator = document.createElement('div');
            indicator.className = `step-indicator ${index === 0 ? 'active' : ''}`;
            indicator.textContent = `${index + 1}. ${group}`;
            indicator.setAttribute('data-step', index.toString());
            stepIndicators.appendChild(indicator);
        });
        wizardContainer.appendChild(stepIndicators);

        // Create step content panels
        const stepPanels = document.createElement('div');
        stepPanels.className = 'step-panels';

        groupOrder.forEach((group, index) => {
            const panel = document.createElement('div');
            panel.className = `step-panel ${index === 0 ? 'active' : 'hidden'}`;
            panel.setAttribute('data-step', index.toString());

            const groupTitle = document.createElement('h3');
            groupTitle.className = 'group-title';
            groupTitle.textContent = group;
            panel.appendChild(groupTitle);

            const attributeContainer = this.renderAttributeGroup(groupedAttributes[group] || []);
            panel.appendChild(attributeContainer);

            stepPanels.appendChild(panel);
        });
        wizardContainer.appendChild(stepPanels);

        // Add navigation buttons
        const navigation = document.createElement('div');
        navigation.className = 'wizard-navigation';

        const prevBtn = document.createElement('button');
        prevBtn.className = 'btn btn-secondary wizard-prev';
        prevBtn.textContent = '← Previous';
        prevBtn.disabled = true;

        const nextBtn = document.createElement('button');
        nextBtn.className = 'btn btn-primary wizard-next';
        nextBtn.textContent = 'Next →';

        navigation.appendChild(prevBtn);
        navigation.appendChild(nextBtn);
        wizardContainer.appendChild(navigation);

        // Add wizard navigation logic
        this.addWizardNavigation(wizardContainer, groupOrder.length);

        return wizardContainer;
    }

    private renderTabsLayout(groupedAttributes: { [group: string]: ResolvedAttributeUI[] }, groupOrder: string[]): HTMLElement {
        const tabsContainer = document.createElement('div');
        tabsContainer.className = 'tabs-layout';

        // Create tab headers
        const tabHeaders = document.createElement('div');
        tabHeaders.className = 'tab-headers';

        groupOrder.forEach((group, index) => {
            const tabHeader = document.createElement('button');
            tabHeader.className = `tab-header ${index === 0 ? 'active' : ''}`;
            tabHeader.textContent = group;
            tabHeader.setAttribute('data-tab', index.toString());
            tabHeader.addEventListener('click', () => this.switchTab(tabsContainer, index));
            tabHeaders.appendChild(tabHeader);
        });
        tabsContainer.appendChild(tabHeaders);

        // Create tab content panels
        const tabContent = document.createElement('div');
        tabContent.className = 'tab-content';

        groupOrder.forEach((group, index) => {
            const panel = document.createElement('div');
            panel.className = `tab-panel ${index === 0 ? 'active' : 'hidden'}`;
            panel.setAttribute('data-tab', index.toString());

            const attributeContainer = this.renderAttributeGroup(groupedAttributes[group] || []);
            panel.appendChild(attributeContainer);

            tabContent.appendChild(panel);
        });
        tabsContainer.appendChild(tabContent);

        return tabsContainer;
    }

    private renderVerticalStackLayout(groupedAttributes: { [group: string]: ResolvedAttributeUI[] }, groupOrder: string[]): HTMLElement {
        const stackContainer = document.createElement('div');
        stackContainer.className = 'vertical-stack-layout';

        groupOrder.forEach(group => {
            const groupContainer = document.createElement('div');
            groupContainer.className = 'attribute-group';

            const groupTitle = document.createElement('h3');
            groupTitle.className = 'group-title';
            groupTitle.textContent = group;
            groupContainer.appendChild(groupTitle);

            const attributeContainer = this.renderAttributeGroup(groupedAttributes[group] || []);
            groupContainer.appendChild(attributeContainer);

            stackContainer.appendChild(groupContainer);
        });

        return stackContainer;
    }

    private renderHorizontalGridLayout(groupedAttributes: { [group: string]: ResolvedAttributeUI[] }, groupOrder: string[]): HTMLElement {
        const gridContainer = document.createElement('div');
        gridContainer.className = 'horizontal-grid-layout';

        groupOrder.forEach(group => {
            const groupContainer = document.createElement('div');
            groupContainer.className = 'grid-group';

            const groupTitle = document.createElement('h4');
            groupTitle.className = 'group-title';
            groupTitle.textContent = group;
            groupContainer.appendChild(groupTitle);

            const attributeContainer = this.renderAttributeGroup(groupedAttributes[group] || []);
            attributeContainer.className += ' grid-attributes';
            groupContainer.appendChild(attributeContainer);

            gridContainer.appendChild(groupContainer);
        });

        return gridContainer;
    }

    private renderAccordionLayout(groupedAttributes: { [group: string]: ResolvedAttributeUI[] }, groupOrder: string[]): HTMLElement {
        const accordionContainer = document.createElement('div');
        accordionContainer.className = 'accordion-layout';

        groupOrder.forEach((group, index) => {
            const accordionItem = document.createElement('div');
            accordionItem.className = 'accordion-item';

            const accordionHeader = document.createElement('button');
            accordionHeader.className = `accordion-header ${index === 0 ? 'active' : ''}`;
            accordionHeader.textContent = group;
            accordionHeader.addEventListener('click', () => this.toggleAccordion(accordionItem));

            const accordionContent = document.createElement('div');
            accordionContent.className = `accordion-content ${index === 0 ? 'expanded' : 'collapsed'}`;

            const attributeContainer = this.renderAttributeGroup(groupedAttributes[group] || []);
            accordionContent.appendChild(attributeContainer);

            accordionItem.appendChild(accordionHeader);
            accordionItem.appendChild(accordionContent);
            accordionContainer.appendChild(accordionItem);
        });

        return accordionContainer;
    }

    /**
     * THE CRITICAL FIELD RENDERER: Uses renderHint to create appropriate UI components
     */
    private renderAttributeGroup(attributes: ResolvedAttributeUI[]): HTMLElement {
        const container = document.createElement('div');
        container.className = 'attribute-group-container';

        attributes.forEach(attr => {
            const fieldContainer = document.createElement('div');
            fieldContainer.className = 'field-container';
            fieldContainer.setAttribute('data-attribute', attr.label);

            const label = document.createElement('label');
            label.className = 'field-label';
            label.textContent = attr.label + (attr.is_required ? ' *' : '');
            fieldContainer.appendChild(label);

            const field = this.createFieldByRenderHint(attr);
            fieldContainer.appendChild(field);

            if (attr.description) {
                const description = document.createElement('div');
                description.className = 'field-description';
                description.textContent = attr.description;
                fieldContainer.appendChild(description);
            }

            if (attr.ai_example) {
                const aiExample = document.createElement('div');
                aiExample.className = 'field-ai-example';
                aiExample.innerHTML = `<em>AI Context: ${attr.ai_example}</em>`;
                fieldContainer.appendChild(aiExample);
            }

            container.appendChild(fieldContainer);
        });

        return container;
    }

    /**
     * THE MISSING RENDER HINT INTERPRETER
     */
    private createFieldByRenderHint(attr: ResolvedAttributeUI): HTMLElement {
        switch (attr.render_hint) {
            case 'text':
                return this.createTextInput(attr);
            case 'textarea':
                return this.createTextArea(attr);
            case 'number':
                return this.createNumberInput(attr);
            case 'date':
                return this.createDateInput(attr);
            case 'select':
                return this.createSelectInput(attr);
            case 'checkbox':
                return this.createCheckboxInput(attr);
            case 'radio':
                return this.createRadioInput(attr);
            case 'file':
                return this.createFileInput(attr);
            default:
                console.warn(`Unknown render hint: ${attr.render_hint}, falling back to text`);
                return this.createTextInput(attr);
        }
    }

    // ===== FIELD TYPE IMPLEMENTATIONS =====

    private createTextInput(attr: ResolvedAttributeUI): HTMLElement {
        const input = document.createElement('input');
        input.type = 'text';
        input.name = attr.label.toLowerCase().replace(/\s+/g, '_');
        input.className = 'field-input text-input';
        input.placeholder = attr.placeholder || '';
        input.required = attr.is_required;
        return input;
    }

    private createTextArea(attr: ResolvedAttributeUI): HTMLElement {
        const textarea = document.createElement('textarea');
        textarea.name = attr.label.toLowerCase().replace(/\s+/g, '_');
        textarea.className = 'field-input textarea-input';
        textarea.placeholder = attr.placeholder || '';
        textarea.required = attr.is_required;
        textarea.rows = 3;
        return textarea;
    }

    private createNumberInput(attr: ResolvedAttributeUI): HTMLElement {
        const input = document.createElement('input');
        input.type = 'number';
        input.name = attr.label.toLowerCase().replace(/\s+/g, '_');
        input.className = 'field-input number-input';
        input.placeholder = attr.placeholder || '';
        input.required = attr.is_required;
        return input;
    }

    private createDateInput(attr: ResolvedAttributeUI): HTMLElement {
        const input = document.createElement('input');
        input.type = 'date';
        input.name = attr.label.toLowerCase().replace(/\s+/g, '_');
        input.className = 'field-input date-input';
        input.required = attr.is_required;
        return input;
    }

    private createSelectInput(attr: ResolvedAttributeUI): HTMLElement {
        const select = document.createElement('select');
        select.name = attr.label.toLowerCase().replace(/\s+/g, '_');
        select.className = 'field-input select-input';
        select.required = attr.is_required;

        // Add default option
        const defaultOption = document.createElement('option');
        defaultOption.value = '';
        defaultOption.textContent = attr.placeholder || 'Select an option';
        select.appendChild(defaultOption);

        // Add sample options - in real implementation, these would come from constraints
        ['Option 1', 'Option 2', 'Option 3'].forEach(optionText => {
            const option = document.createElement('option');
            option.value = optionText.toLowerCase().replace(/\s+/g, '_');
            option.textContent = optionText;
            select.appendChild(option);
        });

        return select;
    }

    private createCheckboxInput(attr: ResolvedAttributeUI): HTMLElement {
        const container = document.createElement('div');
        container.className = 'checkbox-container';

        const input = document.createElement('input');
        input.type = 'checkbox';
        input.name = attr.label.toLowerCase().replace(/\s+/g, '_');
        input.className = 'field-input checkbox-input';

        const label = document.createElement('label');
        label.textContent = attr.placeholder || 'Check this option';
        label.prepend(input);

        container.appendChild(label);
        return container;
    }

    private createRadioInput(attr: ResolvedAttributeUI): HTMLElement {
        const container = document.createElement('div');
        container.className = 'radio-container';
        const fieldName = attr.label.toLowerCase().replace(/\s+/g, '_');

        ['Option A', 'Option B', 'Option C'].forEach((optionText, index) => {
            const label = document.createElement('label');
            label.className = 'radio-label';

            const input = document.createElement('input');
            input.type = 'radio';
            input.name = fieldName;
            input.value = optionText.toLowerCase().replace(/\s+/g, '_');
            input.className = 'field-input radio-input';
            input.required = attr.is_required;

            label.appendChild(input);
            label.appendChild(document.createTextNode(optionText));
            container.appendChild(label);
        });

        return container;
    }

    private createFileInput(attr: ResolvedAttributeUI): HTMLElement {
        const input = document.createElement('input');
        input.type = 'file';
        input.name = attr.label.toLowerCase().replace(/\s+/g, '_');
        input.className = 'field-input file-input';
        input.required = attr.is_required;
        return input;
    }

    // ===== UTILITY METHODS =====

    private createResourceHeader(resource: ResourceObjectFile): HTMLElement {
        const header = document.createElement('div');
        header.className = 'resource-header';

        const title = document.createElement('h2');
        title.className = 'resource-title';
        title.textContent = resource.resourceName.replace(/([A-Z])/g, ' $1').trim();

        const description = document.createElement('p');
        description.className = 'resource-description';
        description.textContent = resource.description;

        const contextIndicator = document.createElement('div');
        contextIndicator.className = 'context-indicator';
        contextIndicator.textContent = `Context: ${this.currentUserContext}`;

        header.appendChild(title);
        header.appendChild(description);
        header.appendChild(contextIndicator);

        return header;
    }

    private createResourceActions(resource: ResourceObjectFile): HTMLElement {
        const actions = document.createElement('div');
        actions.className = 'resource-actions';

        const submitBtn = document.createElement('button');
        submitBtn.className = 'btn btn-primary';
        submitBtn.textContent = 'Submit';
        submitBtn.addEventListener('click', () => this.handleSubmit(resource));

        const cancelBtn = document.createElement('button');
        cancelBtn.className = 'btn btn-secondary';
        cancelBtn.textContent = 'Cancel';
        cancelBtn.addEventListener('click', () => this.handleCancel());

        actions.appendChild(submitBtn);
        actions.appendChild(cancelBtn);

        return actions;
    }

    // ===== INTERACTION HANDLERS =====

    private addWizardNavigation(container: HTMLElement, totalSteps: number): void {
        const prevBtn = container.querySelector('.wizard-prev') as HTMLButtonElement;
        const nextBtn = container.querySelector('.wizard-next') as HTMLButtonElement;
        let currentStep = 0;

        const updateWizardState = () => {
            // Update step indicators
            container.querySelectorAll('.step-indicator').forEach((indicator, index) => {
                indicator.classList.toggle('active', index === currentStep);
                indicator.classList.toggle('completed', index < currentStep);
            });

            // Update step panels
            container.querySelectorAll('.step-panel').forEach((panel, index) => {
                panel.classList.toggle('active', index === currentStep);
                panel.classList.toggle('hidden', index !== currentStep);
            });

            // Update navigation buttons
            prevBtn.disabled = currentStep === 0;
            nextBtn.textContent = currentStep === totalSteps - 1 ? 'Complete' : 'Next →';
        };

        prevBtn.addEventListener('click', () => {
            if (currentStep > 0) {
                currentStep--;
                updateWizardState();
            }
        });

        nextBtn.addEventListener('click', () => {
            if (currentStep < totalSteps - 1) {
                currentStep++;
                updateWizardState();
            } else {
                // Handle completion
                this.handleWizardComplete(container);
            }
        });
    }

    private switchTab(container: HTMLElement, tabIndex: number): void {
        // Update tab headers
        container.querySelectorAll('.tab-header').forEach((header, index) => {
            header.classList.toggle('active', index === tabIndex);
        });

        // Update tab panels
        container.querySelectorAll('.tab-panel').forEach((panel, index) => {
            panel.classList.toggle('active', index === tabIndex);
            panel.classList.toggle('hidden', index !== tabIndex);
        });
    }

    private toggleAccordion(item: HTMLElement): void {
        const header = item.querySelector('.accordion-header');
        const content = item.querySelector('.accordion-content');

        const isExpanded = content?.classList.contains('expanded');

        if (isExpanded) {
            content?.classList.remove('expanded');
            content?.classList.add('collapsed');
            header?.classList.remove('active');
        } else {
            content?.classList.remove('collapsed');
            content?.classList.add('expanded');
            header?.classList.add('active');
        }
    }

    // ===== CONTEXT MANAGEMENT =====

    private async initializeUserContext(): Promise<void> {
        try {
            const context = await window.__TAURI__.invoke('get_user_context');
            this.currentUserContext = context || 'default';
        } catch (error) {
            console.warn('Failed to initialize user context:', error);
            this.currentUserContext = 'default';
        }
    }

    private async setUserContext(context: string): Promise<void> {
        try {
            await window.__TAURI__.invoke('set_user_context', { context });
            this.currentUserContext = context;
            console.log('✅ User context set to:', context);
        } catch (error) {
            console.error('❌ Failed to set user context:', error);
        }
    }

    // ===== EVENT HANDLERS =====

    private handleSubmit(resource: ResourceObjectFile): void {
        console.log('📋 Submitting resource:', resource.resourceName);
        // Collect form data and submit
        const formData = this.collectFormData();
        console.log('Form data:', formData);
    }

    private handleCancel(): void {
        console.log('❌ Cancelling resource form');
        // Return to editor or close form
        this.hideResourceForm();
    }

    private handleWizardComplete(container: HTMLElement): void {
        console.log('🎉 Wizard completed');
        // Handle wizard completion
        const formData = this.collectFormData();
        console.log('Wizard data:', formData);
    }

    private collectFormData(): any {
        const formData: any = {};
        const inputs = document.querySelectorAll('.field-input');

        inputs.forEach((input: any) => {
            if (input.name) {
                if (input.type === 'checkbox' || input.type === 'radio') {
                    if (input.checked) {
                        formData[input.name] = input.value;
                    }
                } else {
                    formData[input.name] = input.value;
                }
            }
        });

        return formData;
    }

    private hideResourceForm(): void {
        const container = document.getElementById('resource-form-container');
        const editorPanel = document.getElementById('editor-panel');

        if (container) {
            container.classList.add('hidden');
            container.innerHTML = '';
        }

        if (editorPanel) {
            editorPanel.style.display = '';
        }
    }

    // ===== HELPER METHODS =====

    private findResource(resourceName: string): ResourceObjectFile | null {
        if (!this.resourceDictionary) return null;
        return this.resourceDictionary.resources.find(r => r.resourceName === resourceName) || null;
    }

    private createSampleDictionary(): ResourceDictionaryFile {
        return {
            metadata: {
                version: "1.0.0",
                description: "Sample Resource Dictionary",
                author: "System",
                creation_date: new Date().toISOString(),
                last_modified: new Date().toISOString()
            },
            resources: [
                {
                    resourceName: "ClientOnboardingKYC",
                    description: "KYC client onboarding workflow",
                    version: "1.0.0",
                    category: "Compliance",
                    ownerTeam: "KYC Team",
                    status: "active",
                    ui: {
                        layout: "wizard",
                        groupOrder: ["Client Details", "Documents", "Verification"],
                        navigation: {}
                    },
                    attributes: [
                        {
                            name: "client_name",
                            dataType: "String",
                            description: "Client full name",
                            persistence_locator: { system: "db", entity: "clients", column: "name" },
                            ui: {
                                group: "Client Details",
                                displayOrder: 1,
                                renderHint: "text",
                                label: "Client Name",
                                placeholder: "Enter full name",
                                isRequired: true
                            },
                            perspectives: {
                                KYC: {
                                    label: "Legal Name",
                                    description: "Full legal name for compliance",
                                    aiExample: "Enter the complete legal name as shown on identification documents"
                                }
                            }
                        }
                    ]
                }
            ]
        };
    }

    // ===== PUBLIC API =====

    async switchPerspective(perspective: string): Promise<void> {
        await this.setUserContext(perspective);
        // Trigger re-render if needed
        console.log(`🔄 Switched to perspective: ${perspective}`);
    }

    getAvailablePerspectives(): string[] {
        if (!this.resourceDictionary) return ['default'];

        const perspectives = new Set<string>();
        this.resourceDictionary.resources.forEach(resource => {
            resource.attributes.forEach(attr => {
                if (attr.perspectives) {
                    Object.keys(attr.perspectives).forEach(p => perspectives.add(p));
                }
            });
        });

        return Array.from(perspectives);
    }
}

// Export for global use
(window as any).MetadataDrivenEngine = MetadataDrivenEngine;