// Data Dictionary Integration for PostgreSQL
// This module handles loading and managing attributes from the database

let dataDictionaryCache = null;
let isLoading = false;

// Load data dictionary from PostgreSQL
async function loadDataDictionary() {
    if (isLoading) return dataDictionaryCache;
    isLoading = true;

    try {
        const invoke = window.__TAURI_INVOKE__ || (window.__TAURI__ && window.__TAURI__.invoke);
        if (!invoke) {
            console.warn('Tauri API not available, using defaults');
            return getDefaultAttributes();
        }

        console.log('Loading data dictionary from PostgreSQL...');
        const response = await invoke('dd_get_data_dictionary', {});
        dataDictionaryCache = response;

        console.log(`Loaded ${response.total_count} attributes from database`);

        // Update the UI
        updateAttributeList(response);

        // Update the attribute picker dialog if it exists
        updateAttributePicker(response);

        return response;
    } catch (error) {
        console.error('Failed to load data dictionary:', error);
        if (window.addToOutput) {
            window.addToOutput('error', `Failed to load data dictionary: ${error}`);
        }
        return getDefaultDataDictionary();
    } finally {
        isLoading = false;
    }
}

// Update the attribute list in the sidebar
function updateAttributeList(dataDictionary) {
    const attrList = document.getElementById('attribute-list');
    if (!attrList) return;

    attrList.innerHTML = '';

    // Iterate through entities
    for (const [entityName, groups] of Object.entries(dataDictionary.entities || {})) {
        // Add entity header
        const entityHeader = document.createElement('div');
        entityHeader.className = 'entity-header';
        entityHeader.style.cssText = 'margin: 10px 0 5px 0; color: #007acc; font-weight: bold;';
        entityHeader.textContent = entityName;
        attrList.appendChild(entityHeader);

        // Add business attributes
        if (groups.business && groups.business.length > 0) {
            const businessHeader = document.createElement('div');
            businessHeader.style.cssText = 'margin: 5px 0 3px 10px; color: #4ecdc4; font-size: 12px;';
            businessHeader.innerHTML = '📊 Business Attributes';
            attrList.appendChild(businessHeader);

            groups.business.forEach(attr => {
                const item = createAttributeListItem(attr, 'business');
                attrList.appendChild(item);
            });
        }

        // Add derived attributes
        if (groups.derived && groups.derived.length > 0) {
            const derivedHeader = document.createElement('div');
            derivedHeader.style.cssText = 'margin: 5px 0 3px 10px; color: #ffa500; font-size: 12px;';
            derivedHeader.innerHTML = '➡️ Derived Attributes';
            attrList.appendChild(derivedHeader);

            groups.derived.forEach(attr => {
                const item = createAttributeListItem(attr, 'derived');
                attrList.appendChild(item);
            });
        }

        // System attributes (collapsible)
        if (groups.system && groups.system.length > 0) {
            const systemToggle = document.createElement('div');
            systemToggle.style.cssText = 'margin: 5px 0 3px 10px; cursor: pointer; color: #888; font-size: 12px;';
            systemToggle.innerHTML = `▶ System Attributes (${groups.system.length})`;
            systemToggle.dataset.entity = entityName;
            systemToggle.dataset.expanded = 'false';

            systemToggle.onclick = function() {
                const expanded = this.dataset.expanded === 'true';
                const container = document.getElementById(`system-${entityName}`);

                if (expanded) {
                    container.style.display = 'none';
                    this.innerHTML = `▶ System Attributes (${groups.system.length})`;
                    this.dataset.expanded = 'false';
                } else {
                    container.style.display = 'block';
                    this.innerHTML = `▼ System Attributes (${groups.system.length})`;
                    this.dataset.expanded = 'true';
                }
            };

            attrList.appendChild(systemToggle);

            const systemContainer = document.createElement('div');
            systemContainer.id = `system-${entityName}`;
            systemContainer.style.display = 'none';

            groups.system.forEach(attr => {
                const item = createAttributeListItem(attr, 'system');
                item.style.marginLeft = '20px';
                systemContainer.appendChild(item);
            });

            attrList.appendChild(systemContainer);
        }
    }
}

// Create a single attribute list item
function createAttributeListItem(attr, type) {
    const item = document.createElement('li');
    item.className = 'attribute-item';
    item.style.cssText = 'padding: 4px 8px; cursor: pointer; list-style: none; border-radius: 4px; margin: 2px 0;';

    item.dataset.fullPath = attr.full_path;
    item.dataset.dataType = attr.data_type;
    item.dataset.rustType = attr.rust_type;
    item.dataset.attrType = type;

    const icon = type === 'business' ? '📊' : type === 'derived' ? '➡️' : '⚙️';
    const color = type === 'business' ? '#4ecdc4' : type === 'derived' ? '#ffa500' : '#888';

    item.innerHTML = `
        <span style="color: ${color}; margin-right: 5px;">${icon}</span>
        <span class="attr-name" style="color: #d4d4d4;">${attr.attribute_name}</span>
        <span class="attr-type" style="color: #888; font-size: 11px; margin-left: 5px;">${attr.data_type}</span>
    `;

    item.title = attr.description || `${attr.full_path} (${attr.rust_type})`;

    item.onmouseover = () => item.style.background = '#3e3e42';
    item.onmouseout = () => item.style.background = 'transparent';

    item.onclick = () => insertAttributeInEditor(attr.full_path);

    return item;
}

// Insert attribute into the editor
function insertAttributeInEditor(fullPath) {
    if (window.editor) {
        const position = window.editor.getPosition();
        window.editor.executeEdits('', [{
            range: new monaco.Range(position.lineNumber, position.column, position.lineNumber, position.column),
            text: fullPath,
            forceMoveMarkers: true
        }]);
        window.editor.focus();
    }
}

// Update the attribute picker dialog (for derived attribute creation)
function updateAttributePicker(dataDictionary) {
    // This will be called when creating new derived attributes
    window.dataDictionary = dataDictionary;
}

// Get business attributes for the picker
window.getBusinessAttributes = function() {
    if (dataDictionaryCache) {
        const attrs = [];
        for (const [entityName, groups] of Object.entries(dataDictionaryCache.entities || {})) {
            // Collect business and system attributes as source attributes
            [...(groups.business || []), ...(groups.system || [])].forEach(attr => {
                attrs.push({
                    name: attr.full_path,
                    type: attr.data_type,
                    rustType: attr.rust_type,
                    description: attr.description
                });
            });
        }
        return attrs;
    }
    return getDefaultBusinessAttributes();
};

// Search attributes
async function searchAttributes(searchTerm) {
    try {
        const invoke = window.__TAURI_INVOKE__ || (window.__TAURI__ && window.__TAURI__.invoke);
        if (!invoke) return [];

        const results = await invoke('dd_search_attributes', { searchTerm });
        return results;
    } catch (error) {
        console.error('Failed to search attributes:', error);
        return [];
    }
}

// Refresh data dictionary from database
async function refreshDataDictionary() {
    try {
        const invoke = window.__TAURI_INVOKE__ || (window.__TAURI__ && window.__TAURI__.invoke);
        if (!invoke) return;

        await invoke('dd_refresh_data_dictionary', {});
        await loadDataDictionary();

        if (window.addToOutput) {
            window.addToOutput('success', 'Data dictionary refreshed from database');
        }
    } catch (error) {
        console.error('Failed to refresh data dictionary:', error);
        if (window.addToOutput) {
            window.addToOutput('error', `Failed to refresh: ${error}`);
        }
    }
}

// Save rule with compilation
async function saveRuleWithCompilation(ruleName, dslCode, dependencies) {
    try {
        const invoke = window.__TAURI_INVOKE__ || (window.__TAURI__ && window.__TAURI__.invoke);
        if (!invoke) {
            throw new Error('Tauri API not available');
        }

        // Detect return type from DSL code
        const dataType = detectReturnType(dslCode);

        // Create derived attribute first
        const attrRequest = {
            entity_name: 'Client', // Default to Client, could be made configurable
            attribute_name: ruleName,
            data_type: dataType,
            description: `Derived attribute calculated by rule: ${ruleName}`,
            dependencies: dependencies
        };

        const attributeId = await invoke('dd_create_derived_attribute', { request: attrRequest });

        // Create and compile the rule
        const result = await invoke('dd_create_and_compile_rule', {
            ruleName,
            dslCode,
            targetAttributeId: attributeId,
            dependencies
        });

        console.log('Rule compiled successfully:', result);

        if (window.addToOutput) {
            window.addToOutput('success', `Rule "${ruleName}" saved and compiled successfully`);
        }

        // Refresh data dictionary to show new derived attribute
        await loadDataDictionary();

        return result;
    } catch (error) {
        console.error('Failed to save rule:', error);
        if (window.addToOutput) {
            window.addToOutput('error', `Failed to save rule: ${error}`);
        }
        throw error;
    }
}

// Detect return type from DSL code
function detectReturnType(dslCode) {
    const code = dslCode.toLowerCase();

    if (code.includes('true') || code.includes('false') ||
        code.includes('is_') || code.includes('has_') ||
        code.includes('>') || code.includes('<') || code.includes('==')) {
        return 'Boolean';
    }

    if (code.includes('concat') || code.includes('"') || code.includes("'") ||
        code.includes('substring')) {
        return 'String';
    }

    if (code.includes('+') || code.includes('-') || code.includes('*') ||
        code.includes('/') || code.includes('sum') || code.includes('avg')) {
        return 'Number';
    }

    return 'String'; // Default
}

// Default attributes if database is not available
function getDefaultBusinessAttributes() {
    return [
        { name: 'Client.client_id', type: 'String', rustType: 'String' },
        { name: 'Client.legal_entity_name', type: 'String', rustType: 'String' },
        { name: 'Client.lei_code', type: 'String', rustType: 'String' },
        { name: 'Client.email', type: 'String', rustType: 'String' },
        { name: 'Client.country_code', type: 'String', rustType: 'String' },
        { name: 'Client.risk_rating', type: 'Number', rustType: 'i32' },
        { name: 'Client.pep_status', type: 'Boolean', rustType: 'bool' },
        { name: 'Client.kyc_status', type: 'String', rustType: 'String' },
        { name: 'Client.aum_usd', type: 'Number', rustType: 'f64' },
        { name: 'Client.onboarding_date', type: 'Date', rustType: 'NaiveDate' }
    ];
}

function getDefaultDataDictionary() {
    return {
        entities: {
            Client: {
                business: getDefaultBusinessAttributes().map(attr => ({
                    attribute_type: 'business',
                    entity_name: 'Client',
                    attribute_name: attr.name.split('.')[1],
                    full_path: attr.name,
                    data_type: attr.type,
                    rust_type: attr.rustType,
                    description: null,
                    required: true,
                    status: 'active'
                })),
                derived: [],
                system: []
            }
        },
        total_count: 10
    };
}

// Export functions for global use
window.loadDataDictionary = loadDataDictionary;
window.refreshDataDictionary = refreshDataDictionary;
window.searchAttributes = searchAttributes;
window.saveRuleWithCompilation = saveRuleWithCompilation;

// Auto-load on page ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        setTimeout(loadDataDictionary, 500); // Small delay to ensure Tauri is ready
    });
} else {
    setTimeout(loadDataDictionary, 500);
}