// Update time
function updateTime() {
    document.getElementById('time').textContent = new Date().toLocaleString();
}

// Test JavaScript
window.testJS = function() {
    const output = document.getElementById('output');
    output.textContent = 'JavaScript is working!\n' +
                        'User Agent: ' + navigator.userAgent + '\n' +
                        'Window Size: ' + window.innerWidth + 'x' + window.innerHeight + '\n' +
                        'Timestamp: ' + Date.now();
}

// Test Tauri
window.testTauri = async function() {
    const output = document.getElementById('output');

    if (window.__TAURI__) {
        try {
            output.textContent = 'Calling Tauri API...\n';
            const result = await window.__TAURI__.invoke('get_test_rules');
            output.textContent = 'Success! Received ' + result.length + ' test rules:\n' +
                               JSON.stringify(result, null, 2);
        } catch (err) {
            output.textContent = 'Tauri API Error:\n' + err;
        }
    } else {
        output.textContent = 'Tauri API not available - running in browser mode';
    }
}

// Clear output
window.clearOutput = function() {
    document.getElementById('output').textContent = 'Cleared.';
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', function() {
    // Update time
    setInterval(updateTime, 1000);
    updateTime();

    // Function to check for Tauri API
    function checkTauriAPI() {
        const isTauri = window.__TAURI__ !== undefined;
        const userAgent = navigator.userAgent;
        const platform = navigator.platform;

        // Determine environment
        let statusText = '';
        if (isTauri) {
            statusText = '✅ Tauri Detected - Native App';
        } else if (userAgent.includes('Chrome') || userAgent.includes('Safari') || userAgent.includes('Firefox')) {
            statusText = '⚠️ Browser Mode - Open the Tauri App Window instead';
        } else {
            statusText = '⏳ WebView Loading... (Waiting for Tauri API)';
        }

        document.getElementById('status').innerHTML = statusText + '<br><small>' + userAgent.substring(0, 50) + '...</small>';

        // Log to console
        console.log('Data Designer loaded');
        console.log('Tauri available:', isTauri);
        console.log('User Agent:', userAgent);
        console.log('Platform:', platform);

        // If Tauri not found and we're in WebView, check again
        if (!isTauri && !userAgent.includes('Chrome') && !userAgent.includes('Safari') && !userAgent.includes('Firefox')) {
            setTimeout(checkTauriAPI, 500);
        }
    }

    // Initial check
    checkTauriAPI();
});