// TypeScript CRUD functions with proper type safety
import { Product, Service, Resource, CreateProductRequest, CreateServiceRequest, UpdateServiceRequest, CreateResourceRequest } from './types/generated.js';

// Tauri API interface
declare global {
    interface Window {
        __TAURI_INVOKE__: (command: string, args?: any) => Promise<any>;
    }
}

// Product CRUD functions
export async function loadProductsData(): Promise<void> {
    const content = document.querySelector('#products-panel .side-panel-content');
    if (!content) return;

    try {
        if (window.__TAURI_INVOKE__) {
            const products: Product[] = await window.__TAURI_INVOKE__('list_products');
            let html = '<ul class="tree-view">';

            if (products && products.length > 0) {
                products.forEach((product: Product) => {
                    html += `
                        <li class="tree-item" onclick="openProduct('${product.id}')">
                            <span class="tree-icon">📦</span>
                            ${product.product_name}
                            <span class="tree-actions">
                                <span onclick="event.stopPropagation(); editProduct('${product.id}')" title="Edit">✏️</span>
                                <span onclick="event.stopPropagation(); deleteProduct('${product.id}')" title="Delete">🗑️</span>
                            </span>
                        </li>
                    `;
                });
            } else {
                html += `
                    <li class="tree-item">
                        <span class="tree-icon">📦</span>
                        No products found
                    </li>
                `;
            }

            html += '</ul>';
            content.innerHTML = html;
        }
    } catch (error) {
        console.error('Error loading products:', error);
        content.innerHTML = `
            <div class="error-message">
                <span class="tree-icon">❌</span>
                Error loading products
            </div>
        `;
    }
}

export async function createProduct(): Promise<void> {
    const form = document.getElementById('product-form') as HTMLFormElement;
    if (!form) return;

    const formData = new FormData(form);

    const request: CreateProductRequest = {
        product_name: formData.get('product_name') as string,
        line_of_business: formData.get('line_of_business') as string,
        description: formData.get('description') as string || undefined,
        pricing_model: formData.get('pricing_model') as string || undefined,
        target_market: formData.get('target_market') as string || undefined,
        created_by: formData.get('created_by') as string || undefined,
    };

    try {
        const result: Product = await window.__TAURI_INVOKE__('create_product', request);
        console.log('Product created:', result);

        // Close modal and refresh list
        closeModal();
        await loadProductsData();

        addToOutput('success', `✅ Product "${result.product_name}" created successfully`);
    } catch (error) {
        console.error('Error creating product:', error);
        addToOutput('error', `❌ Failed to create product: ${error}`);
    }
}

// Service CRUD functions
export async function loadServicesData(): Promise<void> {
    const content = document.querySelector('#services-panel .side-panel-content');
    if (!content) return;

    try {
        if (window.__TAURI_INVOKE__) {
            const services: Service[] = await window.__TAURI_INVOKE__('list_services');
            let html = '<ul class="tree-view">';

            if (services && services.length > 0) {
                services.forEach((service: Service) => {
                    html += `
                        <li class="tree-item" onclick="openService('${service.id}')">
                            <span class="tree-icon">🔧</span>
                            ${service.service_name}
                            <span class="tree-actions">
                                <span onclick="event.stopPropagation(); editService('${service.id}')" title="Edit">✏️</span>
                                <span onclick="event.stopPropagation(); deleteService('${service.id}')" title="Delete">🗑️</span>
                            </span>
                        </li>
                    `;
                });
            } else {
                html += `
                    <li class="tree-item">
                        <span class="tree-icon">🔧</span>
                        No services found
                    </li>
                `;
            }

            html += '</ul>';
            content.innerHTML = html;
        }
    } catch (error) {
        console.error('Error loading services:', error);
        content.innerHTML = `
            <div class="error-message">
                <span class="tree-icon">❌</span>
                Error loading services
            </div>
        `;
    }
}

export async function createService(): Promise<void> {
    const form = document.getElementById('service-form') as HTMLFormElement;
    if (!form) return;

    const formData = new FormData(form);

    const request: CreateServiceRequest = {
        service_name: formData.get('service_name') as string,
        service_category: formData.get('service_category') as string || undefined,
        description: formData.get('description') as string || undefined,
        is_core_service: formData.get('is_core_service') === 'true' || undefined,
        created_by: formData.get('created_by') as string || undefined,
    };

    try {
        const result: Service = await window.__TAURI_INVOKE__('create_service', request);
        console.log('Service created:', result);

        // Close modal and refresh list
        closeModal();
        await loadServicesData();

        addToOutput('success', `✅ Service "${result.service_name}" created successfully`);
    } catch (error) {
        console.error('Error creating service:', error);
        addToOutput('error', `❌ Failed to create service: ${error}`);
    }
}

export async function updateService(serviceId: number): Promise<void> {
    const form = document.getElementById('service-form') as HTMLFormElement;
    if (!form) return;

    const formData = new FormData(form);

    const request: UpdateServiceRequest = {
        service_name: formData.get('service_name') as string || undefined,
        service_category: formData.get('service_category') as string || undefined,
        description: formData.get('description') as string || undefined,
        is_core_service: formData.get('is_core_service') === 'true' || undefined,
        updated_by: formData.get('updated_by') as string || undefined,
    };

    // Remove undefined fields to only update what's provided
    Object.keys(request).forEach(key => {
        if (request[key as keyof UpdateServiceRequest] === undefined) {
            delete request[key as keyof UpdateServiceRequest];
        }
    });

    try {
        const result: Service = await window.__TAURI_INVOKE__('update_service', { service_id: serviceId, request });
        console.log('Service updated:', result);

        // Close modal and refresh list
        closeModal();
        await loadServicesData();

        addToOutput('success', `✅ Service "${result.service_name}" updated successfully`);
    } catch (error) {
        console.error('Error updating service:', error);
        addToOutput('error', `❌ Failed to update service: ${error}`);
    }
}

// Resource CRUD functions
export async function loadResourcesData(): Promise<void> {
    const content = document.querySelector('#resources-panel .side-panel-content');
    if (!content) return;

    try {
        if (window.__TAURI_INVOKE__) {
            const resources: Resource[] = await window.__TAURI_INVOKE__('list_resources');
            let html = '<ul class="tree-view">';

            if (resources && resources.length > 0) {
                resources.forEach((resource: Resource) => {
                    html += `
                        <li class="tree-item" onclick="openResource('${resource.id}')">
                            <span class="tree-icon">🛠️</span>
                            ${resource.resource_name}
                            <span class="tree-actions">
                                <span onclick="event.stopPropagation(); editResource('${resource.id}')" title="Edit">✏️</span>
                                <span onclick="event.stopPropagation(); deleteResource('${resource.id}')" title="Delete">🗑️</span>
                            </span>
                        </li>
                    `;
                });
            } else {
                html += `
                    <li class="tree-item">
                        <span class="tree-icon">🛠️</span>
                        No resources found
                    </li>
                `;
            }

            html += '</ul>';
            content.innerHTML = html;
        }
    } catch (error) {
        console.error('Error loading resources:', error);
        content.innerHTML = `
            <div class="error-message">
                <span class="tree-icon">❌</span>
                Error loading resources
            </div>
        `;
    }
}

export async function createResource(): Promise<void> {
    const form = document.getElementById('resource-form') as HTMLFormElement;
    if (!form) return;

    const formData = new FormData(form);

    const request: CreateResourceRequest = {
        resource_name: formData.get('resource_name') as string,
        resource_type: formData.get('resource_type') as string,
        description: formData.get('description') as string || undefined,
        location: formData.get('location') as string || undefined,
        created_by: formData.get('created_by') as string || undefined,
    };

    try {
        const result: Resource = await window.__TAURI_INVOKE__('create_resource', request);
        console.log('Resource created:', result);

        // Close modal and refresh list
        closeModal();
        await loadResourcesData();

        addToOutput('success', `✅ Resource "${result.resource_name}" created successfully`);
    } catch (error) {
        console.error('Error creating resource:', error);
        addToOutput('error', `❌ Failed to create resource: ${error}`);
    }
}

// Utility functions - these need to be defined elsewhere or imported
declare function closeModal(): void;
declare function addToOutput(type: string, message: string): void;