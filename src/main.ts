import { invoke } from '@tauri-apps/api/core';
import { getSharedDbService } from './shared-db-service.js';
import type { SharedDatabaseService } from './shared-db-service.js';

// Ensure Tauri API is available globally for dynamically loaded modules
(window as any).__TAURI__ = {
    invoke: invoke
};
(window as any).__TAURI_INVOKE__ = invoke;
import * as monaco from 'monaco-editor';
import { createEditorPanelHeader, panelManager } from './ui-components';
import { ResourceDictionary, ResourceObject, AttributeObject } from './data-dictionary-types';
import { ConfigDrivenRenderer, createRenderer } from './config-driven-renderer';
import { MetadataDrivenEngine } from './metadata-driven-engine';

// Types
interface TestResult {
    success: boolean;
    result: any;
    error?: string;
}

interface AISuggestionRequest {
    user_prompt: string;
    perspective: string;
    selected_attributes: string[];
}

interface AISuggestionResponse {
    success: boolean;
    generated_dsl?: string;
    explanation?: string;
    error?: string;
}


// Global variables
let editor: monaco.editor.IStandaloneCodeEditor;
let currentContext: any = {};
let resourceDictionary: ResourceDictionary | null = null;
let configRenderer: ConfigDrivenRenderer | null = null;
let metadataEngine: MetadataDrivenEngine | null = null;
let currentPerspective: string = 'default';
let sharedDbService: SharedDatabaseService | null = null;

// ==========================================
// IMMEDIATE GLOBAL FUNCTION EXPORTS
// ==========================================
// Export functions to window object for onclick handlers
// Forward declarations - actual implementations defined later

// Forward declaration - will be replaced with real implementation after DOM loads
let menuActionImpl: ((action: string) => void) | null = null;
(window as any).menuAction = (action: string) => {
    if (menuActionImpl) {
        menuActionImpl(action);
    } else {
        console.log('MenuAction (not yet initialized):', action);
    }
};
(window as any).refreshRules = () => console.log('RefreshRules (placeholder)');
(window as any).testDatabaseConnection = () => console.log('TestDB (placeholder)');
(window as any).testMonacoEditor = () => console.log('TestMonaco (placeholder)');
(window as any).refreshDatabase = () => console.log('RefreshDatabase (placeholder)');
(window as any).closeTab = (tabId: string) => console.log('CloseTab (placeholder):', tabId);
(window as any).undockEditor = () => console.log('UndockEditor (placeholder)');
(window as any).toggleEditorMaximize = () => console.log('ToggleEditorMaximize (placeholder)');

console.log('🔧 Placeholder functions exported for rules editor');

// Initialize the application
document.addEventListener('DOMContentLoaded', async () => {
    // Initialize shared database service first
    sharedDbService = getSharedDbService();

    // Try to connect to shared database service or initialize it
    try {
        console.log('🔌 Attempting to connect to shared database service...');
        const connectionStatus = sharedDbService.getConnectionStatus();
        console.log('🔍 Initial connection status:', connectionStatus);

        if (!connectionStatus.isConnected) {
            console.log('🔌 Initializing database connection from IDE...');
            await sharedDbService.initialize();
            // Update database status display after successful initialization
            await checkDatabaseStatus();
        } else {
            console.log('✅ Using existing shared database connection');
            // Update database status display for existing connection
            await checkDatabaseStatus();
        }
    } catch (error) {
        console.warn('⚠️ Shared database service initialization failed:', error);
    }

    await initializeMonacoEditor();
    await initializeMetadataEngine();
    await loadDataDictionary();
    setupEventListeners();
    await checkDatabaseStatus(); // Check database status on startup
    console.log('🚀 Data Designer Rules Editor initialized');
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
        language: 'plaintext',
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

// AI Suggestion functionality - The revolutionary AI Context Engine integration
async function getAISuggestion(): Promise<void> {
    if (!editor) {
        console.error('Editor not initialized');
        return;
    }

    // Get user prompt
    const userPrompt = prompt('Describe the rule you want to create:');
    if (!userPrompt?.trim()) {
        return;
    }

    // Get current perspective (default to 'KYC' if not set)
    const perspective = currentPerspective === 'default' ? 'KYC' : currentPerspective;

    // For now, we'll use all available attributes. In the future, this could be
    // selected from the data dictionary sidebar
    const selectedAttributes = [
        'legal_entity_name',
        'ubo_full_name',
        'sanctions_screening_result',
        'risk_score',
        'jurisdiction'
    ];

    try {
        console.log('🤖 AI Context Engine: Requesting DSL generation...');
        console.log('📝 User Prompt:', userPrompt);
        console.log('👀 Perspective:', perspective);
        console.log('📊 Selected Attributes:', selectedAttributes);

        // Show loading indicator
        const loadingMsg = document.createElement('div');
        loadingMsg.innerHTML = '🤖 AI Context Engine is generating your rule...';
        loadingMsg.style.cssText = `
            position: fixed;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            background: #2d2d30;
            color: #d4d4d4;
            padding: 20px;
            border-radius: 8px;
            border: 1px solid #3e3e42;
            z-index: 10000;
            font-family: monospace;
        `;
        document.body.appendChild(loadingMsg);

        const request: AISuggestionRequest = {
            user_prompt: userPrompt,
            perspective: perspective,
            selected_attributes: selectedAttributes
        };

        const response = await invoke<AISuggestionResponse>('get_ai_suggestion', { request });

        // Remove loading indicator
        document.body.removeChild(loadingMsg);

        if (response.success && response.generated_dsl) {
            // Set the generated DSL in the editor
            editor.setValue(response.generated_dsl);
            editor.focus();

            // Show explanation if available
            if (response.explanation) {
                const explanationModal = document.createElement('div');
                explanationModal.innerHTML = `
                    <div style="
                        position: fixed;
                        top: 0;
                        left: 0;
                        width: 100%;
                        height: 100%;
                        background: rgba(0, 0, 0, 0.7);
                        z-index: 10001;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                    ">
                        <div style="
                            background: #2d2d30;
                            color: #d4d4d4;
                            padding: 30px;
                            border-radius: 12px;
                            border: 1px solid #3e3e42;
                            max-width: 600px;
                            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', monospace;
                        ">
                            <h3 style="margin: 0 0 15px 0; color: #4CAF50;">🤖 AI Generated Rule</h3>
                            <p style="margin: 0 0 20px 0; line-height: 1.6;">${response.explanation}</p>
                            <div style="text-align: right;">
                                <button onclick="this.closest('.modal').remove()" style="
                                    background: #0078d4;
                                    color: white;
                                    border: none;
                                    padding: 8px 16px;
                                    border-radius: 4px;
                                    cursor: pointer;
                                ">Got it!</button>
                            </div>
                        </div>
                    </div>
                `;
                explanationModal.className = 'modal';
                document.body.appendChild(explanationModal);
            }

            console.log('✅ AI Context Engine: Successfully generated DSL rule');
        } else {
            // Show error message
            const errorMsg = response.error || 'Unknown error occurred';
            alert(`❌ AI Context Engine Error: ${errorMsg}`);
            console.error('AI Context Engine Error:', errorMsg);
        }

    } catch (error) {
        // Remove loading indicator if it exists
        const loadingIndicator = document.querySelector('div[style*="AI Context Engine is generating"]');
        if (loadingIndicator) {
            document.body.removeChild(loadingIndicator);
        }

        console.error('AI Context Engine Error:', error);
        alert(`❌ Failed to get AI suggestion: ${error}`);
    }
}

// Menu action handler
function handleMenuAction(action: string): void {
    console.log(`ℹ️ Action: ${action}`);

    switch (action) {
        case 'ai-suggestion':
            getAISuggestion();
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

// ===== CRITICAL MISSING COMPONENT: Initialize Metadata Engine =====
async function initializeMetadataEngine(): Promise<void> {
    try {
        metadataEngine = new MetadataDrivenEngine();
        await metadataEngine.loadResourceDictionary();
        console.log('🚀 Metadata-Driven Engine initialized');
    } catch (error) {
        console.error('❌ Failed to initialize Metadata Engine:', error);
        // Continue without the engine for now
    }
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

        // ResourceDictionary is just an array of ResourceObject[]
        const dictionary: ResourceDictionary = resourceObjects;

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
    // THE CRITICAL CHANGE: Use MetadataDrivenEngine if available, fallback to configRenderer
    if (metadataEngine) {
        console.log(`🚀 Using MetadataDrivenEngine for ${resourceName}`);
        try {
            const renderedElement = await metadataEngine.renderResource(resourceName, perspective);

        // Find the container where we want to show the rendered form
        const container = document.getElementById('resource-form-container');
        const editorContainer = document.getElementById('editor-panel');

        if (container) {
            container.innerHTML = '';
            container.appendChild(renderedElement);
            container.classList.remove('hidden');

            // Hide the Monaco editor when showing a form
            if (editorContainer) {
                editorContainer.style.display = 'none';
            }

            // Add close button to return to editor
            const closeButton = document.createElement('button');
            closeButton.textContent = '← Back to Rule Editor';
            closeButton.className = 'btn btn-secondary';
            closeButton.style.cssText = 'margin: 12px; position: absolute; top: 0; right: 0; z-index: 10;';
            closeButton.onclick = () => {
                container.classList.add('hidden');
                if (editorContainer) {
                    editorContainer.style.display = 'flex';
                }
            };
            container.appendChild(closeButton);

            console.log(`📋 Rendered resource: ${resourceName} (${perspective} perspective) via MetadataDrivenEngine`);
        } else {
            console.error('Resource form container not found in DOM');
        }
        } catch (error) {
            console.error(`❌ MetadataDrivenEngine failed for ${resourceName}:`, error);
        }
        return;
    }

    // FALLBACK: Use the old configRenderer if MetadataDrivenEngine not available
    if (!configRenderer) {
        console.error('❌ No renderer available (neither MetadataDrivenEngine nor configRenderer)');
        return;
    }

    console.log(`🔄 Falling back to configRenderer for ${resourceName}`);
    try {
        const renderedElement = configRenderer.renderResource(resourceName, perspective);

        // Find the container where we want to show the rendered form
        const container = document.getElementById('resource-form-container');
        const editorContainer = document.getElementById('editor-panel');

        if (container) {
            container.innerHTML = '';
            container.appendChild(renderedElement);
            container.classList.remove('hidden');

            // Hide the Monaco editor when showing a form
            if (editorContainer) {
                editorContainer.style.display = 'none';
            }

            // Add close button to return to editor
            const closeButton = document.createElement('button');
            closeButton.textContent = '← Back to Rule Editor';
            closeButton.className = 'btn btn-secondary';
            closeButton.style.cssText = 'margin: 12px; position: absolute; top: 0; right: 0; z-index: 10;';
            closeButton.onclick = () => {
                container.classList.add('hidden');
                if (editorContainer) {
                    editorContainer.style.display = '';
                }
            };
            container.appendChild(closeButton);

            console.log(`📋 Rendered resource: ${resourceName} (${perspective} perspective) via configRenderer fallback`);
        } else {
            console.error('Resource form container not found in DOM');
        }
    } catch (error) {
        console.error(`❌ Both renderers failed for ${resourceName}:`, error);
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


// Simplified menu action handler - Rules editor only
function menuAction(action: string): void {
    console.log('🔧 Menu action:', action);

    switch(action) {
        // File menu
        case 'new-rule':
            console.log('Creating new rule...');
            if (editor) {
                editor.setValue('// New rule - ' + new Date().toISOString() + '\n');
                editor.focus();
            }
            break;
        case 'ai-suggestion':
            console.log('Opening AI suggestion...');
            getAISuggestion();
            break;
        case 'open-rule':
            console.log('Opening rule...');
            alert('Open Rule functionality - Coming soon!');
            break;
        case 'save-rule':
            console.log('Saving rule...');
            if (editor) {
                const content = editor.getValue();
                console.log('Rule content to save:', content);
                alert('Save Rule functionality - Coming soon!\nRule content logged to console.');
            }
            break;
        case 'export-rules':
            console.log('Exporting rules...');
            alert('Export Rules functionality - Coming soon!');
            break;

        // Database menu
        case 'data-dictionary':
            console.log('Opening data dictionary...');
            alert('Data Dictionary functionality - Coming soon!');
            break;
        case 'schema-viewer':
            console.log('Opening schema viewer...');
            alert('Schema Viewer functionality - Coming soon!');
            break;
        case 'db-connection':
            console.log('Opening database connection settings...');
            alert('Database Connection Settings - Coming soon!');
            break;

        // Rules menu
        case 'rules-catalogue':
            console.log('Opening rules catalogue...');
            alert('Rules Catalogue functionality - Coming soon!');
            break;
        case 'grammar-editor':
            console.log('Opening grammar editor...');
            alert('Grammar Editor functionality - Coming soon!');
            break;
        case 'test-rule':
            console.log('Testing current rule...');
            if (editor) {
                const content = editor.getValue();
                console.log('Testing rule:', content);
                alert('Test Rule functionality - Coming soon!\nRule content logged to console.');
            }
            break;
        case 'find-similar':
            console.log('Finding similar rules...');
            alert('Find Similar Rules functionality - Coming soon!');
            break;
        case 'show-ast':
            console.log('Showing AST...');
            alert('Show AST functionality - Coming soon!');
            break;

        // Tools menu
        case 'lsp-settings':
            console.log('Opening LSP settings...');
            alert('LSP Settings functionality - Coming soon!');
            break;
        case 'preferences':
            console.log('Opening preferences...');
            alert('Preferences functionality - Coming soon!');
            break;

        // Help menu
        case 'documentation':
            console.log('Opening documentation...');
            alert('Documentation functionality - Coming soon!');
            break;
        case 'about':
            console.log('Opening about...');
            alert('About Data Designer IDE\n\nVersion: 1.0.0\nProfessional multi-domain IDE for data transformation rules');
            break;

        default:
            console.log('⚠️ Unhandled menu action:', action);
            alert(`Menu action "${action}" not yet implemented.`);
    }
}

// Assign the real implementation to replace the forward declaration
menuActionImpl = menuAction;

// UI utility functions
function refreshDatabase(): void {
    console.log('Refreshing database...');
    // Add database refresh logic here
}

function refreshRules(): void {
    console.log('Refreshing rules...');
    // Add rules refresh logic here
}

function undockEditor(): void {
    console.log('Undocking editor...');
    // Add editor undock logic here
}

function toggleEditorMaximize(): void {
    console.log('Toggling editor maximize...');
    // Add editor maximize toggle logic here
}

function closeTab(tabId: string): void {
    console.log('Closing tab:', tabId);
    // Add tab close logic here
}

// ==========================================
// EXPOSE ALL FUNCTIONS TO GLOBAL SCOPE IMMEDIATELY
// ==========================================
// These MUST be at module level, not inside any event handlers,
// so onclick handlers can access them when HTML loads

// Expose functions globally for HTML onclick handlers
(window as any).closeModal = closeModal;
(window as any).runCode = runCode;
(window as any).saveRule = saveRule;
(window as any).showAST = showAST;
(window as any).openKYCWizard = openKYCWizard;
(window as any).openTradeSettlement = openTradeSettlement;
(window as any).demonstrateAllLayouts = demonstrateAllLayouts;
(window as any).getAISuggestion = getAISuggestion;

// Expose menu and UI functions
(window as any).menuAction = menuAction;
(window as any).refreshDatabase = refreshDatabase;
(window as any).refreshRules = refreshRules;
(window as any).undockEditor = undockEditor;
(window as any).toggleEditorMaximize = toggleEditorMaximize;
(window as any).closeTab = closeTab;
(window as any).testDatabaseConnection = testDatabaseConnection;
(window as any).testMonacoEditor = testMonacoEditor;

// Database status checking
async function checkDatabaseStatus(): Promise<void> {
    try {
        // Use shared database service if available, otherwise fall back to direct invoke
        let result: any;
        if (sharedDbService) {
            const connectionStatus = sharedDbService.getConnectionStatus();
            console.log('🔍 Database status check via shared service:', connectionStatus);
            result = {
                connected: connectionStatus.isConnected,
                database: connectionStatus.status?.database || 'Connected'
            };
        } else {
            console.log('🔍 Database status check via direct invoke...');
            result = await invoke('check_database_connection') as any;
            console.log('🔍 Database status check result:', result);
        }

        const statusDot = document.getElementById('db-status');
        const statusText = document.getElementById('db-status-text');

        if (result && result.connected) {
            if (statusDot) statusDot.classList.add('connected');
            if (statusText) statusText.textContent = `Database: ${result.database || 'Connected'}`;
            console.log('✅ Database connection confirmed from frontend');
        } else {
            if (statusDot) statusDot.classList.remove('connected');
            if (statusText) statusText.textContent = 'Database: Disconnected';
            console.log('❌ Database connection failed');
        }
    } catch (error) {
        console.error('❌ Database status check failed:', error);
        const statusDot = document.getElementById('db-status');
        const statusText = document.getElementById('db-status-text');
        if (statusDot) statusDot.classList.remove('connected');
        if (statusText) statusText.textContent = 'Database: Error';
    }
}

// Test functions for debugging
async function testDatabaseConnection(): Promise<void> {
    try {
        console.log('🔍 Testing database connection...');
        const result = await invoke('check_database_connection');
        console.log('✅ Database connection test result:', result);
        alert('Database test successful! Check console for details.');
        await checkDatabaseStatus(); // Update status after test
    } catch (error) {
        console.error('❌ Database connection test failed:', error);
        alert(`Database test failed: ${error}`);
        await checkDatabaseStatus(); // Update status after test
    }
}

function testMonacoEditor(): void {
    try {
        console.log('🔍 Testing Monaco Editor...');
        if (editor) {
            const value = editor.getValue();
            console.log('✅ Monaco Editor is working, current value:', value);
            editor.setValue('// Monaco Editor Test - ' + new Date().toISOString());
            alert('Monaco Editor test successful! Value updated.');
        } else {
            console.error('❌ Monaco Editor not initialized');
            alert('Monaco Editor not initialized');
        }
    } catch (error) {
        console.error('❌ Monaco Editor test failed:', error);
        alert(`Monaco Editor test failed: ${error}`);
    }
}

console.log('🔧 Global functions exported to window for rules editor');