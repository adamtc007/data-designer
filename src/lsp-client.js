/**
 * Language Server Protocol Client for DSL IDE
 * Handles communication with the DSL Language Server via WebSocket or HTTP
 */

class LSPClient {
    constructor(serverUrl = 'ws://localhost:3030') {
        this.serverUrl = serverUrl;
        this.connection = null;
        this.requestId = 0;
        this.pendingRequests = new Map();
        this.initialized = false;
        this.capabilities = {};
        this.diagnosticsCallback = null;
        this.connectionStatusCallback = null;
    }

    /**
     * Connect to the Language Server
     */
    async connect() {
        return new Promise((resolve, reject) => {
            try {
                this.connection = new WebSocket(this.serverUrl);

                this.connection.onopen = async () => {
                    console.log('LSP WebSocket connection established');
                    if (this.connectionStatusCallback) {
                        this.connectionStatusCallback('connected');
                    }

                    // Initialize the LSP session
                    await this.initialize();
                    resolve();
                };

                this.connection.onmessage = (event) => {
                    this.handleMessage(JSON.parse(event.data));
                };

                this.connection.onerror = (error) => {
                    console.error('LSP WebSocket error:', error);
                    if (this.connectionStatusCallback) {
                        this.connectionStatusCallback('error');
                    }
                    reject(error);
                };

                this.connection.onclose = () => {
                    console.log('LSP WebSocket connection closed');
                    if (this.connectionStatusCallback) {
                        this.connectionStatusCallback('disconnected');
                    }
                    this.initialized = false;
                };

            } catch (error) {
                console.error('Failed to connect to LSP:', error);
                reject(error);
            }
        });
    }

    /**
     * Initialize LSP session
     */
    async initialize() {
        const initParams = {
            processId: null,
            clientInfo: {
                name: "DSL IDE",
                version: "1.0.0"
            },
            capabilities: {
                textDocument: {
                    synchronization: {
                        dynamicRegistration: false,
                        willSave: false,
                        willSaveWaitUntil: false,
                        didSave: true
                    },
                    completion: {
                        dynamicRegistration: false,
                        completionItem: {
                            snippetSupport: true,
                            documentationFormat: ["markdown", "plaintext"]
                        }
                    },
                    hover: {
                        dynamicRegistration: false,
                        contentFormat: ["markdown", "plaintext"]
                    },
                    publishDiagnostics: {
                        relatedInformation: true
                    },
                    semanticTokens: {
                        dynamicRegistration: false,
                        requests: {
                            full: true
                        },
                        tokenTypes: [
                            "keyword", "operator", "string", "number",
                            "variable", "function", "comment"
                        ],
                        tokenModifiers: []
                    }
                }
            },
            rootUri: null,
            workspaceFolders: null
        };

        const result = await this.sendRequest('initialize', initParams);
        this.capabilities = result.capabilities;
        this.initialized = true;

        // Send initialized notification
        await this.sendNotification('initialized', {});

        return result;
    }

    /**
     * Send a request to the Language Server
     */
    sendRequest(method, params) {
        return new Promise((resolve, reject) => {
            const id = ++this.requestId;
            const message = {
                jsonrpc: "2.0",
                id: id,
                method: method,
                params: params
            };

            this.pendingRequests.set(id, { resolve, reject });
            this.connection.send(JSON.stringify(message));

            // Timeout after 10 seconds
            setTimeout(() => {
                if (this.pendingRequests.has(id)) {
                    this.pendingRequests.delete(id);
                    reject(new Error(`Request ${method} timed out`));
                }
            }, 10000);
        });
    }

    /**
     * Send a notification to the Language Server
     */
    sendNotification(method, params) {
        const message = {
            jsonrpc: "2.0",
            method: method,
            params: params
        };
        this.connection.send(JSON.stringify(message));
    }

    /**
     * Handle incoming messages from the Language Server
     */
    handleMessage(message) {
        // Handle responses to requests
        if (message.id !== undefined && this.pendingRequests.has(message.id)) {
            const { resolve, reject } = this.pendingRequests.get(message.id);
            this.pendingRequests.delete(message.id);

            if (message.error) {
                reject(new Error(message.error.message));
            } else {
                resolve(message.result);
            }
        }
        // Handle notifications from server
        else if (message.method) {
            this.handleNotification(message.method, message.params);
        }
    }

    /**
     * Handle notifications from the Language Server
     */
    handleNotification(method, params) {
        switch (method) {
            case 'textDocument/publishDiagnostics':
                if (this.diagnosticsCallback) {
                    this.diagnosticsCallback(params.uri, params.diagnostics);
                }
                break;

            case 'window/logMessage':
                console.log(`LSP [${params.type}]:`, params.message);
                break;

            case 'window/showMessage':
                console.info('LSP Message:', params.message);
                break;

            default:
                console.log('Unhandled notification:', method, params);
        }
    }

    /**
     * Open a document in the Language Server
     */
    async openDocument(uri, languageId, version, text) {
        await this.sendNotification('textDocument/didOpen', {
            textDocument: {
                uri: uri,
                languageId: languageId,
                version: version,
                text: text
            }
        });
    }

    /**
     * Update a document in the Language Server
     */
    async changeDocument(uri, version, text) {
        await this.sendNotification('textDocument/didChange', {
            textDocument: {
                uri: uri,
                version: version
            },
            contentChanges: [
                {
                    text: text
                }
            ]
        });
    }

    /**
     * Request completions at a specific position
     */
    async getCompletions(uri, line, character) {
        if (!this.initialized) return [];

        try {
            const result = await this.sendRequest('textDocument/completion', {
                textDocument: { uri: uri },
                position: { line: line, character: character }
            });

            return Array.isArray(result) ? result : result?.items || [];
        } catch (error) {
            console.error('Failed to get completions:', error);
            return [];
        }
    }

    /**
     * Request hover information at a specific position
     */
    async getHover(uri, line, character) {
        if (!this.initialized) return null;

        try {
            return await this.sendRequest('textDocument/hover', {
                textDocument: { uri: uri },
                position: { line: line, character: character }
            });
        } catch (error) {
            console.error('Failed to get hover info:', error);
            return null;
        }
    }

    /**
     * Request semantic tokens for syntax highlighting
     */
    async getSemanticTokens(uri) {
        if (!this.initialized) return null;

        try {
            return await this.sendRequest('textDocument/semanticTokens/full', {
                textDocument: { uri: uri }
            });
        } catch (error) {
            console.error('Failed to get semantic tokens:', error);
            return null;
        }
    }

    /**
     * Execute a command on the Language Server
     */
    async executeCommand(command, args) {
        if (!this.initialized) return null;

        try {
            return await this.sendRequest('workspace/executeCommand', {
                command: command,
                arguments: args
            });
        } catch (error) {
            console.error('Failed to execute command:', error);
            return null;
        }
    }

    /**
     * Shutdown the Language Server
     */
    async shutdown() {
        if (this.initialized) {
            await this.sendRequest('shutdown', {});
            this.sendNotification('exit', {});
            this.initialized = false;
        }
    }

    /**
     * Disconnect from the Language Server
     */
    disconnect() {
        if (this.connection) {
            this.shutdown().then(() => {
                this.connection.close();
                this.connection = null;
            });
        }
    }

    /**
     * Set callback for diagnostics updates
     */
    onDiagnostics(callback) {
        this.diagnosticsCallback = callback;
    }

    /**
     * Set callback for connection status changes
     */
    onConnectionStatus(callback) {
        this.connectionStatusCallback = callback;
    }

    /**
     * Check if connected to the Language Server
     */
    isConnected() {
        return this.connection && this.connection.readyState === WebSocket.OPEN;
    }

    /**
     * Check if LSP session is initialized
     */
    isInitialized() {
        return this.initialized;
    }
}

// Export for use in browser or Node.js
if (typeof module !== 'undefined' && module.exports) {
    module.exports = LSPClient;
} else {
    window.LSPClient = LSPClient;
}