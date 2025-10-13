// UI Components and Panel Management

export class PanelManager {
    private undockedWindows: Map<string, Window> = new Map();

    // Undock a panel to a separate window
    undockPanel(panelId: string): void {
        const panel = document.getElementById(panelId);
        if (!panel) {
            console.error(`Panel ${panelId} not found`);
            return;
        }

        // Create new window
        const newWindow = window.open('', '_blank', 'width=800,height=600,scrollbars=yes,resizable=yes');
        if (!newWindow) {
            console.error('Failed to create new window');
            return;
        }

        // Set up the new window
        newWindow.document.write(`
            <!DOCTYPE html>
            <html>
            <head>
                <title>Undocked Panel - ${panelId}</title>
                <style>
                    body {
                        margin: 0;
                        padding: 0;
                        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                        background-color: #1e1e1e;
                        color: #d4d4d4;
                    }
                    .undocked-panel {
                        height: 100vh;
                        width: 100vw;
                        overflow: auto;
                    }
                </style>
            </head>
            <body>
                <div class="undocked-panel" id="undocked-content"></div>
            </body>
            </html>
        `);
        newWindow.document.close();

        // Move panel content to new window
        const panelContent = panel.innerHTML;
        const newContent = newWindow.document.getElementById('undocked-content');
        if (newContent) {
            newContent.innerHTML = panelContent;
        }

        // Hide original panel
        panel.style.display = 'none';

        // Store reference
        this.undockedWindows.set(panelId, newWindow);

        // Handle window close
        newWindow.addEventListener('beforeunload', () => {
            this.redockPanel(panelId);
        });

        console.log(`📱 Panel ${panelId} undocked to new window`);
    }

    // Redock a panel back to the main window
    redockPanel(panelId: string): void {
        const undockedWindow = this.undockedWindows.get(panelId);
        if (!undockedWindow) return;

        const panel = document.getElementById(panelId);
        if (panel) {
            // Restore original panel
            panel.style.display = '';

            // Get content from undocked window
            const undockedContent = undockedWindow.document.getElementById('undocked-content');
            if (undockedContent) {
                panel.innerHTML = undockedContent.innerHTML;
            }
        }

        // Clean up
        undockedWindow.close();
        this.undockedWindows.delete(panelId);

        console.log(`📱 Panel ${panelId} redocked to main window`);
    }

    // Check if panel is undocked
    isUndocked(panelId: string): boolean {
        return this.undockedWindows.has(panelId);
    }

    // Close all undocked windows
    closeAllUndocked(): void {
        this.undockedWindows.forEach((window, panelId) => {
            this.redockPanel(panelId);
        });
    }
}

// Global panel manager instance
export const panelManager = new PanelManager();

// Editor Panel Header Component
export function createEditorPanelHeader(panelId: string, title: string): string {
    return `
        <div class="editor-panel-header">
            <div class="panel-title">
                <span class="panel-icon">📝</span>
                <span class="panel-name">${title}</span>
            </div>
            <div class="panel-controls">
                <button class="panel-control-btn" onclick="togglePanelSize('${panelId}')" title="Toggle Size">
                    <span class="control-icon">⬌</span>
                </button>
                <button class="panel-control-btn" onclick="undockPanel('${panelId}')" title="Undock Panel">
                    <span class="control-icon">🗗</span>
                </button>
                <button class="panel-control-btn" onclick="minimizePanel('${panelId}')" title="Minimize">
                    <span class="control-icon">−</span>
                </button>
            </div>
        </div>
    `;
}

// Panel control functions
export function togglePanelSize(panelId: string): void {
    const panel = document.getElementById(panelId);
    if (!panel) return;

    panel.classList.toggle('maximized');
    console.log(`📐 Panel ${panelId} size toggled`);
}

export function undockPanel(panelId: string): void {
    panelManager.undockPanel(panelId);
}

export function minimizePanel(panelId: string): void {
    const panel = document.getElementById(panelId);
    if (!panel) return;

    panel.classList.toggle('minimized');
    console.log(`📉 Panel ${panelId} minimized`);
}

// Data Dictionary Sidebar Component
export function createDataDictionarySidebar(): string {
    return `
        <div id="data-dictionary-sidebar" class="sidebar">
            <div class="sidebar-header">
                <h3>Data Dictionary</h3>
                <button class="btn btn-small btn-primary" onclick="createNewAttribute()">+ New</button>
            </div>
            <div class="sidebar-content">
                <div class="attribute-search">
                    <input type="text" id="attribute-search" placeholder="Search attributes..." />
                </div>
                <div class="attribute-categories" id="attribute-list">
                    <div class="loading">Loading attributes...</div>
                </div>
            </div>
        </div>
    `;
}

// Attribute creation modal
export function createNewAttribute(): void {
    const modalHtml = `
        <div class="modal" id="create-attribute-modal" style="display: block;">
            <div class="modal-content">
                <div class="modal-header">
                    <h3>Create New Derived Attribute</h3>
                    <span class="close" onclick="closeModal('create-attribute-modal')">&times;</span>
                </div>
                <div class="modal-body">
                    <form id="attribute-form">
                        <div class="form-group">
                            <label for="attr-name">Attribute Name *</label>
                            <input type="text" id="attr-name" required placeholder="e.g., risk_score">
                        </div>
                        <div class="form-group">
                            <label for="attr-type">Return Type *</label>
                            <select id="attr-type" required>
                                <option value="">Select type</option>
                                <option value="String">String</option>
                                <option value="Number">Number</option>
                                <option value="Boolean">Boolean</option>
                                <option value="List">List</option>
                            </select>
                        </div>
                        <div class="form-group">
                            <label for="attr-description">Description</label>
                            <textarea id="attr-description" rows="3" placeholder="Describe what this attribute computes"></textarea>
                        </div>
                        <div class="form-group">
                            <label>Dependencies (select source attributes):</label>
                            <div id="dependency-checkboxes" class="checkbox-list">
                                <div class="loading">Loading available attributes...</div>
                            </div>
                        </div>
                    </form>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-secondary" onclick="closeModal('create-attribute-modal')">Cancel</button>
                    <button type="button" class="btn btn-primary" onclick="submitAttribute()">Create Attribute</button>
                </div>
            </div>
        </div>
    `;

    document.body.insertAdjacentHTML('beforeend', modalHtml);
    loadAttributeOptions();
}

// Load attribute options for dependency selection
async function loadAttributeOptions(): Promise<void> {
    try {
        const attributes = await (window as any).invoke('dd_get_data_dictionary');
        const checkboxContainer = document.getElementById('dependency-checkboxes');
        if (!checkboxContainer || !attributes) return;

        const checkboxHtml = attributes.map((attr: any) => `
            <div class="checkbox-item">
                <input type="checkbox" id="dep-${attr.attribute_name}" value="${attr.attribute_name}">
                <label for="dep-${attr.attribute_name}">${attr.attribute_name} (${attr.data_type})</label>
            </div>
        `).join('');

        checkboxContainer.innerHTML = checkboxHtml;
    } catch (error) {
        console.error('Failed to load attribute options:', error);
    }
}

// Submit new attribute
export async function submitAttribute(): Promise<void> {
    const form = document.getElementById('attribute-form') as HTMLFormElement;
    if (!form) return;

    const name = (document.getElementById('attr-name') as HTMLInputElement).value;
    const type = (document.getElementById('attr-type') as HTMLSelectElement).value;
    const description = (document.getElementById('attr-description') as HTMLTextAreaElement).value;

    // Get selected dependencies
    const dependencies = Array.from(document.querySelectorAll('#dependency-checkboxes input:checked'))
        .map(cb => (cb as HTMLInputElement).value);

    try {
        const result = await (window as any).invoke('dd_create_derived_attribute', {
            name,
            returnType: type,
            description,
            dependencies
        });

        console.log('✅ Attribute created:', result);
        closeModal('create-attribute-modal');
        showSuccessMessage('Attribute created successfully!');

        // Generate rule template and load in editor
        generateRuleTemplate(name, type, dependencies);

    } catch (error) {
        console.error('❌ Failed to create attribute:', error);
        showErrorMessage(`Failed to create attribute: ${error}`);
    }
}

// Generate rule template based on attribute type and dependencies
function generateRuleTemplate(name: string, type: string, dependencies: string[]): void {
    let template = '';

    switch (type) {
        case 'Number':
            template = dependencies.length > 0
                ? `// Calculate ${name}\n${dependencies[0]} * 1.2 + ${dependencies[1] || '0'}`
                : `// Calculate ${name}\n100.0`;
            break;
        case 'String':
            template = dependencies.length > 0
                ? `// Generate ${name}\nCONCAT("${name.toUpperCase()}: ", ${dependencies[0]})`
                : `// Generate ${name}\n"${name.replace('_', ' ').toUpperCase()}"`;
            break;
        case 'Boolean':
            template = dependencies.length > 0
                ? `// Check ${name}\n${dependencies[0]} > 0 AND ${dependencies[1] || 'true'}`
                : `// Check ${name}\ntrue`;
            break;
        case 'List':
            template = dependencies.length > 0
                ? `// Build ${name}\n[${dependencies.map(d => `${d}`).join(', ')}]`
                : `// Build ${name}\n[]`;
            break;
        default:
            template = `// Define ${name}\n${dependencies.join(' + ') || 'value'}`;
    }

    // Set template in editor
    if ((window as any).editor) {
        (window as any).editor.setValue(template);
        (window as any).editor.focus();
    }
}

// Utility functions that need to be global
function closeModal(modalId: string): void {
    const modal = document.getElementById(modalId);
    if (modal) {
        modal.remove();
    }
}

function showSuccessMessage(message: string): void {
    console.log('✅', message);
}

function showErrorMessage(message: string): void {
    console.error('❌', message);
}

// Expose functions globally
(window as any).togglePanelSize = togglePanelSize;
(window as any).undockPanel = undockPanel;
(window as any).minimizePanel = minimizePanel;
(window as any).createNewAttribute = createNewAttribute;
(window as any).submitAttribute = submitAttribute;
(window as any).closeModal = closeModal;