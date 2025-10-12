// Auto-generated TypeScript types from Rust structs
// This prevents field name mismatches between frontend and backend

export interface Product {
  id: number;
  product_id: string;
  product_name: string;
  line_of_business: string;
  description?: string;
  status: string;
  pricing_model?: string;
  target_market?: string;
  created_by?: string;
  created_at: string;
  updated_by?: string;
  updated_at: string;
}

export interface Service {
  id: number;
  service_id: string;
  service_name: string;
  service_category?: string;
  description?: string;
  is_core_service: boolean;
  status: string;
  created_by?: string;
  created_at: string;
  updated_by?: string;
  updated_at: string;
}

export interface Resource {
  id: number;
  resource_id: string;
  resource_name: string;
  resource_type: string;
  description?: string;
  location?: string;
  status: string;
  created_by?: string;
  created_at: string;
  updated_by?: string;
  updated_at: string;
}

export interface CreateProductRequest {
  product_name: string;
  line_of_business: string;
  description?: string;
  pricing_model?: string;
  target_market?: string;
  created_by?: string;
}

export interface CreateServiceRequest {
  service_name: string;
  service_category?: string;
  description?: string;
  is_core_service?: boolean;
  created_by?: string;
}

export interface CreateResourceRequest {
  resource_name: string;
  resource_type: string;
  description?: string;
  location?: string;
  created_by?: string;
}
