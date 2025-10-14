import { getSharedDbService } from './shared-db-service.js';
import type { SharedDatabaseService } from './shared-db-service.js';

export class HomeManager {
    private sharedDbService: SharedDatabaseService | null = null;

    constructor() {
        console.log('🏠 HomeManager constructor called');
    }

    async initialize(): Promise<void> {
        try {
            console.log('🏠 Data Designer Home Page loaded');
            console.log('🔌 Initializing shared database connection...');

            this.sharedDbService = getSharedDbService();
            await this.sharedDbService.initialize();

            console.log('✅ Shared database connection initialized successfully');
            this.updateStatusIndicator(true);
        } catch (error) {
            console.error('❌ Failed to initialize shared database connection:', error);
            this.updateStatusIndicator(false, (error as Error).message);
        }
    }

    // Application launcher functions
    launchApplication(appType: string): void {
        console.log(`🚀 Launching ${appType} application...`);

        // Check if database connection is available before launching
        if (!this.sharedDbService?.getConnectionStatus().isConnected) {
            alert('Database connection not available. Please wait for initialization to complete.');
            return;
        }

        switch(appType) {
            case 'ide':
                // Force hard reload with cache bypass for Tauri webview
                const timestamp = Date.now();
                const random = Math.random().toString(36).substring(7);
                window.location.href = `index.html?t=${timestamp}&r=${random}&v=fresh&nocache=true`;
                break;
            case 'cbu':
                window.location.href = 'cbu-management.html';
                break;
            case 'products':
                window.location.href = 'products-management.html';
                break;
            case 'resources':
                window.location.href = 'resources-management.html';
                break;
            default:
                console.warn('Unknown application type:', appType);
                alert(`Application "${appType}" not yet implemented.`);
        }
    }

    // Quick action handlers
    quickAction(actionType: string): void {
        console.log(`⚡ Quick action: ${actionType}`);
        switch(actionType) {
            case 'database':
                this.checkDatabaseStatus();
                break;
            case 'backup':
                alert('System backup functionality - Coming soon!');
                break;
            case 'logs':
                alert('System logs viewer - Coming soon!');
                break;
            case 'settings':
                alert('System settings - Coming soon!');
                break;
            default:
                alert(`Quick action "${actionType}" not implemented.`);
        }
    }

    // Database status check
    async checkDatabaseStatus(): Promise<void> {
        try {
            console.log('🔍 Checking database status...');
            if (!this.sharedDbService) {
                throw new Error('Database service not initialized');
            }

            const connectionStatus = this.sharedDbService.getConnectionStatus();
            const status = connectionStatus.status;
            const statusColor = connectionStatus.isConnected ? '#4caf50' : '#f44336';
            const statusText = connectionStatus.isConnected ? 'Connected' : 'Disconnected';
            const statusIcon = connectionStatus.isConnected ? '✅' : '❌';

            const statusDialog = document.createElement('div');
            statusDialog.innerHTML = `
                <div style="
                    position: fixed; top: 0; left: 0; width: 100%; height: 100%;
                    background: rgba(0, 0, 0, 0.7); display: flex; align-items: center;
                    justify-content: center; z-index: 10000;">
                    <div style="
                        background: #2d2d30; border: 1px solid #3e3e42; border-radius: 12px;
                        padding: 2rem; max-width: 400px; text-align: center; color: #d4d4d4;">
                        <h3 style="color: ${statusColor}; margin-bottom: 1rem;">${statusIcon} Database Status</h3>
                        <p style="margin-bottom: 1rem;">
                            Database: <strong>data_designer</strong><br>
                            Status: <span style="color: ${statusColor};">${statusText}</span><br>
                            ${status && status.host ? `Host: ${status.host}` : 'Host: localhost:5432'}<br>
                            ${status && status.error ? `Error: ${status.error}` : ''}
                        </p>
                        <button onclick="this.closest('div').remove()" style="
                            background: #0e639c; color: white; border: none; padding: 8px 16px;
                            border-radius: 6px; cursor: pointer;">Close</button>
                    </div>
                </div>
            `;
            document.body.appendChild(statusDialog);
        } catch (error) {
            console.error('❌ Database status check failed:', error);
            alert('Database status check failed. Please check the console for details.');
        }
    }

    // Update the status indicator in the header
    updateStatusIndicator(connected: boolean, errorMessage: string = ''): void {
        const statusIndicator = document.querySelector('.status-indicator') as HTMLElement;
        const statusDot = document.querySelector('.status-dot') as HTMLElement;
        const statusText = statusIndicator?.querySelector('span') as HTMLElement;

        if (!statusIndicator || !statusDot || !statusText) return;

        if (connected) {
            statusIndicator.style.background = 'rgba(76, 175, 80, 0.1)';
            statusIndicator.style.borderColor = '#4caf50';
            statusDot.style.background = '#4caf50';
            statusText.textContent = 'Database Connected';
        } else {
            statusIndicator.style.background = 'rgba(244, 67, 54, 0.1)';
            statusIndicator.style.borderColor = '#f44336';
            statusDot.style.background = '#f44336';
            statusText.textContent = errorMessage ? `Connection Failed: ${errorMessage}` : 'Database Disconnected';
        }
    }
}

// Make functions available globally for HTML onclick handlers
declare global {
    interface Window {
        HomeManager: typeof HomeManager;
        homeManager?: HomeManager;
        launchApplication: (appType: string) => void;
        quickAction: (actionType: string) => void;
    }
}

window.HomeManager = HomeManager;

// Global functions that delegate to the homeManager instance
window.launchApplication = (appType: string) => {
    window.homeManager?.launchApplication(appType);
};

window.quickAction = (actionType: string) => {
    window.homeManager?.quickAction(actionType);
};