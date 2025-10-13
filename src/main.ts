import { invoke } from '@tauri-apps/api/core';
import * as monaco from 'monaco-editor';
import { createEditorPanelHeader, panelManager } from './ui-components';

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

// Data Dictionary
async function loadDataDictionary(): Promise<void> {
    try {
        const attributes = await invoke('dd_get_data_dictionary');
        console.log('📚 Data dictionary loaded:', attributes);
        // Implementation to populate sidebar
    } catch (error) {
        console.error('Failed to load data dictionary:', error);
    }
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

// Expose functions globally for HTML onclick handlers
(window as any).closeModal = closeModal;
(window as any).submitCBU = submitCBU;
(window as any).runCode = runCode;
(window as any).saveRule = saveRule;
(window as any).showAST = showAST;