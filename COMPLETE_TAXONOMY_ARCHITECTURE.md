# Complete Financial Services Taxonomy Architecture

## Overview
A comprehensive multi-level taxonomy system for financial services that maps the complete chain from products to resources, including investment mandates and CBU member roles.

## Architecture Hierarchy

```
🏢 CBU (Client Business Unit)
├── 👥 CBU Members (by Role)
│   ├── 💰 Asset Owner/SPV → gives investment mandate
│   ├── 📊 Investment Manager → receives investment mandate & executes
│   ├── 🏛️ Custodian → safekeeping & settlement
│   ├── 📋 Administrator → administration & reporting
│   ├── ⚖️ Compliance Officer → compliance & monitoring
│   └── 🔧 Other roles (Processor, Technology Provider, etc.)
│
├── 📦 Products (Public/Generic Commercial Products)
│   ├── 🎛️ Product Options (Market Settlement, Currency Support, etc.)
│   │   └── 🔧 Services (Generic Financial Services)
│   │       └── 💻 Resources (Proprietary Applications/Systems)
│   │           └── 📊 Attributes (Enhanced with AI/UI metadata)
│   │
│   └── 🎯 Investment Mandates (Given by Asset Owner to Investment Manager)
│       ├── 🎪 Instruments (Industry Standard Taxonomy)
│       ├── 📐 Volumes & Limits
│       └── 📄 Instruction Formats
```

## Database Implementation

### Core Tables

#### 1. CBU & Members
- `client_business_units` - Business units
- `cbu_members` - Members with roles and authorities
- `cbu_roles` - Role definitions (Asset Owner, Investment Manager, etc.)

#### 2. Products & Options
- `products` - Commercial products in contracts
- `product_options` - Market settlement choices, currency support
- `product_option_service_mappings` - Options → Services

#### 3. Services & Resources
- `services` - Generic financial services (custody, reconciliation, etc.)
- `service_resource_mappings` - Services → Resources
- `resource_objects` - Proprietary applications/systems
- `attribute_objects` - Enhanced attributes with AI/UI metadata

#### 4. Investment Mandates
- `investment_mandates` - Mandates with asset owner → investment manager flow
- `mandate_instruments` - Allowed instruments with volumes/limits
- `mandate_instruction_mappings` - Instruction format preferences
- `instruction_formats` - Standard message formats (FIX, SWIFT, ISO20022)

#### 5. Instrument Taxonomy
- `instrument_taxonomy` - Industry standard classifications
- `mandate_instrument_allocations` - Allocation constraints per mandate

## Key Relationships

### Investment Mandate Flow
```
Asset Owner/SPV → gives mandate to → Investment Manager
                                  ↓
                            executes mandate using
                                  ↓
                           Specified Instruments
                           within Volume Limits
                           using Instruction Formats
```

### Product Delivery Chain
```
Product → Options → Services → Resources → Attributes
  ↓         ↓         ↓         ↓           ↓
Custody → Markets → Settlement → Apps → KYC Fields
```

## Example Implementation

### CBU-203914: Global Trade Finance Consortium
- **Asset Owner**: Singapore Sovereign Wealth Fund
- **Investment Manager**: Asian Trade Capital Management
- **Mandate**: Conservative trade finance strategy
- **Instruments**: Government bonds (50%), Corporate bonds (30%), Money market (20%)
- **Product**: Trade Settlement Professional
- **Services**: Enhanced custody, reconciliation, settlement

## Views & Functions

### Key Views
- `cbu_investment_mandate_structure` - Complete CBU → Mandate → Instruments
- `cbu_member_investment_roles` - Role-based responsibilities
- `enhanced_commercial_taxonomy_view` - Products → Options → Services → Resources
- `complete_investment_taxonomy_view` - Full hierarchy with mandates

### Key Functions
- `get_enhanced_product_taxonomy_hierarchy()` - 4-level hierarchy retrieval
- `validate_commercial_taxonomy()` - Integrity validation
- `create_attribute_set_snapshot()` - Version control

## Business Context

### Financial Services Types
- **Products**: Public/generic (Institutional Custody Plus, Fund Administration Complete)
- **Services**: Public/generic (custody, safekeeping, reconciliation, fund accounting, middle office, trade order management)
- **Resources**: Proprietary/private implementations (application accounts, routing tables, reconciliation app, FA app, IBOR app)

### Investment Management
- **Mandates**: Define what can be invested, volumes, instruction formats
- **Instruments**: Industry standard taxonomy (equities, fixed income, derivatives, alternatives)
- **Formats**: Standard messaging (FIX, SWIFT, ISO20022, JSON, CSV)

## Technical Features

### AI & UI Enhancement
- Vector embeddings for semantic similarity
- Comprehensive metadata for LLM understanding
- Dynamic form generation based on context
- Enhanced attribute descriptions and business context

### Multi-User Architecture
- Connection pooling for concurrent access
- Role-based access control
- Real-time state management
- Conflict-free collaborative editing

### Data Quality
- Validation rules and integrity checks
- Version control and audit trails
- Comprehensive indexing for performance
- Cascading updates and deletions

## Usage Patterns

### CBU Expansion
- Click CBU → View members and their roles
- See investment mandates by investment manager
- Track mandate compliance and limits

### Product Configuration
- Select product → Choose options → Configure services
- Map services to proprietary resources
- Define attribute sets for data collection

### Investment Management
- Asset owner creates mandate
- Investment manager receives constraints
- System enforces limits and formats
- Real-time monitoring and reporting

This architecture provides a complete, role-based, multi-level taxonomy that scales from high-level commercial products down to individual data attributes, while maintaining proper business relationships and technical implementation details.