# Analytics - Business Context

**Version**: 1.0  
**Date**: 2025-12-31  
**Module**: Analytics

**Technical Design**: `@/modules/analytics/architecture/DESIGN.md`

---

## Section A: VISION

**Purpose**: Comprehensive framework for creating, managing, and displaying data visualizations and reports within the Hyperspot Platform

**Target Users**:
- Platform Administrators - Infrastructure and security management
- Data Engineers - External data infrastructure (indirect interaction)
- Plugin Developers - Custom datasource and query plugins
- Dashboard Designers - Creating dashboards and visualizations
- Business Analysts - Consuming reports and insights
- End Users - Viewing dashboards and exploring data

**Key Problems Solved**:
- **Data Fragmentation**: Unified access to multiple external data sources through plugin architecture
- **Data Agnostic Design**: No vendor lock-in - works with any data source (OLAP, OLTP, APIs) via query registration
- **Visualization Complexity**: Rich set of chart types and interactive features without coding
- **Type Safety**: Strong typing through GTS (Global Type System) prevents runtime errors
- **Multi-Tenancy**: Complete tenant isolation with automatic JWT propagation
- **Extensibility**: Plugin-based architecture for custom datasources and query adapters
- **Reporting & Scheduling**: Automated report generation and delivery via platform services
- **Performance**: Query result caching and horizontal scalability (data performance depends on external sources)
- **Security**: SecurityCtx enforced at every layer with automatic tenant context injection

**Success Criteria**:
- Sub-second query response for typical dashboards (p95 < 1s)
- Support 100+ concurrent users per tenant
- 99.9% uptime SLA
- Plugin registration without service restart
- Complete tenant data isolation

---

## Section B: Actors

**Human Actors**:

#### Platform Administrator
**ID**: `fdd-analytics-actor-platform-admin`  
**Role**: Manages platform infrastructure and configuration

#### Data Engineer
**ID**: `fdd-analytics-actor-data-engineer`  
**Role**: Manages external data infrastructure (indirect interaction with Analytics - manages external DWH/ETL systems that Analytics queries via plugins)

#### Plugin Developer
**ID**: `fdd-analytics-actor-plugin-developer`  
**Role**: Develops custom datasource plugins and adapters

#### Dashboard Designer
**ID**: `fdd-analytics-actor-dashboard-designer`  
**Role**: Creates dashboards and visualizations

#### Business Analyst
**ID**: `fdd-analytics-actor-business-analyst`  
**Role**: Analyzes data and creates insights

#### End User
**ID**: `fdd-analytics-actor-end-user`  
**Role**: Consumes dashboards and reports

#### Template Developer
**ID**: `fdd-analytics-actor-template-developer`  
**Role**: Develops custom widget templates and visualizations

#### System Integrator
**ID**: `fdd-analytics-actor-system-integrator`  
**Role**: Embeds analytics into third-party products

#### Tenant Administrator
**ID**: `fdd-analytics-actor-tenant-admin`  
**Role**: Manages tenant-specific configurations

#### API Consumer
**ID**: `fdd-analytics-actor-api-consumer`  
**Role**: Integrates analytics programmatically

**System Actors**:

#### UI Application (HAI3)
**ID**: `fdd-analytics-actor-ui-app`  
**Role**: Frontend application for Analytics module (REST API consumption with JWT authentication)

#### Hyperspot Platform
**ID**: `fdd-analytics-actor-platform`  
**Role**: Provides core infrastructure services (event management, tenancy, authentication, scheduling, email delivery)

#### Query Plugin
**ID**: `fdd-analytics-actor-query-plugin`  
**Role**: Executes queries against datasources (Plugin API with JWT propagation)

#### External API Provider
**ID**: `fdd-analytics-actor-external-api`  
**Role**: Provides data through REST/OData APIs (must validate JWT and filter by tenant_id)

---

## Section C: Capabilities

#### Data Visualization
**ID**: `fdd-analytics-capability-data-visualization`
- Rich chart types (line, bar, pie, scatter, heatmap, etc.)
- Interactive tables with sorting and filtering
- Geographic maps with custom layers
- Custom widget templates
- Values selectors (dropdowns, autocomplete, pickers) for filters and parameters

**Actors**: `fdd-analytics-actor-dashboard-designer`, `fdd-analytics-actor-business-analyst`, `fdd-analytics-actor-end-user`, `fdd-analytics-actor-template-developer`, `fdd-analytics-actor-ui-app`

#### Data Access
**ID**: `fdd-analytics-capability-data-access`
- Plugin-based datasource architecture
- OData v4 query support
- Native REST API queries
- Real-time data refresh
- Data agnostic - no built-in DWH or data sources, all connected via query registration

**Actors**: `fdd-analytics-actor-plugin-developer`, `fdd-analytics-actor-query-plugin`, `fdd-analytics-actor-external-api`, `fdd-analytics-actor-ui-app`

#### Datasource Management
**ID**: `fdd-analytics-capability-datasource-mgmt`
- Datasource configuration (query + parameters + UI controls)
- Parameter binding and validation
- Values selector integration for parameter inputs
- Datasource reusability across widgets
- Runtime parameter injection

**Actors**: `fdd-analytics-actor-plugin-developer`, `fdd-analytics-actor-dashboard-designer`, `fdd-analytics-actor-tenant-admin`

#### Dashboard Management
**ID**: `fdd-analytics-capability-dashboard-mgmt`
- Grid-based responsive layouts
- Drag-and-drop widget positioning
- Dashboard templates
- Version history

**Actors**: `fdd-analytics-actor-dashboard-designer`, `fdd-analytics-actor-business-analyst`, `fdd-analytics-actor-ui-app`

#### Query Execution
**ID**: `fdd-analytics-capability-query-execution`
- Multi-datasource queries
- Query result caching
- Automatic JWT generation with tenant context
- Plugin-based query adapters

**Actors**: `fdd-analytics-actor-business-analyst`, `fdd-analytics-actor-api-consumer`, `fdd-analytics-actor-query-plugin`, `fdd-analytics-actor-ui-app`

#### Reporting
**ID**: `fdd-analytics-capability-reporting`
- Report generation (on-demand, scheduled via platform)
- Report templates (based on dashboards)
- Multi-format export (PDF, CSV, Excel)
- Report history and versioning
- Report delivery (email via platform)

**Actors**: `fdd-analytics-actor-business-analyst`, `fdd-analytics-actor-dashboard-designer`, `fdd-analytics-actor-platform`

#### Export & Sharing
**ID**: `fdd-analytics-capability-export-sharing`
- Dashboard export to multiple formats
- Dashboard sharing with permissions
- Embed widgets in external apps
- Public/private dashboard URLs

**Actors**: `fdd-analytics-actor-dashboard-designer`, `fdd-analytics-actor-business-analyst`, `fdd-analytics-actor-end-user`, `fdd-analytics-actor-system-integrator`

#### Security & Multi-Tenancy
**ID**: `fdd-analytics-capability-security`
- Complete tenant isolation
- SecurityCtx enforced everywhere
- JWT-based API authentication
- Row-level security in queries

**Actors**: `fdd-analytics-actor-platform-admin`, `fdd-analytics-actor-tenant-admin`, `fdd-analytics-actor-platform`

#### Extensible Architecture
**ID**: `fdd-analytics-capability-extensibility`
- Dynamic datasource registration
- Custom query implementations
- Contract format adapters (native, odata, rest)
- GTS-based type extensions
- Plugin-based extensibility

**Actors**: `fdd-analytics-actor-plugin-developer`, `fdd-analytics-actor-template-developer`

#### Organization & Libraries
**ID**: `fdd-analytics-capability-organization`
- Categories for all GTS types and instances (hierarchical classification)
- Widget Libraries - reusable widget collections
- Template Libraries - visualization template marketplace
- Datasource Libraries - preconfigured data source connectors
- Query Libraries - shareable query definitions

**Actors**: `fdd-analytics-actor-dashboard-designer`, `fdd-analytics-actor-template-developer`, `fdd-analytics-actor-plugin-developer`

#### Performance
**ID**: `fdd-analytics-capability-performance`
- Query result caching
- Horizontal scalability
- External data sources accessed via query plugins
- No built-in ETL or DWH (data agnostic)

**Actors**: `fdd-analytics-actor-platform-admin`, `fdd-analytics-actor-query-plugin`

---

## Section E: Additional Context

**Note**: This section is optional and reserved for product owner notes, business rationale, or other relevant context not covered by the core FDD structure.

<!-- Add any additional business context, market positioning, stakeholder feedback, or other relevant information here -->
