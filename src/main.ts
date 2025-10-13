import { invoke } from '@tauri-apps/api/core';
import * as monaco from 'monaco-editor';
import { createEditorPanelHeader, panelManager } from './ui-components';
import { ResourceDictionary, ResourceObject, AttributeObject } from './data-dictionary-types';
import { ConfigDrivenRenderer, createRenderer } from './config-driven-renderer';

// Types
interface TestResult {
    success: boolean;
    result: any;
    error?: string;
}

interface CBUCreateRequest {
    cbu_name: string;
    description?: string;
    primary_entity_id?: string;
    primary_lei?: string;
    domicile_country?: string;
    regulatory_jurisdiction?: string;
    business_type?: string;
    created_by?: string;
}

// Global variables
let editor: monaco.editor.IStandaloneCodeEditor;
let currentContext: any = {};
let resourceDictionary: ResourceDictionary | null = null;
let configRenderer: ConfigDrivenRenderer | null = null;
let currentPerspective: string = 'default';

// Initialize the application
document.addEventListener('DOMContentLoaded', async () => {
    await initializeMonacoEditor();
    await loadDataDictionary();
    setupEventListeners();
    console.log('🚀 Data Designer IDE initialized');
});

// Monaco Editor setup
async function initializeMonacoEditor(): Promise<void> {
    const editorContainer = document.getElementById('monaco-editor');
    if (!editorContainer) {
        console.error('Monaco editor container not found');
        return;
    }

    editor = monaco.editor.create(editorContainer, {
        value: 'price * quantity + tax',
        language: 'javascript',
        theme: 'vs-dark',
        fontSize: 14,
        minimap: { enabled: false },
        scrollbar: {
            vertical: 'visible',
            horizontal: 'visible',
            verticalScrollbarSize: 12,
            horizontalScrollbarSize: 12,
            useShadows: false,
            verticalHasArrows: true,
            horizontalHasArrows: true,
        },
        automaticLayout: true,
        wordWrap: 'on',
        lineNumbers: 'on',
        renderWhitespace: 'boundary',
    });

    // Expose editor globally for debugging
    (window as any).editor = editor;
}

// Event listeners setup
function setupEventListeners(): void {
    // Menu action handlers
    document.addEventListener('click', (e) => {
        const target = e.target as HTMLElement;
        if (target.classList.contains('menu-item') && target.dataset.action) {
            handleMenuAction(target.dataset.action);
        }
    });

    // Run code button
    const runButton = document.getElementById('run-code');
    if (runButton) {
        runButton.addEventListener('click', runCode);
    }

    // Save rule button
    const saveButton = document.getElementById('save-rule');
    if (saveButton) {
        saveButton.addEventListener('click', saveRule);
    }
}

// Menu action handler
function handleMenuAction(action: string): void {
    console.log(`ℹ️ Action: ${action}`);

    switch (action) {
        case 'create-cbu':
            createNewCBU();
            break;
        case 'data-dictionary':
            toggleDataDictionary();
            break;
        case 'kyc-wizard':
            openKYCWizard();
            break;
        case 'trade-settlement':
            openTradeSettlement();
            break;
        case 'schema-view':
            openSchemaViewer();
            break;
        case 'ast-view':
            showAST();
            break;
        case 'export-rules':
            exportRules();
            break;
        case 'import-rules':
            importRules();
            break;
        case 'settings':
            openSettings();
            break;
        default:
            showComingSoon(action);
            break;
    }
}

// CBU Creation Modal
function createNewCBU(): void {
    const modalHtml = `
        <div class="modal" id="create-cbu-modal" style="display: block;">
            <div class="modal-content">
                <div class="modal-header">
                    <h3>Create New Client Business Unit</h3>
                    <span class="close" onclick="closeModal('create-cbu-modal')">&times;</span>
                </div>
                <div class="modal-body">
                    <form id="cbu-form">
                        <div class="form-group">
                            <label for="cbu-name">CBU Name *</label>
                            <input type="text" id="cbu-name" required placeholder="Enter CBU name">
                        </div>
                        <div class="form-group">
                            <label for="cbu-description">Description</label>
                            <textarea id="cbu-description" rows="3" placeholder="Optional description"></textarea>
                        </div>
                        <div class="form-group">
                            <label for="primary-entity-id">Primary Entity ID</label>
                            <input type="text" id="primary-entity-id" placeholder="e.g., ENT-12345">
                        </div>
                        <div class="form-group">
                            <label for="primary-lei">Primary LEI</label>
                            <input type="text" id="primary-lei" placeholder="Legal Entity Identifier">
                        </div>
                        <div class="form-row">
                            <div class="form-group">
                                <label for="domicile-country">Domicile Country</label>
                                <select id="domicile-country">
                                    <option value="">Select country</option>
                                    <option value="US">United States</option>
                                    <option value="UK">United Kingdom</option>
                                    <option value="DE">Germany</option>
                                    <option value="FR">France</option>
                                    <option value="SG">Singapore</option>
                                </select>
                            </div>
                            <div class="form-group">
                                <label for="regulatory-jurisdiction">Regulatory Jurisdiction</label>
                                <select id="regulatory-jurisdiction">
                                    <option value="">Select jurisdiction</option>
                                    <option value="SEC">SEC (US)</option>
                                    <option value="FCA">FCA (UK)</option>
                                    <option value="BaFin">BaFin (Germany)</option>
                                    <option value="MAS">MAS (Singapore)</option>
                                </select>
                            </div>
                        </div>
                        <div class="form-group">
                            <label for="business-type">Business Type</label>
                            <select id="business-type">
                                <option value="">Select business type</option>
                                <option value="Investment Manager">Investment Manager</option>
                                <option value="Hedge Fund">Hedge Fund</option>
                                <option value="Private Equity">Private Equity</option>
                                <option value="Family Office">Family Office</option>
                                <option value="Pension Fund">Pension Fund</option>
                                <option value="Insurance Company">Insurance Company</option>
                            </select>
                        </div>
                        <div class="form-group">
                            <label for="created-by">Created By</label>
                            <input type="text" id="created-by" placeholder="Your name/ID">
                        </div>
                    </form>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-secondary" onclick="closeModal('create-cbu-modal')">Cancel</button>
                    <button type="button" class="btn btn-primary" onclick="submitCBU()">Create CBU</button>
                </div>
            </div>
        </div>
    `;

    document.body.insertAdjacentHTML('beforeend', modalHtml);
}

// Submit CBU form
async function submitCBU(): Promise<void> {
    const form = document.getElementById('cbu-form') as HTMLFormElement;
    if (!form) return;

    const formData = new FormData(form);
    const cbuData: CBUCreateRequest = {
        cbu_name: (document.getElementById('cbu-name') as HTMLInputElement).value,
        description: (document.getElementById('cbu-description') as HTMLTextAreaElement).value || undefined,
        primary_entity_id: (document.getElementById('primary-entity-id') as HTMLInputElement).value || undefined,
        primary_lei: (document.getElementById('primary-lei') as HTMLInputElement).value || undefined,
        domicile_country: (document.getElementById('domicile-country') as HTMLSelectElement).value || undefined,
        regulatory_jurisdiction: (document.getElementById('regulatory-jurisdiction') as HTMLSelectElement).value || undefined,
        business_type: (document.getElementById('business-type') as HTMLSelectElement).value || undefined,
        created_by: (document.getElementById('created-by') as HTMLInputElement).value || undefined,
    };

    try {
        const result = await invoke('create_cbu', { request: cbuData });
        console.log('✅ CBU created successfully:', result);
        closeModal('create-cbu-modal');
        showSuccessMessage('CBU created successfully!');
    } catch (error) {
        console.error('❌ Failed to create CBU:', error);
        showErrorMessage(`Failed to create CBU: ${error}`);
    }
}

// Modal utilities
function closeModal(modalId: string): void {
    const modal = document.getElementById(modalId);
    if (modal) {
        modal.remove();
    }
}

function showSuccessMessage(message: string): void {
    // Implementation for success toast
    console.log('✅', message);
}

function showErrorMessage(message: string): void {
    // Implementation for error toast
    console.error('❌', message);
}

// Data Dictionary v2.0 - Configuration-driven approach
async function loadDataDictionary(): Promise<void> {
    try {
        // First try to load the new v2.0 resource dictionary structure
        const resourceDict = await loadResourceDictionary();
        if (resourceDict) {
            resourceDictionary = resourceDict;
            await initializeConfigRenderer();
            console.log('📚 Resource Dictionary v2.0 loaded:', resourceDict);
            return;
        }

        // Fallback to legacy data dictionary if new format not available
        const attributes = await invoke('dd_get_data_dictionary');
        console.log('📚 Legacy data dictionary loaded:', attributes);
        // TODO: Convert legacy format to new structure
    } catch (error) {
        console.error('Failed to load data dictionary:', error);
    }
}

// Load new ResourceDictionary format
async function loadResourceDictionary(): Promise<ResourceDictionary | null> {
    try {
        // Load from database via Tauri commands
        const dictionaries = await invoke('cd_get_resource_dictionaries') as any[];
        if (!dictionaries || dictionaries.length === 0) {
            console.warn('No resource dictionaries found in database');
            return null;
        }

        // Use the first dictionary (Financial Services Data Dictionary)
        const dict = dictionaries[0];
        const resources = await invoke('cd_get_resources', { dictionary_id: dict.id }) as any[];

        // Convert to frontend format
        const resourceObjects: ResourceObject[] = [];
        for (const resource of resources) {
            const config = await invoke('cd_get_resource_config', { resource_name: resource.resource_name });
            if (config) {
                resourceObjects.push(config as ResourceObject);
            }
        }

        const dictionary: ResourceDictionary = {
            dictionaryName: dict.dictionary_name,
            version: dict.version,
            description: dict.description,
            author: dict.author,
            creationDate: dict.creation_date,
            lastModified: dict.last_modified,
            resources: resourceObjects
        };

        return dictionary;
    } catch (error) {
        console.warn('Failed to load resource dictionary from database:', error);
        return null;
    }
}

// Initialize configuration-driven renderer
async function initializeConfigRenderer(): Promise<void> {
    if (!resourceDictionary) return;

    configRenderer = createRenderer(resourceDictionary, {
        enableValidation: true,
        enableConditionalLogic: true,
        enableRealTimeValidation: true,
        eventHandlers: {
            onFieldChange: handleFieldChange,
            onPerspectiveChange: handlePerspectiveChange,
            onStepChange: handleWizardStepChange
        }
    });

    console.log('🎨 Configuration-driven renderer initialized');
}

// ===== CONFIGURATION-DRIVEN RENDERER EVENT HANDLERS =====

function handleFieldChange(fieldName: string, value: any, context: any): void {
    console.log(`🔄 Field changed: ${fieldName} = ${value}`);

    // Update the current context with new value
    currentContext[fieldName] = value;

    // Trigger real-time rule generation if applicable
    if (shouldTriggerRuleGeneration(fieldName, value)) {
        generateRuleFromContext(fieldName, value);
    }
}

function handlePerspectiveChange(newPerspective: string): void {
    console.log(`👁️ Perspective switched: ${currentPerspective} → ${newPerspective}`);
    currentPerspective = newPerspective;

    // Re-render current resource with new perspective
    if (configRenderer && resourceDictionary) {
        refreshCurrentResourceView();
    }
}

function handleWizardStepChange(step: number): void {
    console.log(`🪄 Wizard step: ${step}`);

    // Update progress indicator
    updateWizardProgress(step);
}

function shouldTriggerRuleGeneration(fieldName: string, value: any): boolean {
    // Logic to determine if this field change should trigger AI rule generation
    const ruleTriggerFields = ['risk_score', 'sanctions_screening_result', 'jurisdiction'];
    return ruleTriggerFields.includes(fieldName) && value;
}

async function generateRuleFromContext(triggeredField: string, value: any): Promise<void> {
    try {
        if (!resourceDictionary) return;

        // Find the attribute that triggered the rule generation
        const resource = resourceDictionary[0]; // For demo, use first resource
        const attribute = resource.attributes.find(attr => attr.name === triggeredField);

        if (attribute?.generationExamples) {
            const examples = attribute.perspectives?.[currentPerspective]?.generationExamples
                || attribute.generationExamples;

            console.log(`🤖 Generating rule for ${triggeredField} with examples:`, examples);

            // In a real implementation, this would call an AI service
            // For now, use the first example as a template
            if (examples.length > 0) {
                const generatedRule = examples[0].response.replace(
                    new RegExp(triggeredField, 'g'),
                    `${triggeredField} == '${value}'`
                );

                // Update the Monaco editor with generated rule
                if (editor) {
                    const currentContent = editor.getValue();
                    const newContent = currentContent + '\n\n// Auto-generated rule:\n' + generatedRule;
                    editor.setValue(newContent);
                }

                console.log(`✨ Generated rule: ${generatedRule}`);
            }
        }
    } catch (error) {
        console.error('Failed to generate rule:', error);
    }
}

// ===== RESOURCE RENDERING AND NAVIGATION =====

async function renderResource(resourceName: string, perspective: string = currentPerspective): Promise<void> {
    if (!configRenderer) {
        console.error('Config renderer not initialized');
        return;
    }

    try {
        const renderedElement = configRenderer.renderResource(resourceName, perspective);

        // Find the container where we want to show the rendered form
        const container = document.getElementById('resource-form-container');
        if (container) {
            container.innerHTML = '';
            container.appendChild(renderedElement);
            console.log(`📋 Rendered resource: ${resourceName} (${perspective} perspective)`);
        } else {
            // Create container if it doesn't exist
            const newContainer = document.createElement('div');
            newContainer.id = 'resource-form-container';
            newContainer.className = 'resource-form-container';
            newContainer.appendChild(renderedElement);

            // Insert into sidebar or main content area
            const sidebar = document.querySelector('.sidebar-content');
            if (sidebar) {
                sidebar.appendChild(newContainer);
            }
        }
    } catch (error) {
        console.error(`Failed to render resource ${resourceName}:`, error);
    }
}

function refreshCurrentResourceView(): void {
    // Re-render the current resource with the new perspective
    if (resourceDictionary && resourceDictionary.length > 0) {
        renderResource(resourceDictionary[0].resourceName, currentPerspective);
    }
}

function updateWizardProgress(currentStep: number): void {
    const progressSteps = document.querySelectorAll('.progress-step');
    progressSteps.forEach((step, index) => {
        step.classList.toggle('completed', index < currentStep);
        step.classList.toggle('current', index === currentStep);
    });
}

// ===== PERSPECTIVE MANAGEMENT =====

function createPerspectiveSelector(): HTMLElement {
    if (!resourceDictionary || resourceDictionary.length === 0) {
        return document.createElement('div');
    }

    const resource = resourceDictionary[0]; // For demo, use first resource
    const perspectives = new Set<string>(['default']);

    // Collect all available perspectives from attributes
    resource.attributes.forEach(attr => {
        if (attr.perspectives) {
            Object.keys(attr.perspectives).forEach(p => perspectives.add(p));
        }
    });

    const selectorContainer = document.createElement('div');
    selectorContainer.className = 'perspective-selector';

    const label = document.createElement('label');
    label.textContent = 'View: ';
    label.className = 'perspective-label';

    const select = document.createElement('select');
    select.className = 'perspective-select';

    perspectives.forEach(perspective => {
        const option = document.createElement('option');
        option.value = perspective;
        option.textContent = perspective.charAt(0).toUpperCase() + perspective.slice(1);
        option.selected = perspective === currentPerspective;
        select.appendChild(option);
    });

    select.addEventListener('change', (event) => {
        const target = event.target as HTMLSelectElement;
        handlePerspectiveChange(target.value);
    });

    selectorContainer.appendChild(label);
    selectorContainer.appendChild(select);

    return selectorContainer;
}

function toggleDataDictionary(): void {
    const sidebar = document.getElementById('data-dictionary-sidebar');
    if (sidebar) {
        sidebar.classList.toggle('hidden');
    }
}

// Run code functionality
async function runCode(): Promise<void> {
    if (!editor) {
        console.error('Editor not initialized');
        return;
    }

    const code = editor.getValue();
    if (!code.trim()) {
        showErrorMessage('Please enter some code to run');
        return;
    }

    try {
        const result: TestResult = await invoke('test_rule', {
            dslText: code,
            context: currentContext
        });

        console.log('🎯 Test result:', result);
        displayTestResult(result);
    } catch (error) {
        console.error('❌ Test failed:', error);
        showErrorMessage(`Test failed: ${error}`);
    }
}

function displayTestResult(result: TestResult): void {
    const resultContainer = document.getElementById('test-results');
    if (!resultContainer) return;

    const resultHtml = result.success
        ? `<div class="result-success">✅ Success: ${JSON.stringify(result.result)}</div>`
        : `<div class="result-error">❌ Error: ${result.error}</div>`;

    resultContainer.innerHTML = resultHtml;
}

// Save rule functionality
async function saveRule(): Promise<void> {
    if (!editor) return;

    const code = editor.getValue();
    const ruleName = prompt('Enter rule name:');

    if (!ruleName) return;

    try {
        await invoke('save_rule', {
            name: ruleName,
            dslText: code
        });
        console.log('💾 Rule saved successfully');
        showSuccessMessage('Rule saved successfully!');
    } catch (error) {
        console.error('❌ Failed to save rule:', error);
        showErrorMessage(`Failed to save rule: ${error}`);
    }
}

// AST Visualization
async function showAST(): Promise<void> {
    if (!editor) return;

    const code = editor.getValue();
    if (!code.trim()) {
        showErrorMessage('Please enter some code to visualize');
        return;
    }

    try {
        const ast = await invoke('visualize_ast', { dslText: code });
        console.log('🌳 AST:', ast);
        // Implementation to show AST in UI
    } catch (error) {
        console.error('❌ AST visualization failed:', error);
        showErrorMessage(`AST visualization failed: ${error}`);
    }
}

// Schema viewer
async function openSchemaViewer(): Promise<void> {
    try {
        await invoke('open_schema_viewer');
    } catch (error) {
        console.error('Failed to open schema viewer:', error);
    }
}

// Placeholder implementations
function exportRules(): void {
    showComingSoon('export-rules');
}

function importRules(): void {
    showComingSoon('import-rules');
}

function openSettings(): void {
    showComingSoon('settings');
}

function showComingSoon(action: string): void {
    showErrorMessage(`${action.replace('-', ' ')} - Coming soon!`);
}

// ===== CONFIGURATION-DRIVEN FORM LAUNCHERS =====

async function openKYCWizard(): Promise<void> {
    console.log('🧙‍♂️ Opening KYC Wizard...');

    if (!configRenderer) {
        console.error('Configuration renderer not initialized');
        return;
    }

    try {
        // Render the KYC wizard using configuration
        await renderResource('ClientOnboardingKYC', currentPerspective);

        // Show the perspective selector
        const sidebar = document.querySelector('.sidebar-content');
        if (sidebar) {
            // Remove existing perspective selector
            const existingSelector = sidebar.querySelector('.perspective-selector');
            if (existingSelector) {
                existingSelector.remove();
            }

            // Add new perspective selector
            sidebar.insertBefore(createPerspectiveSelector(), sidebar.firstChild);
        }

        // Focus on the resource form
        const formContainer = document.getElementById('resource-form-container');
        if (formContainer) {
            formContainer.scrollIntoView({ behavior: 'smooth' });
        }

    } catch (error) {
        console.error('Failed to open KYC wizard:', error);
        showErrorMessage('Failed to open KYC wizard');
    }
}

async function openTradeSettlement(): Promise<void> {
    console.log('📈 Opening Trade Settlement Form...');

    if (!configRenderer) {
        console.error('Configuration renderer not initialized');
        return;
    }

    try {
        // Render the Trade Settlement form using tabs layout
        await renderResource('TradeSettlementSystem', currentPerspective);

        console.log('✅ Trade settlement form opened');
    } catch (error) {
        console.error('Failed to open trade settlement form:', error);
        showErrorMessage('Failed to open trade settlement form');
    }
}

// Function to demonstrate all layout types
async function demonstrateAllLayouts(): Promise<void> {
    if (!resourceDictionary || !configRenderer) {
        console.log('Demo not available - resource dictionary not loaded');
        return;
    }

    console.log('🎨 Demonstrating all layout types...');

    // Create demo containers for each layout
    const demoContainer = document.createElement('div');
    demoContainer.className = 'layout-demo-container';
    demoContainer.style.cssText = `
        position: fixed;
        top: 50px;
        left: 50px;
        width: 80vw;
        height: 80vh;
        background: #1e1e1e;
        border: 2px solid #0e639c;
        border-radius: 8px;
        padding: 20px;
        z-index: 2000;
        overflow-y: auto;
    `;

    // Demo header
    const header = document.createElement('h2');
    header.textContent = 'Configuration-Driven UI Layouts Demo';
    header.style.cssText = 'color: #0e639c; margin-bottom: 20px;';
    demoContainer.appendChild(header);

    // Close button
    const closeButton = document.createElement('button');
    closeButton.textContent = '✕ Close Demo';
    closeButton.style.cssText = `
        position: absolute;
        top: 20px;
        right: 20px;
        background: #f44747;
        color: white;
        border: none;
        padding: 8px 16px;
        border-radius: 4px;
        cursor: pointer;
    `;
    closeButton.onclick = () => demoContainer.remove();
    demoContainer.appendChild(closeButton);

    // Perspective info
    const perspectiveInfo = document.createElement('div');
    perspectiveInfo.style.cssText = 'background: #2d2d30; padding: 15px; border-radius: 6px; margin-bottom: 20px;';
    perspectiveInfo.innerHTML = `
        <h3 style="color: #4ec9b0; margin-top: 0;">Current Perspective: ${currentPerspective}</h3>
        <p style="color: #d4d4d4; margin: 10px 0;">
            This demo shows how the same data renders differently based on perspective.
            Each attribute can have different labels, descriptions, and UI configurations per perspective.
        </p>
        <div style="display: flex; gap: 10px; margin-top: 10px;">
            <button onclick="window.switchDemoPerspective('KYC')" style="background: #0e639c; color: white; border: none; padding: 6px 12px; border-radius: 3px; cursor: pointer;">KYC View</button>
            <button onclick="window.switchDemoPerspective('FundAccounting')" style="background: #0e639c; color: white; border: none; padding: 6px 12px; border-radius: 3px; cursor: pointer;">Fund Accounting View</button>
        </div>
    `;
    demoContainer.appendChild(perspectiveInfo);

    // Render the KYC resource (wizard layout)
    const kycRendered = configRenderer.renderResource('ClientOnboardingKYC', currentPerspective);
    demoContainer.appendChild(kycRendered);

    document.body.appendChild(demoContainer);

    // Expose demo perspective switcher
    (window as any).switchDemoPerspective = (newPerspective: string) => {
        handlePerspectiveChange(newPerspective);
        demoContainer.remove();
        demonstrateAllLayouts(); // Re-render with new perspective
    };
}

// Expose functions globally for HTML onclick handlers
(window as any).closeModal = closeModal;
(window as any).submitCBU = submitCBU;
(window as any).runCode = runCode;
(window as any).saveRule = saveRule;
(window as any).showAST = showAST;
(window as any).openKYCWizard = openKYCWizard;
(window as any).openTradeSettlement = openTradeSettlement;
(window as any).demonstrateAllLayouts = demonstrateAllLayouts;