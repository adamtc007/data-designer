export class SharedDatabaseService {
    private isConnected: boolean = false;
    private connectionStatus: any = null;
    private initializationPromise: Promise<{ success: boolean; status: any }> | null = null;
    private connectionListeners = new Set<(connected: boolean, status: any) => void>();
    private retryCount = 0;
    private maxRetries = 3;
    private workingInvoke: any = null;

    async initialize(): Promise<{ success: boolean; status: any }> {
        if (this.initializationPromise) {
            return this.initializationPromise;
        }
        this.initializationPromise = this._performInitializationWithRetry();
        return this.initializationPromise;
    }

    private async _performInitializationWithRetry(): Promise<{ success: boolean; status: any }> {
        for (let retry = 0; retry <= this.maxRetries; retry++) {
            this.retryCount = retry;
            try {
                console.log(`🔄 Initialization attempt ${retry + 1}/${this.maxRetries + 1}`);
                const result = await this._performInitialization();
                console.log(`✅ Successfully initialized on attempt ${retry + 1}`);
                return result;
            } catch (error) {
                console.error(`❌ Initialization attempt ${retry + 1} failed:`, error);
                if (retry < this.maxRetries) {
                    const retryDelay = Math.min(1000 * Math.pow(2, retry), 5000);
                    console.log(`⏳ Retrying in ${retryDelay}ms...`);
                    await new Promise(resolve => setTimeout(resolve, retryDelay));
                } else {
                    console.error(`💥 All ${this.maxRetries + 1} initialization attempts failed`);
                    throw error;
                }
            }
        }
        throw new Error('Failed to initialize after all retries');
    }

    private async _performInitialization(): Promise<{ success: boolean; status: any }> {
        try {
            console.log('🔌 Initializing shared database connection...');
            console.log('🔍 Environment debug info:');
            console.log('  - User Agent:', navigator.userAgent);
            console.log('  - Window location:', window.location.href);
            console.log('  - Is Tauri defined?', typeof (window as any).__TAURI__ !== 'undefined');
            console.log('  - Document ready state:', document.readyState);

            // Special handling: If we're retrying, add extra delay for stability
            if (this.retryCount && this.retryCount > 0) {
                console.log(`🔄 Retry #${this.retryCount} - Adding extra stabilization delay...`);
                await new Promise(resolve => setTimeout(resolve, 1000));
            }

            // Wait for Tauri to be available
            let attempts = 0;
            const maxAttempts = 30;
            let invokeFunction = null;
            let tauriFound = false;

            while (attempts < maxAttempts) {
                console.log(`🔍 Attempt ${attempts + 1}/${maxAttempts}: Checking Tauri availability...`);

                const tauri = (window as any).__TAURI__;
                console.log('  - window.__TAURI__ type:', typeof tauri);
                console.log('  - window.__TAURI__ value:', tauri);
                console.log('  - Document ready state:', document.readyState);

                if (typeof tauri !== 'undefined' && tauri) {
                    console.log('  - Checking __TAURI__ properties:');
                    console.log('    - .tauri:', typeof tauri.tauri);
                    console.log('    - .invoke:', typeof tauri.invoke);
                    console.log('    - .core:', typeof tauri.core);
                    if (tauri.core) {
                        console.log('    - .core.invoke:', typeof tauri.core.invoke);
                    }

                    // Check for different Tauri API structures
                    invokeFunction = null;
                    tauriFound = false;

                    // Tauri v1 structure: window.__TAURI__.tauri.invoke
                    if (tauri.tauri && tauri.tauri.invoke) {
                        console.log('🎯 Found Tauri v1 API structure');
                        invokeFunction = tauri.tauri.invoke;
                        tauriFound = true;
                    }
                    // Tauri v2 structure: window.__TAURI__.invoke
                    else if (tauri.invoke) {
                        console.log('🎯 Found Tauri v2 API structure');
                        invokeFunction = tauri.invoke;
                        tauriFound = true;
                    }
                    // Tauri core structure: window.__TAURI__.core.invoke
                    else if (tauri.core && tauri.core.invoke) {
                        console.log('🎯 Found Tauri core API structure');
                        invokeFunction = tauri.core.invoke;
                        tauriFound = true;
                    }

                    if (tauriFound && invokeFunction) {
                        console.log('✅ Tauri environment found with invoke function!');
                        // Extra validation: try a simple Tauri call
                        try {
                            console.log('🧪 Testing Tauri invoke function...');
                            const testCall = await invokeFunction('check_database_connection');
                            console.log('✅ Tauri invoke test successful!', testCall);
                            this.workingInvoke = invokeFunction;
                            break;
                        } catch (testError) {
                            console.warn('⚠️ Tauri invoke test failed:', testError);
                            console.log('🔄 Will continue trying...');
                        }
                    }
                }

                console.log(`⏳ Waiting for Tauri environment... attempt ${attempts + 1}/${maxAttempts}`);
                await new Promise(resolve => setTimeout(resolve, 300));
                attempts++;
            }

            if (!tauriFound || !invokeFunction) {
                console.error('❌ Final Tauri check failed:');
                console.error('  - window.__TAURI__ type:', typeof (window as any).__TAURI__);
                console.error('  - window.__TAURI__ value:', (window as any).__TAURI__);
                console.error('  - Location:', window.location.href);
                console.error('  - Document ready state:', document.readyState);
                console.error('  - tauriFound:', tauriFound);
                console.error('  - invokeFunction:', invokeFunction);
                throw new Error('Tauri environment not available after waiting');
            }

            // Use the detected working invoke function
            const status = await invokeFunction('check_database_connection');
            if (status.connected) {
                this.isConnected = true;
                this.connectionStatus = status;
                console.log('✅ Shared database connection established:', status);
                this._notifyConnectionListeners(true, status);
                return { success: true, status };
            } else {
                throw new Error(`Database connection failed: ${status.error || 'Unknown error'}`);
            }
        } catch (error: any) {
            console.error('❌ Failed to initialize shared database connection:', error);
            this.isConnected = false;
            this.connectionStatus = { connected: false, error: error.message };
            this._notifyConnectionListeners(false, this.connectionStatus);
            throw error;
        }
    }

    getConnectionStatus() {
        return {
            isConnected: this.isConnected,
            status: this.connectionStatus
        };
    }

    addConnectionListener(callback: (connected: boolean, status: any) => void) {
        this.connectionListeners.add(callback);
        if (this.isConnected && this.connectionStatus) {
            callback(true, this.connectionStatus);
        }
        return () => this.connectionListeners.delete(callback);
    }

    private _notifyConnectionListeners(connected: boolean, status: any) {
        this.connectionListeners.forEach(callback => {
            try {
                callback(connected, status);
            } catch (error) {
                console.error('Error in connection listener:', error);
            }
        });
    }

    async invokeCommand(command: string, args: any = {}) {
        if (!this.isConnected) {
            throw new Error('Database connection not available. Please ensure the connection is initialized.');
        }
        try {
            return await this.workingInvoke(command, args);
        } catch (error) {
            console.error(`Error executing command '${command}':`, error);
            throw error;
        }
    }

    async getProducts() {
        return this.invokeCommand('list_products');
    }

    async getCbus() {
        return this.invokeCommand('list_cbus');
    }

    async getResources() {
        return this.invokeCommand('list_resources');
    }

    async testRule(dslText: string) {
        return this.invokeCommand('test_rule', { dslText });
    }

    async getRules() {
        return this.invokeCommand('db_get_all_rules');
    }

    async createProduct(productData: any) {
        return this.invokeCommand('create_product', productData);
    }

    async createCbu(cbuData: any) {
        return this.invokeCommand('create_cbu', cbuData);
    }

    async createResource(resourceData: any) {
        return this.invokeCommand('create_resource', resourceData);
    }

    async getDataDictionary() {
        return this.invokeCommand('dd_get_data_dictionary');
    }

    async waitForConnection(timeout = 10000) {
        if (this.isConnected && this.connectionStatus) {
            return this.connectionStatus;
        }

        return new Promise<any>((resolve, reject) => {
            const timeoutId = setTimeout(() => {
                reject(new Error('Database connection timeout'));
            }, timeout);

            const unsubscribe = this.addConnectionListener((connected, status) => {
                if (connected) {
                    clearTimeout(timeoutId);
                    unsubscribe();
                    resolve(status);
                }
            });
        });
    }
}

let sharedDbServiceInstance: SharedDatabaseService | null = null;

export function getSharedDbService(): SharedDatabaseService {
    if (!sharedDbServiceInstance) {
        sharedDbServiceInstance = new SharedDatabaseService();
        (window as any).sharedDbService = sharedDbServiceInstance;
        // Auto-initialize the service when first created
        sharedDbServiceInstance.initialize().catch(error => {
            console.error('Failed to auto-initialize shared database service:', error);
        });
    }
    return sharedDbServiceInstance;
}

export default SharedDatabaseService;