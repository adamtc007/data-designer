// Simple version without Monaco Editor for debugging

// Check if Tauri API is available
let isTauriAvailable = false;

// Initialize Tauri
function initTauri() {
    if (typeof window !== 'undefined' && window.__TAURI__) {
        console.log('Tauri object found:', window.__TAURI__);
        console.log('Available Tauri properties:', Object.keys(window.__TAURI__));

        // Check specifically for invoke
        if (window.__TAURI__.invoke) {
            isTauriAvailable = true;
            console.log('✅ Running in Tauri application with invoke');
            document.getElementById('status').textContent = '✅ Tauri Connected';
            document.getElementById('status').style.color = '#4caf50';
            return true;
        } else if (window.__TAURI__.tauri && window.__TAURI__.tauri.invoke) {
            // Sometimes invoke is nested under tauri
            window.__TAURI__.invoke = window.__TAURI__.tauri.invoke;
            isTauriAvailable = true;
            console.log('✅ Found invoke under window.__TAURI__.tauri');
            document.getElementById('status').textContent = '✅ Tauri Connected';
            document.getElementById('status').style.color = '#4caf50';
            return true;
        } else if (window.__TAURI__.core) {
            // Try core.invoke pattern
            window.__TAURI__.invoke = window.__TAURI__.core.invoke;
            isTauriAvailable = true;
            console.log('✅ Found invoke under window.__TAURI__.core');
            document.getElementById('status').textContent = '✅ Tauri Connected';
            document.getElementById('status').style.color = '#4caf50';
            return true;
        } else {
            console.error('Tauri object exists but invoke not found');
            document.getElementById('status').textContent = '⚠️ Tauri Incomplete';
            document.getElementById('status').style.color = '#ff9800';
            return false;
        }
    } else {
        console.warn('⚠️ Running in browser mode');
        document.getElementById('status').textContent = '⚠️ Browser Mode';
        document.getElementById('status').style.color = '#ff9800';
        return false;
    }
}

// Load test rules
async function loadTestRules() {
    try {
        console.log('Loading test rules...');
        // Make sure we have invoke function
        if (!window.__TAURI__ || !window.__TAURI__.invoke) {
            console.error('Tauri API not available');
            document.getElementById('rulesLoaded').textContent = 'Tauri API not available';
            return;
        }

        const rules = await window.__TAURI__.invoke('get_test_rules');
        console.log('Received rules:', rules);

        const select = document.getElementById('ruleSelect');
        if (!select) {
            console.error('ruleSelect element not found');
            return;
        }

        select.innerHTML = '<option value="">Select a test rule...</option>';

        if (rules && rules.length > 0) {
            rules.forEach(rule => {
                const option = document.createElement('option');
                option.value = rule.dsl;
                option.textContent = `${rule.name} - ${rule.description}`;
                option.dataset.id = rule.id;
                select.appendChild(option);
            });
            console.log(`Loaded ${rules.length} test rules`);
            document.getElementById('rulesLoaded').textContent = `${rules.length} rules loaded`;
        } else {
            console.warn('No rules received from backend');
            document.getElementById('rulesLoaded').textContent = 'No rules available';
        }
    } catch (error) {
        console.error('Failed to load test rules:', error);
        document.getElementById('rulesLoaded').textContent = 'Failed to load rules: ' + error;
    }
}

// Test selected rule
async function testRule() {
    const ruleText = document.getElementById('ruleEditor').value;
    const resultsDiv = document.getElementById('testResults');
    const resultContent = document.getElementById('resultContent');

    if (!ruleText) {
        alert('Please enter or select a rule to test');
        return;
    }

    resultsDiv.style.display = 'block';
    resultContent.innerHTML = 'Testing rule...';

    try {
        if (!window.__TAURI__ || !window.__TAURI__.invoke) {
            throw new Error('Tauri API not available');
        }
        const result = await window.__TAURI__.invoke('test_rule', { dslText: ruleText });

        if (result.success) {
            resultContent.innerHTML = `
                <div class="result-success">✅ Test Passed</div>
                <div class="rule-dsl">Result: ${JSON.stringify(result.result, null, 2)}</div>
            `;
        } else {
            resultContent.innerHTML = `
                <div class="result-error">❌ Test Failed</div>
                <div class="rule-dsl">Error: ${result.error || 'Unknown error'}</div>
            `;
        }
    } catch (error) {
        resultContent.innerHTML = `
            <div class="result-error">❌ Error</div>
            <div class="rule-dsl">Error: ${error}</div>
        `;
    }
}

// Handle tab switching
function switchTab(tabName) {
    // Hide all tabs
    document.querySelectorAll('.tab-content').forEach(tab => {
        tab.classList.remove('active');
    });

    // Remove active from all buttons
    document.querySelectorAll('.tab-button').forEach(btn => {
        btn.classList.remove('active');
    });

    // Show selected tab
    document.getElementById(tabName + 'Tab').classList.add('active');

    // Mark button as active
    document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', function() {
    console.log('Initializing Data Designer...');

    // Add status indicator
    const container = document.getElementById('container');
    if (container) {
        container.innerHTML = `
            <div style="padding: 20px; background: #2d2d30; border-radius: 5px;">
                <h3>System Status</h3>
                <p>Tauri Status: <span id="status">Checking...</span></p>
                <p>Rules: <span id="rulesLoaded">Loading...</span></p>
                <button id="reloadRulesBtn" style="background: #0e639c; color: white; border: none; padding: 8px 16px; border-radius: 3px; cursor: pointer; margin: 10px 5px;">Reload Rules</button>
                <button id="inspectTauriBtn" style="background: #ff9800; color: white; border: none; padding: 8px 16px; border-radius: 3px; cursor: pointer; margin: 10px 5px;">Inspect Tauri</button>
                <div style="margin-top: 20px;">
                    <h4>DSL Rule Editor</h4>
                    <textarea id="ruleEditor" style="width: 100%; height: 200px; background: #1e1e1e; color: #d4d4d4; border: 1px solid #3c3c3c; padding: 10px; font-family: monospace; font-size: 14px;" placeholder="Enter your DSL rule here or select from dropdown..."></textarea>
                </div>
            </div>
        `;

        // Add reload button handler
        const reloadBtn = document.getElementById('reloadRulesBtn');
        if (reloadBtn) {
            reloadBtn.addEventListener('click', () => {
                console.log('Manual reload triggered');
                loadTestRules();
            });
        }

        // Add inspect button handler
        const inspectBtn = document.getElementById('inspectTauriBtn');
        if (inspectBtn) {
            inspectBtn.addEventListener('click', () => {
                console.log('=== Tauri Inspection ===');
                console.log('window.__TAURI__ exists?', !!window.__TAURI__);
                if (window.__TAURI__) {
                    console.log('Tauri object:', window.__TAURI__);
                    console.log('Properties:', Object.keys(window.__TAURI__));

                    // Try to find invoke in different places
                    console.log('Direct invoke?', typeof window.__TAURI__.invoke);
                    console.log('tauri.invoke?', window.__TAURI__.tauri ? typeof window.__TAURI__.tauri.invoke : 'no tauri property');
                    console.log('core.invoke?', window.__TAURI__.core ? typeof window.__TAURI__.core.invoke : 'no core property');

                    // Show in UI
                    alert('Tauri inspection logged to console. Properties: ' + Object.keys(window.__TAURI__).join(', '));
                } else {
                    alert('window.__TAURI__ is not defined');
                }
            });
        }
    }

    // Initialize Tauri with retry
    let retryCount = 0;
    function tryInitTauri() {
        if (initTauri()) {
            // Add a small delay to ensure everything is ready
            setTimeout(() => {
                console.log('Tauri ready, loading rules...');
                loadTestRules();
            }, 100);
        } else if (retryCount < 5) {
            retryCount++;
            console.log(`Retrying Tauri init... (${retryCount}/5)`);
            setTimeout(tryInitTauri, 500);
        } else {
            console.log('Using browser mode fallback');
            loadTestRules(); // Load mock rules
        }
    }
    tryInitTauri();

    // Setup event listeners
    const ruleSelect = document.getElementById('ruleSelect');
    if (ruleSelect) {
        ruleSelect.addEventListener('change', function() {
            const editor = document.getElementById('ruleEditor');
            if (editor && this.value) {
                editor.value = this.value;
                document.getElementById('testButton').disabled = false;
            }
        });
    }

    const testButton = document.getElementById('testButton');
    if (testButton) {
        testButton.addEventListener('click', testRule);
    }

    // Setup tab buttons
    document.querySelectorAll('.tab-button').forEach(button => {
        button.addEventListener('click', function() {
            switchTab(this.dataset.tab);
        });
    });

    console.log('Data Designer initialized');
});

// Export for console debugging
window.debugDSL = {
    invoke: () => window.__TAURI__ ? window.__TAURI__.invoke : null,
    isTauriAvailable: () => isTauriAvailable,
    testRule,
    loadTestRules,
    getTauri: () => window.__TAURI__
};