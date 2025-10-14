import { getSharedDbService } from './shared-db-service.js';
import type { SharedDatabaseService } from './shared-db-service.js';

interface Product {
    id: number;
    name: string;
    status: string;
    description?: string;
    category?: string;
    created_at?: string;
    updated_at?: string;
}

export class ProductServicesManager {
    private sharedDbService: SharedDatabaseService | null = null;
    private selectedProductId: number | null = null;

    constructor() {
        console.log('🏗️ ProductServicesManager constructor called');
    }

    async initialize(): Promise<void> {
        try {
            console.log('🏗️ Initializing ProductServicesManager...');
            this.sharedDbService = getSharedDbService();

            // Wait for the shared database service to be ready
            if (!this.sharedDbService.getConnectionStatus().isConnected) {
                console.log('⏳ Waiting for database connection...');
                await this.sharedDbService.waitForConnection();
            }

            console.log('✅ ProductServicesManager initialized');
            await this.loadProductsAndServices();
        } catch (error) {
            console.error('❌ Failed to initialize ProductServicesManager:', error);
            this.showError('Failed to initialize: ' + (error as Error).message);
        }
    }

    async loadProductsAndServices(): Promise<void> {
        try {
            console.log('📦 Loading products and services...');
            if (!this.sharedDbService) {
                throw new Error('Database service not initialized');
            }

            const products = await this.sharedDbService.getProducts();
            console.log('✅ Products loaded:', products);

            this.updateProductsList(products);
        } catch (error) {
            console.error('❌ Failed to load products:', error);
            this.showError('Failed to load products and services: ' + (error as Error).message);
        }
    }

    updateProductsList(products: Product[]): void {
        const itemsList = document.getElementById('itemsList');
        if (!itemsList) return;

        if (!products || products.length === 0) {
            itemsList.innerHTML = `
                <div class="empty-state">
                    <h3>No Products Found</h3>
                    <p>Create your first product to get started.</p>
                </div>
            `;
            return;
        }

        itemsList.innerHTML = products.map(product => `
            <div class="item" onclick="window.productServicesManager?.selectProduct(${product.id})" data-product-id="${product.id}">
                <div class="item-name">${this.escapeHtml(product.name || 'Unnamed Product')}</div>
                <div class="item-id">ID: ${product.id}</div>
                <div class="item-status status-${product.status || 'inactive'}">${product.status || 'inactive'}</div>
            </div>
        `).join('');
    }

    selectProduct(productId: number): void {
        try {
            console.log('📦 Selected product:', productId);
            this.selectedProductId = productId;

            // Update UI to show selected state
            document.querySelectorAll('.item').forEach(item => {
                item.classList.remove('selected');
                if (item.getAttribute('data-product-id') === productId.toString()) {
                    item.classList.add('selected');
                }
            });

            // Show product details
            this.showProductDetails(productId);
        } catch (error) {
            console.error('❌ Failed to select product:', error);
            this.showError('Failed to select product: ' + (error as Error).message);
        }
    }

    async showProductDetails(productId: number): Promise<void> {
        try {
            if (!this.sharedDbService) {
                throw new Error('Database service not initialized');
            }

            // Hide welcome screen and show details
            const welcomeScreen = document.getElementById('welcomeScreen');
            const itemDetails = document.getElementById('itemDetails');

            if (welcomeScreen) welcomeScreen.style.display = 'none';
            if (itemDetails) itemDetails.style.display = 'block';

            // Load and display product details
            const products = await this.sharedDbService.getProducts();
            const product = products.find((p: Product) => p.id === productId);

            if (product) {
                this.renderProductOverview(product);
            }
        } catch (error) {
            console.error('❌ Failed to show product details:', error);
            this.showError('Failed to load product details: ' + (error as Error).message);
        }
    }

    renderProductOverview(product: Product): void {
        const overviewContent = document.getElementById('overviewContent');
        if (!overviewContent) return;

        overviewContent.innerHTML = `
            <div class="form-group">
                <label>Product Name</label>
                <div class="readonly-field">${this.escapeHtml(product.name || 'N/A')}</div>
            </div>
            <div class="form-row">
                <div class="form-group">
                    <label>Product ID</label>
                    <div class="readonly-field">${product.id}</div>
                </div>
                <div class="form-group">
                    <label>Status</label>
                    <div class="readonly-field">
                        <span class="status-badge status-${product.status || 'inactive'}">${product.status || 'inactive'}</span>
                    </div>
                </div>
            </div>
            <div class="form-group">
                <label>Description</label>
                <div class="readonly-field">${this.escapeHtml(product.description || 'No description available')}</div>
            </div>
            <div class="form-group">
                <label>Category</label>
                <div class="readonly-field">${this.escapeHtml(product.category || 'Uncategorized')}</div>
            </div>
            <div class="form-row">
                <div class="form-group">
                    <label>Created</label>
                    <div class="readonly-field">${product.created_at ? new Date(product.created_at).toLocaleDateString() : 'N/A'}</div>
                </div>
                <div class="form-group">
                    <label>Updated</label>
                    <div class="readonly-field">${product.updated_at ? new Date(product.updated_at).toLocaleDateString() : 'N/A'}</div>
                </div>
            </div>
        `;
    }

    showError(message: string): void {
        const itemsList = document.getElementById('itemsList');
        if (itemsList) {
            itemsList.innerHTML = `
                <div class="error">
                    <strong>Error:</strong> ${this.escapeHtml(message)}
                </div>
            `;
        }
    }

    goBackToIDE(): void {
        window.location.href = 'home.html';
    }

    showCreateProductForm(): void {
        console.log('📝 Showing create product form...');
        const createForm = document.getElementById('createForm');
        const welcomeScreen = document.getElementById('welcomeScreen');

        if (createForm) createForm.style.display = 'block';
        if (welcomeScreen) welcomeScreen.style.display = 'none';

        // TODO: Implement create form
    }

    switchTab(tabName: string): void {
        // Hide all tabs
        document.querySelectorAll('.tab').forEach(tab => tab.classList.remove('active'));
        document.querySelectorAll('.tab-content').forEach(content => content.classList.remove('active'));

        // Show selected tab
        const tabIndex = tabName === 'overview' ? 1 : tabName === 'services' ? 2 : 3;
        const tabElement = document.querySelector(`.tab:nth-child(${tabIndex})`);
        const contentElement = document.getElementById(`${tabName}-tab`);

        if (tabElement) tabElement.classList.add('active');
        if (contentElement) contentElement.classList.add('active');
    }

    showAddServiceForm(): void {
        console.log('📝 Showing add service form...');
        // TODO: Implement add service form
    }

    cancelCreate(): void {
        const createForm = document.getElementById('createForm');
        const welcomeScreen = document.getElementById('welcomeScreen');

        if (createForm) createForm.style.display = 'none';
        if (welcomeScreen) welcomeScreen.style.display = 'block';
    }

    private escapeHtml(text: string): string {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Make it available globally for the HTML onclick handlers
declare global {
    interface Window {
        ProductServicesManager: typeof ProductServicesManager;
        productServicesManager?: ProductServicesManager;
    }
}

window.ProductServicesManager = ProductServicesManager;