// Configuration-Driven UI Renderer v2.0
// Dynamically renders forms based on ResourceDictionary JSON schema

import {
    ResourceDictionary,
    ResourceObject,
    AttributeObject,
    UIRenderContext,
    PerspectiveContext,
    AttributeResolution,
    RendererConfig,
    ValidationResult,
    LayoutType,
    RenderHint
} from './data-dictionary-types';

export class ConfigDrivenRenderer {
    private dictionary: ResourceDictionary;
    private config: RendererConfig;
    private context: UIRenderContext;

    constructor(dictionary: ResourceDictionary, config: RendererConfig) {
        this.dictionary = dictionary;
        this.config = config;
        this.context = this.initializeContext();
    }

    // ===== CORE RENDERING METHODS =====

    /**
     * Renders a complete resource UI based on configuration
     */
    public renderResource(resourceName: string, perspective: string = 'default'): HTMLElement {
        const resource = this.findResource(resourceName);
        if (!resource) {
            throw new Error(`Resource '${resourceName}' not found in dictionary`);
        }

        this.context.perspective = {
            activePerspective: perspective,
            availablePerspectives: this.getAvailablePerspectives(resource),
            resourceName: resourceName
        };

        return this.renderResourceLayout(resource);
    }

    /**
     * Renders the main layout container based on resource.ui.layout
     */
    private renderResourceLayout(resource: ResourceObject): HTMLElement {
        const container = this.createElement('div', ['resource-container', `layout-${resource.ui.layout}`]);

        // Add perspective switcher if multiple perspectives available
        if (this.context.perspective.availablePerspectives.length > 1) {
            container.appendChild(this.renderPerspectiveSwitcher());
        }

        // Render layout based on configuration
        switch (resource.ui.layout) {
            case 'wizard':
                return this.renderWizardLayout(resource, container);
            case 'tabs':
                return this.renderTabsLayout(resource, container);
            case 'vertical-stack':
                return this.renderVerticalStackLayout(resource, container);
            case 'horizontal-grid':
                return this.renderHorizontalGridLayout(resource, container);
            case 'accordion':
                return this.renderAccordionLayout(resource, container);
            default:
                return this.renderVerticalStackLayout(resource, container);
        }
    }

    // ===== LAYOUT RENDERERS =====

    private renderWizardLayout(resource: ResourceObject, container: HTMLElement): HTMLElement {
        const wizardContainer = this.createElement('div', ['wizard-container']);
        const groups = this.groupAttributesByUI(resource);

        // Progress indicator
        if (resource.ui.navigation?.showProgress) {
            wizardContainer.appendChild(this.renderProgressIndicator(groups.length));
        }

        // Current step content
        const stepContainer = this.createElement('div', ['wizard-step-container']);
        const currentGroup = groups[this.context.currentStep || 0];

        if (currentGroup) {
            stepContainer.appendChild(this.renderGroup(currentGroup.name, currentGroup.attributes, 'wizard'));
        }

        // Navigation buttons
        wizardContainer.appendChild(this.renderWizardNavigation(resource, groups));

        container.appendChild(wizardContainer);
        return container;
    }

    private renderTabsLayout(resource: ResourceObject, container: HTMLElement): HTMLElement {
        const tabsContainer = this.createElement('div', ['tabs-container']);
        const groups = this.groupAttributesByUI(resource);

        // Tab headers
        const tabHeaders = this.createElement('div', ['tab-headers']);
        groups.forEach((group, index) => {
            const tabHeader = this.createElement('button', ['tab-header', index === 0 ? 'active' : '']);
            tabHeader.textContent = group.name;
            tabHeader.onclick = () => this.switchTab(index);
            tabHeaders.appendChild(tabHeader);
        });

        // Tab content
        const tabContent = this.createElement('div', ['tab-content']);
        groups.forEach((group, index) => {
            const tabPane = this.createElement('div', ['tab-pane', index === 0 ? 'active' : '']);
            tabPane.appendChild(this.renderGroup(group.name, group.attributes, 'tabs'));
            tabContent.appendChild(tabPane);
        });

        tabsContainer.appendChild(tabHeaders);
        tabsContainer.appendChild(tabContent);
        container.appendChild(tabsContainer);
        return container;
    }

    private renderVerticalStackLayout(resource: ResourceObject, container: HTMLElement): HTMLElement {
        const stackContainer = this.createElement('div', ['vertical-stack']);
        const groups = this.groupAttributesByUI(resource);

        groups.forEach(group => {
            stackContainer.appendChild(this.renderGroup(group.name, group.attributes, 'vertical-stack'));
        });

        container.appendChild(stackContainer);
        return container;
    }

    private renderHorizontalGridLayout(resource: ResourceObject, container: HTMLElement): HTMLElement {
        const gridContainer = this.createElement('div', ['horizontal-grid']);
        const groups = this.groupAttributesByUI(resource);

        groups.forEach(group => {
            const gridItem = this.createElement('div', ['grid-item']);
            gridItem.appendChild(this.renderGroup(group.name, group.attributes, 'horizontal-grid'));
            gridContainer.appendChild(gridItem);
        });

        container.appendChild(gridContainer);
        return container;
    }

    private renderAccordionLayout(resource: ResourceObject, container: HTMLElement): HTMLElement {
        const accordionContainer = this.createElement('div', ['accordion-container']);
        const groups = this.groupAttributesByUI(resource);

        groups.forEach((group, index) => {
            const accordionItem = this.createElement('div', ['accordion-item']);

            // Header
            const header = this.createElement('div', ['accordion-header']);
            header.textContent = group.name;
            header.onclick = () => this.toggleAccordion(index);

            // Content
            const content = this.createElement('div', ['accordion-content', index === 0 ? 'open' : '']);
            content.appendChild(this.renderGroup(group.name, group.attributes, 'accordion'));

            accordionItem.appendChild(header);
            accordionItem.appendChild(content);
            accordionContainer.appendChild(accordionItem);
        });

        container.appendChild(accordionContainer);
        return container;
    }

    // ===== GROUP AND FIELD RENDERING =====

    private renderGroup(groupName: string, attributes: AttributeObject[], layoutContext: string): HTMLElement {
        const groupContainer = this.createElement('div', ['attribute-group', `group-${layoutContext}`]);

        // Group header
        const groupHeader = this.createElement('h3', ['group-header']);
        groupHeader.textContent = groupName;
        groupContainer.appendChild(groupHeader);

        // Group fields
        const fieldsContainer = this.createElement('div', ['group-fields']);

        attributes.forEach(attribute => {
            const resolvedAttribute = this.resolveAttributeForPerspective(attribute);
            fieldsContainer.appendChild(this.renderField(resolvedAttribute));
        });

        groupContainer.appendChild(fieldsContainer);
        return groupContainer;
    }

    private renderField(resolved: AttributeResolution): HTMLElement {
        const fieldContainer = this.createElement('div', ['field-container']);
        const ui = resolved.resolvedUI;

        // Field label
        if (ui.label) {
            const label = this.createElement('label', ['field-label']);
            label.textContent = ui.label;
            if (resolved.attribute.constraints?.required) {
                label.classList.add('required');
            }
            fieldContainer.appendChild(label);
        }

        // Field input
        const inputElement = this.renderInputElement(resolved);
        fieldContainer.appendChild(inputElement);

        // Help text
        if (ui.helpText) {
            const helpText = this.createElement('div', ['field-help']);
            helpText.textContent = ui.helpText;
            fieldContainer.appendChild(helpText);
        }

        // Validation message container
        const validationContainer = this.createElement('div', ['field-validation']);
        fieldContainer.appendChild(validationContainer);

        return fieldContainer;
    }

    private renderInputElement(resolved: AttributeResolution): HTMLElement {
        const { attribute, resolvedUI } = resolved;
        const ui = resolvedUI;

        switch (ui.renderHint) {
            case 'text-input':
                return this.createTextInput(attribute, ui);
            case 'textarea':
                return this.createTextArea(attribute, ui);
            case 'select':
                return this.createSelect(attribute, ui);
            case 'multiselect':
                return this.createMultiSelect(attribute, ui);
            case 'checkbox':
                return this.createCheckbox(attribute, ui);
            case 'radio':
                return this.createRadioGroup(attribute, ui);
            case 'date-picker':
                return this.createDatePicker(attribute, ui);
            case 'number-input':
                return this.createNumberInput(attribute, ui);
            case 'email-input':
                return this.createEmailInput(attribute, ui);
            case 'phone-input':
                return this.createPhoneInput(attribute, ui);
            default:
                return this.createTextInput(attribute, ui);
        }
    }

    // ===== INPUT ELEMENT CREATORS =====

    private createTextInput(attribute: AttributeObject, ui: any): HTMLElement {
        const input = this.createElement('input', ['field-input', 'text-input']) as HTMLInputElement;
        input.type = 'text';
        input.name = attribute.name;
        input.placeholder = ui.placeholder || '';

        if (attribute.constraints?.maxLength) {
            input.maxLength = attribute.constraints.maxLength;
        }

        this.attachEventHandlers(input, attribute);
        return input;
    }

    private createTextArea(attribute: AttributeObject, ui: any): HTMLElement {
        const textarea = this.createElement('textarea', ['field-input', 'textarea']) as HTMLTextAreaElement;
        textarea.name = attribute.name;
        textarea.placeholder = ui.placeholder || '';

        this.attachEventHandlers(textarea, attribute);
        return textarea;
    }

    private createSelect(attribute: AttributeObject, ui: any): HTMLElement {
        const select = this.createElement('select', ['field-input', 'select']) as HTMLSelectElement;
        select.name = attribute.name;

        // Add default option
        const defaultOption = this.createElement('option', []) as HTMLOptionElement;
        defaultOption.value = '';
        defaultOption.textContent = ui.placeholder || 'Select...';
        select.appendChild(defaultOption);

        // Add options from allowedValues
        if (attribute.allowedValues) {
            attribute.allowedValues.forEach(value => {
                const option = this.createElement('option', []) as HTMLOptionElement;
                option.value = value;
                option.textContent = value;
                select.appendChild(option);
            });
        }

        this.attachEventHandlers(select, attribute);
        return select;
    }

    private createNumberInput(attribute: AttributeObject, ui: any): HTMLElement {
        const input = this.createElement('input', ['field-input', 'number-input']) as HTMLInputElement;
        input.type = 'number';
        input.name = attribute.name;
        input.placeholder = ui.placeholder || '';

        if (attribute.constraints?.min !== undefined) {
            input.min = attribute.constraints.min.toString();
        }
        if (attribute.constraints?.max !== undefined) {
            input.max = attribute.constraints.max.toString();
        }

        this.attachEventHandlers(input, attribute);
        return input;
    }

    private createDatePicker(attribute: AttributeObject, ui: any): HTMLElement {
        const input = this.createElement('input', ['field-input', 'date-picker']) as HTMLInputElement;
        input.type = attribute.dataType === 'DateTime' ? 'datetime-local' : 'date';
        input.name = attribute.name;

        this.attachEventHandlers(input, attribute);
        return input;
    }

    private createCheckbox(attribute: AttributeObject, ui: any): HTMLElement {
        const container = this.createElement('div', ['checkbox-container']);
        const input = this.createElement('input', ['field-input', 'checkbox']) as HTMLInputElement;
        input.type = 'checkbox';
        input.name = attribute.name;

        const label = this.createElement('label', ['checkbox-label']);
        label.textContent = ui.label || attribute.name;

        container.appendChild(input);
        container.appendChild(label);

        this.attachEventHandlers(input, attribute);
        return container;
    }

    private createRadioGroup(attribute: AttributeObject, ui: any): HTMLElement {
        const container = this.createElement('div', ['radio-group']);

        if (attribute.allowedValues) {
            attribute.allowedValues.forEach(value => {
                const radioContainer = this.createElement('div', ['radio-item']);
                const input = this.createElement('input', ['field-input', 'radio']) as HTMLInputElement;
                input.type = 'radio';
                input.name = attribute.name;
                input.value = value;

                const label = this.createElement('label', ['radio-label']);
                label.textContent = value;

                radioContainer.appendChild(input);
                radioContainer.appendChild(label);
                container.appendChild(radioContainer);

                this.attachEventHandlers(input, attribute);
            });
        }

        return container;
    }

    private createEmailInput(attribute: AttributeObject, ui: any): HTMLElement {
        const input = this.createTextInput(attribute, ui) as HTMLInputElement;
        input.type = 'email';
        input.classList.add('email-input');
        return input;
    }

    private createPhoneInput(attribute: AttributeObject, ui: any): HTMLElement {
        const input = this.createTextInput(attribute, ui) as HTMLInputElement;
        input.type = 'tel';
        input.classList.add('phone-input');
        return input;
    }

    private createMultiSelect(attribute: AttributeObject, ui: any): HTMLElement {
        const select = this.createSelect(attribute, ui) as HTMLSelectElement;
        select.multiple = true;
        select.classList.add('multiselect');
        return select;
    }

    // ===== PERSPECTIVE AND CONTEXT RESOLUTION =====

    private resolveAttributeForPerspective(attribute: AttributeObject): AttributeResolution {
        const activePerspective = this.context.perspective.activePerspective;
        const perspective = attribute.perspectives?.[activePerspective];

        return {
            attribute,
            resolvedUI: { ...attribute.ui, ...(perspective?.ui || {}) },
            resolvedDescription: perspective?.description || attribute.description,
            resolvedExamples: [...(attribute.generationExamples || []), ...(perspective?.generationExamples || [])]
        };
    }

    // ===== UTILITY METHODS =====

    private findResource(resourceName: string): ResourceObject | undefined {
        return this.dictionary.find(resource => resource.resourceName === resourceName);
    }

    private groupAttributesByUI(resource: ResourceObject): Array<{ name: string; attributes: AttributeObject[] }> {
        const groups = new Map<string, AttributeObject[]>();

        // Group attributes by their UI group
        resource.attributes.forEach(attribute => {
            const resolvedAttribute = this.resolveAttributeForPerspective(attribute);
            const groupName = resolvedAttribute.resolvedUI.group;

            if (!groups.has(groupName)) {
                groups.set(groupName, []);
            }
            groups.get(groupName)!.push(attribute);
        });

        // Sort groups according to resource.ui.groupOrder
        const orderedGroups: Array<{ name: string; attributes: AttributeObject[] }> = [];

        resource.ui.groupOrder.forEach(groupName => {
            if (groups.has(groupName)) {
                const attributes = groups.get(groupName)!;
                // Sort attributes within group by displayOrder
                attributes.sort((a, b) => {
                    const aResolved = this.resolveAttributeForPerspective(a);
                    const bResolved = this.resolveAttributeForPerspective(b);
                    return aResolved.resolvedUI.displayOrder - bResolved.resolvedUI.displayOrder;
                });
                orderedGroups.push({ name: groupName, attributes });
            }
        });

        return orderedGroups;
    }

    private getAvailablePerspectives(resource: ResourceObject): string[] {
        const perspectives = new Set<string>(['default']);

        resource.attributes.forEach(attribute => {
            if (attribute.perspectives) {
                Object.keys(attribute.perspectives).forEach(perspective => {
                    perspectives.add(perspective);
                });
            }
        });

        return Array.from(perspectives);
    }

    private createElement(tagName: string, classes: string[] = []): HTMLElement {
        const element = document.createElement(tagName);
        element.className = classes.join(' ');
        return element;
    }

    private attachEventHandlers(element: HTMLElement, attribute: AttributeObject): void {
        if (this.config.eventHandlers?.onFieldChange) {
            element.addEventListener('change', (event) => {
                const target = event.target as HTMLInputElement;
                this.config.eventHandlers!.onFieldChange!(attribute.name, target.value, this.context);
            });
        }

        if (this.config.enableRealTimeValidation) {
            element.addEventListener('input', (event) => {
                this.validateField(attribute, (event.target as HTMLInputElement).value);
            });
        }
    }

    private validateField(attribute: AttributeObject, value: any): ValidationResult {
        // Basic validation logic - can be extended
        const errors: string[] = [];

        if (attribute.constraints?.required && !value) {
            errors.push(`${attribute.name} is required`);
        }

        return {
            isValid: errors.length === 0,
            errors: errors.length > 0 ? { [attribute.name]: errors } : {},
            warnings: {}
        };
    }

    private renderPerspectiveSwitcher(): HTMLElement {
        const switcher = this.createElement('div', ['perspective-switcher']);
        const select = this.createElement('select', ['perspective-select']) as HTMLSelectElement;

        this.context.perspective.availablePerspectives.forEach(perspective => {
            const option = this.createElement('option', []) as HTMLOptionElement;
            option.value = perspective;
            option.textContent = perspective;
            option.selected = perspective === this.context.perspective.activePerspective;
            select.appendChild(option);
        });

        select.addEventListener('change', (event) => {
            const newPerspective = (event.target as HTMLSelectElement).value;
            this.switchPerspective(newPerspective);
        });

        switcher.appendChild(select);
        return switcher;
    }

    private renderWizardNavigation(resource: ResourceObject, groups: any[]): HTMLElement {
        const nav = this.createElement('div', ['wizard-navigation']);
        const currentStep = this.context.currentStep || 0;

        // Previous button
        if (currentStep > 0) {
            const prevBtn = this.createElement('button', ['btn', 'btn-secondary']);
            prevBtn.textContent = 'Previous';
            prevBtn.onclick = () => this.previousStep();
            nav.appendChild(prevBtn);
        }

        // Next/Finish button
        const nextBtn = this.createElement('button', ['btn', 'btn-primary']);
        nextBtn.textContent = currentStep === groups.length - 1 ? 'Finish' : 'Next';
        nextBtn.onclick = () => this.nextStep(groups.length);
        nav.appendChild(nextBtn);

        return nav;
    }

    private renderProgressIndicator(totalSteps: number): HTMLElement {
        const progress = this.createElement('div', ['wizard-progress']);
        const currentStep = this.context.currentStep || 0;

        for (let i = 0; i < totalSteps; i++) {
            const step = this.createElement('div', ['progress-step']);
            if (i <= currentStep) step.classList.add('completed');
            if (i === currentStep) step.classList.add('current');
            progress.appendChild(step);
        }

        return progress;
    }

    private initializeContext(): UIRenderContext {
        return {
            perspective: {
                activePerspective: 'default',
                availablePerspectives: ['default'],
                resourceName: ''
            },
            formData: {},
            validationErrors: {},
            currentStep: 0,
            groupVisibility: {}
        };
    }

    // ===== NAVIGATION AND INTERACTION =====

    private switchPerspective(newPerspective: string): void {
        this.context.perspective.activePerspective = newPerspective;
        this.config.eventHandlers?.onPerspectiveChange?.(newPerspective);
        // Re-render the current view
        // Implementation depends on framework integration
    }

    private switchTab(tabIndex: number): void {
        // Tab switching logic
        document.querySelectorAll('.tab-header').forEach((header, index) => {
            header.classList.toggle('active', index === tabIndex);
        });
        document.querySelectorAll('.tab-pane').forEach((pane, index) => {
            pane.classList.toggle('active', index === tabIndex);
        });
    }

    private toggleAccordion(itemIndex: number): void {
        const items = document.querySelectorAll('.accordion-content');
        items[itemIndex]?.classList.toggle('open');
    }

    private nextStep(totalSteps: number): void {
        if (this.context.currentStep! < totalSteps - 1) {
            this.context.currentStep!++;
            this.config.eventHandlers?.onStepChange?.(this.context.currentStep!);
        }
    }

    private previousStep(): void {
        if (this.context.currentStep! > 0) {
            this.context.currentStep!--;
            this.config.eventHandlers?.onStepChange?.(this.context.currentStep!);
        }
    }
}

// ===== FACTORY FUNCTION =====

export function createRenderer(dictionary: ResourceDictionary, config: Partial<RendererConfig> = {}): ConfigDrivenRenderer {
    const defaultConfig: RendererConfig = {
        enableValidation: true,
        enableConditionalLogic: true,
        enableRealTimeValidation: false
    };

    return new ConfigDrivenRenderer(dictionary, { ...defaultConfig, ...config });
}