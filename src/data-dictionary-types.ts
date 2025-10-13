// Data Dictionary v2.0 - Multi-layered Schema with Configuration-Driven UI
// Based on Gemini design session - supports UI rendering, AI RAG, and business logic

// ===== CORE DATA TYPES =====

export type DataType = 'String' | 'Number' | 'Decimal' | 'Integer' | 'Date' | 'DateTime' | 'Boolean' | 'List' | 'Object';
export type LayoutType = 'tabs' | 'wizard' | 'vertical-stack' | 'horizontal-grid' | 'accordion';
export type RenderHint = 'text-input' | 'textarea' | 'select' | 'multiselect' | 'checkbox' | 'radio' | 'date-picker' | 'number-input' | 'email-input' | 'phone-input' | 'rich-text' | 'file-upload' | 'color-picker';

// ===== CONSTRAINT SYSTEM =====

export interface AttributeConstraints {
    required?: boolean;
    minLength?: number;
    maxLength?: number;
    min?: number;
    max?: number;
    pattern?: string;
    allowedValues?: string[];
    customValidation?: string;
}

// ===== PERSISTENCE LAYER =====

export interface PersistenceLocator {
    system: string;           // e.g., "EntityMasterDB", "TradingSystem"
    entity: string;           // e.g., "related_parties", "transactions"
    identifier: string;       // e.g., "full_name", "trade_amount"
    schema?: string;          // Optional database schema
    version?: string;         // Optional version tracking
}

// ===== AI INTEGRATION =====

export interface GenerationExample {
    prompt: string;           // User prompt for AI
    response: string;         // Expected AI response/rule generation
    context?: string;         // Optional context description
    tags?: string[];          // Categorization tags
}

// ===== UI CONFIGURATION SYSTEM =====

export interface WizardNavigation {
    step: number;
    title: string;
    nextButton?: string;      // Custom button text
    previousButton?: string;  // Custom button text
    description?: string;     // Step description
    validation?: string[];    // Required fields for step completion
}

export interface UIConfiguration {
    group: string;                    // UI grouping/section
    displayOrder: number;             // Order within group
    renderHint: RenderHint;           // UI component type
    label: string;                    // Display label
    placeholder?: string;             // Input placeholder
    helpText?: string;                // Help/description text
    icon?: string;                    // UI icon identifier
    width?: string;                   // CSS width (e.g., "100%", "200px")
    conditional?: {                   // Conditional display
        dependsOn: string;            // Attribute name
        values: string[];             // Show when dependent has these values
    };
    wizard?: WizardNavigation;        // Wizard-specific configuration
    validation?: {
        message?: string;             // Custom validation message
        realTime?: boolean;           // Real-time validation
    };
}

// ===== PERSPECTIVE SYSTEM =====

export interface AttributePerspective {
    description?: string;             // Context-specific description
    ui?: Partial<UIConfiguration>;   // UI overrides for this perspective
    generationExamples?: GenerationExample[];  // Context-specific AI examples
    constraints?: Partial<AttributeConstraints>;  // Additional constraints
    tags?: string[];                  // Perspective-specific tags
}

// ===== CORE ATTRIBUTE OBJECT =====

export interface AttributeObject {
    name: string;                     // Unique machine-readable ID
    dataType: DataType;               // Strict data type
    description: string;              // Default business definition
    constraints?: AttributeConstraints;  // Validation rules
    allowedValues?: string[];         // Valid enumerated values
    persistence_locator: PersistenceLocator;  // Storage location
    rules_dsl?: string;               // Optional DSL for derived attributes
    generationExamples?: GenerationExample[];  // Default AI examples
    ui: UIConfiguration;              // Default UI rendering configuration
    perspectives?: Record<string, AttributePerspective>;  // Context-specific overrides

    // Metadata
    version?: string;
    created_date?: string;
    last_modified?: string;
    created_by?: string;
    tags?: string[];
}

// ===== RESOURCE LEVEL CONFIGURATION =====

export interface ResourceUIConfiguration {
    layout: LayoutType;               // Overall layout strategy
    groupOrder: string[];             // Precise order of UI groups
    theme?: string;                   // UI theme/styling
    responsive?: boolean;             // Responsive design enabled
    customCSS?: string;               // Custom CSS classes
    navigation?: {
        showProgress?: boolean;       // Show progress indicator
        allowSkip?: boolean;          // Allow skipping steps
        confirmExit?: boolean;        // Confirm before exit
    };
}

// ===== RESOURCE OBJECT =====

export interface ResourceObject {
    resourceName: string;             // Unique identifier
    description: string;              // Business purpose
    version?: string;                 // Schema version
    ui: ResourceUIConfiguration;      // Macro-level UI configuration
    attributes: AttributeObject[];    // Array of attributes

    // Resource metadata
    category?: string;                // Resource category
    owner?: string;                   // Business owner
    created_date?: string;
    last_modified?: string;
    status?: 'active' | 'deprecated' | 'draft';
}

// ===== ROOT DICTIONARY TYPE =====

export type ResourceDictionary = ResourceObject[];

// ===== RUNTIME CONTEXT TYPES =====

export interface PerspectiveContext {
    activePerspective: string;        // Currently active perspective
    availablePerspectives: string[];  // All available perspectives
    resourceName: string;             // Current resource
}

export interface UIRenderContext {
    perspective: PerspectiveContext;
    formData: Record<string, any>;    // Current form values
    validationErrors: Record<string, string[]>;  // Validation state
    currentStep?: number;             // For wizard layouts
    groupVisibility: Record<string, boolean>;  // Group visibility state
}

// ===== CONFIGURATION-DRIVEN RENDERER TYPES =====

export interface RendererConfig {
    enableValidation: boolean;
    enableConditionalLogic: boolean;
    enableRealTimeValidation: boolean;
    customComponents?: Record<string, any>;  // Custom component registry
    eventHandlers?: {
        onFieldChange?: (fieldName: string, value: any, context: UIRenderContext) => void;
        onGroupChange?: (groupName: string, visible: boolean) => void;
        onPerspectiveChange?: (newPerspective: string) => void;
        onStepChange?: (step: number) => void;
    };
}

// ===== AI-INTEGRATION TYPES =====

export interface AIGenerationRequest {
    prompt: string;
    context: {
        resourceName: string;
        perspective: string;
        availableAttributes: string[];
        formData?: Record<string, any>;
    };
}

export interface AIGenerationResponse {
    generatedRule: string;
    confidence: number;
    usedExamples: string[];  // Which examples influenced the generation
    suggestions: string[];   // Alternative suggestions
}

// ===== UTILITY TYPES =====

export interface ValidationResult {
    isValid: boolean;
    errors: Record<string, string[]>;
    warnings: Record<string, string[]>;
}

export interface AttributeResolution {
    attribute: AttributeObject;
    resolvedUI: UIConfiguration;      // UI with perspective overrides applied
    resolvedDescription: string;      // Description with perspective override
    resolvedExamples: GenerationExample[];  // Examples with perspective context
}

// ===== EXPORT CONVENIENCE FUNCTIONS =====

export type { AttributeObject as Attribute };
export type { ResourceObject as Resource };
export type { ResourceDictionary as Dictionary };