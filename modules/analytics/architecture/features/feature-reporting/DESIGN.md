# Feature: Reporting

**Status**: NOT_STARTED  
**Feature Slug**: `feature-reporting`

---

## A. Feature Context

### Overview

Report generation with scheduling and delivery via platform services. Handles report creation, scheduled generation, multi-format export, and delivery integration with Hyperspot Platform services.

**Purpose**: Provide automated report generation and delivery with scheduling integration.

**Scope**:
- Report generation (on-demand, scheduled)
- Report templates based on dashboards
- Multi-format export (PDF, CSV, Excel)
- Report history and versioning
- Schedule management via **Hyperspot Platform Scheduling Service**
- Report delivery via **Hyperspot Platform Email Service**
- Report access control
- Report parameters and filters
- Async generation for large reports

**Out of Scope**:
- Report layout storage - handled by feature-report-layouts
- Dashboard business logic - handled by feature-dashboards
- Email infrastructure - provided by Hyperspot Platform
- Scheduling infrastructure - provided by Hyperspot Platform

### GTS Types

This feature **uses** report layout types:

**Uses types from**:
- `gts://gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~*` - Report layouts
- `gts://gts.hypernetix.hyperspot.ax.subscription.v1~*` - Report subscriptions

References from `gts/types/`:
- Report layout schemas (owned by feature-report-layouts)

### OpenAPI Endpoints

From `architecture/openapi/v1/api.yaml`:
- `POST /api/analytics/v1/reports/{report-id}/generate` - Generate report on-demand
- `GET /api/analytics/v1/reports/{report-id}/generations/{generation-id}` - Get generation status
- `GET /api/analytics/v1/reports/{report-id}/generations` - List report history
- `POST /api/analytics/v1/reports/{report-id}/rollback` - Rollback to version

### Platform Service Dependencies

**Hyperspot Platform Services**:
- **Scheduling Service**: `POST /api/platform/v1/scheduling/jobs` - Cron-based scheduling
- **Email Service**: `POST /api/platform/v1/email/send` - Email delivery

### Actors

**Human Actors** (from Overall Design):
- **Report Creator** - Creates and configures reports
- **Report Subscriber** - Subscribes to scheduled reports
- **Admin** - Manages report schedules and permissions

**System Actors**:
- **Report Generator** - Orchestrates report generation
- **Export Engine** - Generates PDF/Excel/CSV files
- **Schedule Manager** - Integrates with Platform Scheduling Service
- **Delivery Manager** - Integrates with Platform Email Service

**Service Roles** (from OpenAPI):
- `analytics:reports:read` - View reports
- `analytics:reports:write` - Create/edit reports
- `analytics:reports:generate` - Generate reports
- `analytics:reports:schedule` - Manage schedules

---

## B. Actor Flows

### Flow 1: Report Creator Creates Scheduled Report

**Actor**: Report Creator  
**Trigger**: Need automated weekly sales report  
**Goal**: Create report with automatic generation and email delivery

**Steps**:
1. Navigate to Reports → Create New
2. Choose starting point (convert from dashboard or start blank)
3. Enter report metadata (name, description, category)
4. Configure report-specific settings:
   - Paper size (A4, Letter)
   - Orientation (portrait, landscape)
   - Header/footer templates
5. Add widgets to report (same as dashboard)
6. Set up schedule:
   - Frequency (weekly)
   - Time (Monday 9am)
   - Recipients (sales-team@acme.com)
   - Format (PDF)
7. Test report generation
8. Save report

**API Interaction**:
```
POST /api/analytics/v1/gts
Type: gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~
Instance: gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~acme.sales._.weekly.v1

Body: {
  "items": [...widgets...],
  "settings": {
    "paper_size": "A4",
    "orientation": "portrait",
    "schedule": {
      "enabled": true,
      "frequency": "weekly",
      "cron": "0 9 * * MON",
      "recipients": ["sales-team@acme.com"],
      "delivery_format": "PDF"
    }
  }
}

→ Analytics registers schedule with Platform:
POST /api/platform/v1/scheduling/jobs
Body: {
  "id": "analytics.report.acme.sales._.weekly.v1",
  "cron": "0 9 * * MON",
  "callback_url": "https://analytics/api/v1/reports/{id}/generate"
}
```

---

### Flow 2: Platform Scheduler Triggers Report Generation

**Actor**: Schedule Manager (System)  
**Trigger**: Cron expression matches current time  
**Goal**: Generate and deliver scheduled report

**Steps**:
1. Platform Scheduling Service calls callback URL
2. Analytics loads report configuration
3. Executes all widget queries
4. Renders widgets with templates
5. Composes report with header/footer
6. Exports to PDF
7. Stores file in storage
8. Calls Platform Email Service for delivery
9. Logs generation and delivery status

**Platform Integration**:
```
Platform → POST /api/analytics/v1/reports/{report-id}/generate

→ Analytics generates report
→ Analytics calls Platform Email Service:

POST /api/platform/v1/email/send
Body: {
  "to": ["sales-team@acme.com"],
  "subject": "Weekly Sales Report - {{date}}",
  "attachments": [{
    "filename": "weekly-sales.pdf",
    "url": "https://storage/reports/550e8400.pdf"
  }]
}
```

---

### Flow 3: Report Subscriber Subscribes to Report

**Actor**: Report Subscriber  
**Trigger**: Want to receive specific report  
**Goal**: Create personal subscription with custom schedule

**Steps**:
1. Browse available reports
2. Click "Subscribe" on report
3. Configure delivery preferences:
   - Frequency (daily, weekly, monthly)
   - Format (PDF, Excel)
   - Filters (custom date range, region)
4. Save subscription
5. Receive reports via email on schedule

**API Interaction**:
```
POST /api/analytics/v1/gts
Type: gts.hypernetix.hyperspot.ax.subscription.v1~
Instance: gts.hypernetix.hyperspot.ax.subscription.v1~tenant-123~user-456~weekly-sales

Body: {
  "user_id": "user-456",
  "report_id": "...",
  "schedule": {"frequency": "weekly"},
  "delivery": {"email": "user@example.com", "format": "PDF"},
  "filters": {"region": "EMEA"}
}
```

---

### Flow 4: User Generates Report On-Demand

**Actor**: Report Creator  
**Trigger**: Need immediate report  
**Goal**: Generate and download report now

**API Interaction**:
```
POST /api/analytics/v1/reports/{report-id}/generate
Body: {
  "format": "PDF",
  "filters": {"date_range": "last_7_days"}
}

→ Returns:
{
  "generation_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "completed",
  "download_url": "https://storage/reports/550e8400.pdf",
  "expires_at": "2024-01-08T18:00:00Z"
}
```

---

## C. Algorithms

### Service Algorithm 1: Report Generation Flow

**Purpose**: Generate report with all widgets and export to file

**Steps**:

1. Load report configuration from GTS registry
2. Validate all dependencies exist
3. **PARALLEL**: Execute all widget queries
4. Wait for all queries to complete
5. Render widgets with templates
6. Compose final report layout
7. Export to requested format (PDF/Excel/CSV)
8. Store file in storage
9. **RETURN** generation result with file URL
    // 5. Compose report with layout
    let composed_report = compose_report(&report, &rendered_widgets)?;
    
    // 6. Export to requested format
    let file = export_report(&composed_report, format)?;
    
    // 7. Store file and return URL
    let url = storage.upload(file)?;
    
    Ok(GenerationResult {
        generation_id: Uuid::new_v4(),
        status: Status::Completed,
        download_url: url,
        expires_at: now() + Duration::hours(24)
    })
}
```

---

### Service Algorithm 2: Schedule Registration with Platform

**Purpose**: Register report schedule with Hyperspot Platform Scheduling Service

**Steps**:

1. **IF** schedule not enabled:
   1. **RETURN** (skip registration)
2. Build scheduling job:
   - ID: `analytics.report.{report_id}`
   - Cron expression from report config
   - Timezone from report config
   - Callback URL: `/api/v1/reports/{id}/generate`
3. Call Platform Scheduling Service API
4. Store schedule_id in report metadata
5. **RETURN** success
            "report_id": report.id,
            "recipients": report.settings.schedule.recipients,
            "format": report.settings.schedule.delivery_format
        })
    };
    
    // Register with platform
    platform_client.scheduling.create_job(&job, ctx)?;
    
    Ok(())
}
```

---

## D. States

### Report Generation States

```
[Not Started] → (Trigger) → [Queued]
[Queued] → (Process) → [Processing]
[Processing] → (Success) → [Completed]
[Processing] → (Failure) → [Failed]
[Processing] → (Cancel) → [Cancelled]
```

**State Descriptions**:
- **Queued**: Generation job queued for processing
- **Processing**: Report being generated
- **Completed**: Report ready for download
- **Failed**: Generation failed with error
- **Cancelled**: User cancelled generation

---

## E. Technical Details

### User Scenarios

*(17 detailed user scenarios preserved in technical details)*

### Scenario 1: Create Report

Similar to creating a dashboard, with report-specific differences:

**UI Flow:**
1. Navigate to Reports → Create New
2. Choose starting point:
   - **Convert from dashboard** (import dashboard layout and widgets)
   - **Start blank** (create from scratch)
3. Enter report metadata (name, description, icon, category)
4. Configure report-specific settings:
   - Paper size (A4, Letter, etc.)
   - Orientation (portrait, landscape)
   - Header/footer templates (title, date, page numbers, company logo)
5. Add widgets to report (same as dashboard)
   - Widget size: width percentage (15-100%), height preset (micro/small/medium/high/unlimited)
6. Configure print/export settings:
   - Page breaks
   - Color mode (color, grayscale)
7. Set up schedule (optional):
   - Frequency (daily, weekly, monthly, custom cron)
   - Time of day / timezone
   - Recipients (email addresses, distribution lists)
   - Delivery format (PDF, Excel, CSV)
8. Configure sharing (optional):
   - Choose visibility: private, specific tenants, or all tenants
   - Select tenants if sharing with specific tenants
9. Test report generation
10. Save report

**API Calls:**
```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~')
  # Browse report categories
GET /api/analytics/v1/gts/{dashboard_id}  # If converting from dashboard
POST /api/analytics/v1/gts  # Create report instance
  # Type: gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~
  # Instance ID: gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~acme.sales._.weekly_report.v1
  # Contains: items (widgets/groups), settings (paper_size, orientation, header/footer, schedule, delivery)
PUT /api/analytics/v1/gts/{report_id}/enablement  # Share report with tenants (optional)
  # Body: { "enabled_for": ["tenant-1", "tenant-2"] } or { "enabled_for": "all" }
POST /api/analytics/v1/reports/{report_id}/generate  # Test report generation
  # Returns generated report file or download URL
```

Note: Adding/editing widgets and groups in reports uses the same API calls as dashboards.

---

### Scenario 2: Edit Report

Similar to dashboard editing scenarios, with report-specific additions:

**UI Flow:**
1. Open report (user has edit permissions)
2. Edit report content:
   - Add/remove/move widgets
   - Edit widget settings
   - Create/edit/delete groups
3. Edit report-specific settings:
   - Paper size, orientation
   - Header/footer templates
   - Page breaks
   - Print settings
4. Edit schedule configuration:
   - Enable/disable schedule
   - Change frequency, time, timezone
   - Update recipients list
   - Change delivery format
5. Edit report metadata (name, description, category)
6. Test report generation with new settings
7. Save changes

**API Calls:**
```
GET /api/analytics/v1/gts/{report_id}  # Load report
PATCH /api/analytics/v1/gts/{report_id}  # Update report settings
  # JSON Patch operations on:
  #   - report/entity/items (widgets/groups) - same as dashboard
  #   - report/settings (paper_size, orientation, header, footer, schedule, delivery)
POST /api/analytics/v1/reports/{report_id}/generate  # Test report generation
```

Note: Widget/group management uses same API patterns as dashboard scenarios.

---

### Scenario 3: Delete Report

**UI Flow:**
1. Navigate to Reports list
2. Select report to delete
3. Click delete button
4. Confirm deletion - warn if report has active schedule
5. Report soft-deleted (sets deleted_at timestamp)
6. Associated schedule automatically disabled

**API Calls:**
```
DELETE /api/analytics/v1/gts/{report_id}
  # Soft-delete report (sets deleted_at timestamp)
  # Disables schedule if active
  # Returns: 204 No Content
```

---

### Scenario 4: Subscribe to Scheduled Reports

**UI Flow:**
1. User browses available reports
2. Clicks "Subscribe" on report
3. Configures delivery preferences:
   - Frequency (daily, weekly, monthly)
   - Preferred format (PDF, Excel)
   - Delivery method (email)
4. Saves subscription
5. User receives reports via email on schedule

**API Calls:**
```
GET /api/analytics/v1/gts?$filter=...
POST /api/analytics/v1/gts  # Create subscription instance
  # Type: gts.hypernetix.hyperspot.ax.subscription.v1~
  # Instance ID: gts.hypernetix.hyperspot.ax.subscription.v1~tenant-123~user-456~weekly-sales
  # Contains: user_id, report_id, schedule, delivery, filters
PATCH /api/analytics/v1/gts/{subscription_id}  # Update entity/schedule or entity/delivery
DELETE /api/analytics/v1/gts/{subscription_id}  # Unsubscribe
```

---

## Report Structure

```json
{
  "id": "gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~acme.sales._.weekly.v1",
  "type": "gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~",
  "entity": {
    "name": "Weekly Sales Report",
    "description": "Comprehensive weekly sales analysis",
    "icon": "report",
    "category_id": "gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.report.v1~sales",
    "items": [
      {
        "type": "gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~",
        "settings": {
          "name": "Sales Summary",
          "template": {
            "id": "gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~table.v1",
            "config": { /* template config */ }
          },
          "datasource": {
            "query_id": "gts.hypernetix.hyperspot.ax.query.v1~sales.summary.v1",
            "params": { /* OData params */ }
          },
          "size": {
            "width": 100,
            "height": "medium"
          }
        }
      }
    ],
    "settings": {
      "paper_size": "A4",
      "orientation": "portrait",
      "header": {
        "enabled": true,
        "template": "{{company_logo}} {{report_name}}",
        "height": 60
      },
      "footer": {
        "enabled": true,
        "template": "Page {{page_number}} of {{total_pages}} | {{generation_date}}",
        "height": 40
      },
      "page_breaks": {
        "enabled": true,
        "break_after_widgets": ["summary", "details"]
      },
      "color_mode": "color",
      "schedule": {
        "enabled": true,
        "frequency": "weekly",
        "cron": "0 9 * * MON",
        "timezone": "America/New_York",
        "recipients": [
          "sales-team@acme.com",
          "management@acme.com"
        ],
        "delivery_format": "PDF"
      }
    }
  }
}
```

---

## Report Configuration

### Paper Settings

- **paper_size**: `A4`, `Letter`, `Legal`, `A3`, `Tabloid`
- **orientation**: `portrait`, `landscape`

### Header/Footer

- **enabled**: Boolean
- **template**: String with placeholders
  - `{{company_logo}}` - Company logo image
  - `{{report_name}}` - Report title
  - `{{page_number}}` - Current page number
  - `{{total_pages}}` - Total page count
  - `{{generation_date}}` - Report generation timestamp
  - `{{user_name}}` - Report generator name
- **height**: Height in pixels

### Page Breaks

- **enabled**: Boolean
- **break_after_widgets**: Array of widget IDs to insert page break after

### Color Mode

- `color` - Full color rendering
- `grayscale` - Black and white rendering

### Schedule Configuration

- **enabled**: Boolean
- **frequency**: `daily`, `weekly`, `monthly`, `custom`
- **cron**: Cron expression for custom schedules
- **timezone**: IANA timezone identifier
- **recipients**: Array of email addresses or distribution lists
- **delivery_format**: `PDF`, `Excel`, `CSV`

---

## Report Generation Flow

### On-Demand Generation

1. **User triggers generation** - Click "Generate Report" button
2. **Validate report configuration** - Check all widgets, templates, queries exist
3. **Execute queries** - Run all widget queries with current parameters
4. **Render widgets** - Generate widget visualizations using templates
5. **Compose report** - Combine widgets into report layout with headers/footers
6. **Export to format** - Generate PDF/Excel/CSV file
7. **Return download URL** - Provide temporary download link

### Scheduled Generation

1. **Platform scheduler triggers job** - Cron expression matches current time
2. **Load report configuration** - Fetch report from GTS registry
3. **Check report enabled** - Verify report not deleted, schedule still active
4. **Execute generation** - Same as on-demand generation flow
5. **Store report file** - Save generated file to storage
6. **Send email via platform** - Call Hyperspot Platform Email Service
7. **Log delivery** - Record generation and delivery status

---

## Report Generation API

### Generate Report (On-Demand)

```
POST /api/analytics/v1/reports/{report-id}/generate
```

**Request Body (optional):**
```json
{
  "format": "PDF",
  "filters": {
    "date_range": "last_7_days",
    "region": "EMEA"
  },
  "delivery": {
    "email": "user@example.com"
  }
}
```

**Response:**
```json
{
  "report_id": "gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~acme.sales._.weekly.v1",
  "generation_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "completed",
  "format": "PDF",
  "download_url": "https://storage.example.com/reports/550e8400.pdf",
  "expires_at": "2024-01-08T18:00:00Z",
  "generated_at": "2024-01-08T12:00:00Z",
  "size_bytes": 1024768,
  "pages": 15
}
```

---

### Get Report Generation Status

```
GET /api/analytics/v1/reports/{report-id}/generations/{generation-id}
```

**Response:**
```json
{
  "generation_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "processing",
  "progress": 45,
  "started_at": "2024-01-08T12:00:00Z",
  "estimated_completion": "2024-01-08T12:02:00Z"
}
```

**Status values:**
- `queued` - Generation job queued
- `processing` - Report being generated
- `completed` - Report ready for download
- `failed` - Generation failed with error
- `cancelled` - User cancelled generation

---

### List Report History

```
GET /api/analytics/v1/reports/{report-id}/generations
```

**Query Parameters:**
- `$filter` - Filter by date range, status, format
- `$orderby` - Sort by generated_at
- `$top` - Page size

**Response:**
```json
{
  "@odata.context": "...",
  "@odata.count": 156,
  "@odata.nextLink": "...",
  "value": [
    {
      "generation_id": "550e8400-e29b-41d4-a716-446655440000",
      "format": "PDF",
      "status": "completed",
      "generated_at": "2024-01-08T12:00:00Z",
      "generated_by": "user@example.com",
      "size_bytes": 1024768,
      "download_url": "https://storage.example.com/reports/550e8400.pdf"
    }
  ]
}
```

---

## Scheduling Integration

### Platform Scheduling Service

Reports are scheduled via **Hyperspot Platform Scheduling Service**, not internal scheduler.

**Integration Flow:**

1. **Report created with schedule** - User enables schedule in report settings
2. **Register job with platform** - Analytics calls Platform Scheduling API:
   ```
   POST /api/platform/v1/scheduling/jobs
   Body: {
     "id": "analytics.report.{report_id}",
     "cron": "0 9 * * MON",
     "timezone": "America/New_York",
     "callback_url": "https://analytics/api/v1/reports/{report_id}/generate",
     "enabled": true
   }
   ```
3. **Platform manages schedule** - Cron execution handled by platform
4. **Platform calls callback** - At scheduled time, platform POSTs to callback URL
5. **Analytics generates report** - Report generation flow executes
6. **Analytics returns result** - Status returned to platform

**Schedule Management:**

- **Create:** Report with schedule → Register with platform
- **Update:** Schedule modified → Update platform job
- **Delete:** Report deleted → Unregister from platform
- **Disable:** Schedule disabled → Pause platform job

---

## Email Delivery Integration

### Platform Email Service

Report delivery uses **Hyperspot Platform Email Service**, not SMTP integration.

**Integration Flow:**

1. **Report generated** - File ready for delivery
2. **Call Platform Email API:**
   ```
   POST /api/platform/v1/email/send
   Body: {
     "to": ["sales-team@acme.com"],
     "subject": "Weekly Sales Report - {{date}}",
     "body": "Your scheduled report is attached.",
     "attachments": [
       {
         "filename": "weekly-sales-2024-01-08.pdf",
         "content_type": "application/pdf",
         "url": "https://storage.example.com/reports/550e8400.pdf"
       }
     ],
     "template": "report-delivery",
     "tenant_id": "tenant-123"
   }
   ```
3. **Platform sends email** - Email infrastructure handled by platform
4. **Platform returns status** - Delivery status returned
5. **Analytics logs delivery** - Record in report history

**Email Template Variables:**
- `{{report_name}}` - Report title
- `{{generation_date}}` - When report was generated
- `{{download_url}}` - Link to download report
- `{{recipient_name}}` - Recipient's name
- `{{tenant_name}}` - Tenant/organization name

---

## Multi-Format Export

### PDF Export

**Features:**
- Vector graphics for charts (high quality)
- Embedded fonts
- Clickable links and TOC
- Page numbers and headers/footers
- Bookmarks for sections

**Libraries:**
- Server-side rendering: Puppeteer, wkhtmltopdf
- Layout engine: CSS Paged Media

### Excel Export

**Features:**
- Multiple worksheets (one per widget)
- Formatted tables with styling
- Charts embedded as images or native Excel charts
- Formulas for calculations
- Auto-column sizing

**Libraries:**
- exceljs, xlsx, openpyxl

### CSV Export

**Features:**
- Flattened data structure
- UTF-8 encoding with BOM
- Configurable delimiter (comma, semicolon, tab)
- Header row with column names
- One file per table widget

**Use Case:**
- Data imports to other systems
- Simple analysis in spreadsheet tools

---

## Report Versioning

### Version History

Track report configuration changes:

```sql
CREATE TABLE report_versions (
    report_id VARCHAR(500) NOT NULL,
    version_number INTEGER NOT NULL,
    configuration JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    created_by VARCHAR(255),
    change_summary TEXT,
    PRIMARY KEY (report_id, version_number)
);

CREATE INDEX idx_report_versions_report_id ON report_versions(report_id);
CREATE INDEX idx_report_versions_created_at ON report_versions(created_at DESC);
```

**Version Creation:**
- Automatic on each report save
- Change summary can be user-provided or auto-generated
- Full configuration snapshot stored

**Version Rollback:**
```
POST /api/analytics/v1/reports/{report_id}/rollback
Body: { "version_number": 5 }
```

---

## Report Access Control

**Access Levels:**

- **View:** Can view and generate reports
- **Edit:** Can modify report configuration
- **Schedule:** Can set up and manage schedules
- **Share:** Can share reports with other users/tenants
- **Delete:** Can delete reports

**Tenant Enablement:**
- Reports inherit tenant enablement system
- Automatic dependency enablement (widgets, templates, datasources, queries)
- Share with specific tenants or all tenants

---

## Performance Optimization

### Caching

- **Template bundles:** Cached by browser
- **Query results:** Cached per query (5 min TTL)
- **Generated reports:** Cached for download (24 hours)

### Async Generation

- Large reports (>50 pages) generated asynchronously
- Job queue for generation requests
- Webhook or polling for completion notification

### Parallel Processing

- Execute widget queries in parallel
- Render widgets concurrently
- Merge results in final report

---

## Error Handling

**Generation Failures:**

- **Query timeout:** Retry with extended timeout
- **Template render error:** Show error widget placeholder
- **Export failure:** Retry with different library
- **Delivery failure:** Retry up to 3 times, then notify user

**Schedule Failures:**

- Log failure reason
- Send error notification to recipients
- Retry next scheduled time
- Disable schedule after 5 consecutive failures

---

## Database Schema

```sql
CREATE TABLE report_generations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_id VARCHAR(500) NOT NULL,
    format VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL,
    progress INTEGER,
    file_url TEXT,
    file_size_bytes BIGINT,
    pages INTEGER,
    generated_at TIMESTAMPTZ,
    generated_by VARCHAR(255),
    error_message TEXT,
    filters JSONB,
    FOREIGN KEY (report_id) REFERENCES report_layouts(id)
);

CREATE TABLE report_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    generation_id UUID NOT NULL,
    recipient VARCHAR(255) NOT NULL,
    delivery_method VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL,
    delivered_at TIMESTAMPTZ,
    error_message TEXT,
    FOREIGN KEY (generation_id) REFERENCES report_generations(id)
);

CREATE INDEX idx_report_generations_report_id ON report_generations(report_id);
CREATE INDEX idx_report_generations_status ON report_generations(status);
CREATE INDEX idx_report_generations_generated_at ON report_generations(generated_at DESC);
CREATE INDEX idx_report_deliveries_generation_id ON report_deliveries(generation_id);
```

---

### Access Control

**SecurityCtx Enforcement**:
- All report operations require authenticated user
- Tenant isolation enforced on all reports
- Report ownership via `created_by` field
- Schedule management requires admin permissions

**Permission Checks**:
- Report generation: Requires `analytics:reports:generate`
- Schedule management: Requires `analytics:reports:schedule` + ownership verification

---

### Database Operations

**Tables**:
- `report_generations` - Generation history
- `report_deliveries` - Delivery tracking
- `report_versions` - Configuration versioning

**Indexes**:
- `idx_report_generations_report_id` - Generations by report
- `idx_report_generations_status` - Active generations
- `idx_report_generations_generated_at` - Recent history
- `idx_report_deliveries_generation_id` - Deliveries by generation

---

### Error Handling

**Common Errors**:
- **404 Not Found**: Report not found
- **400 Bad Request**: Invalid generation parameters
- **503 Service Unavailable**: Export engine unavailable
- **504 Gateway Timeout**: Generation timeout
- **500 Internal Server Error**: Platform service integration failure

**Error Response Format (RFC 7807)**:
```json
{
  "type": "https://example.com/problems/report-generation-failed",
  "title": "Report Generation Failed",
  "status": 500,
  "detail": "Widget 'sales-summary' query execution failed",
  "instance": "/api/analytics/v1/reports/acme.sales._.weekly.v1"
}
```

---

## F. Validation & Implementation

### Testing Scenarios

**Unit Tests**:
- Report generation flow logic
- Widget query parallel execution
- Export format conversion
- Schedule registration
- Email template rendering

**Integration Tests**:
- Platform Scheduling Service integration
- Platform Email Service integration
- End-to-end report generation
- Multi-format export
- Version rollback

**Performance Tests**:
- Small report generation (< 10 pages, < 2s)
- Large report generation (100+ pages, < 30s)
- Concurrent generation (10+ reports)
- File storage and retrieval

**Edge Cases**:
1. Report with 50+ widgets
2. Widget query timeout during generation
3. Platform service temporarily unavailable
4. Invalid email recipients
5. Expired download link access
6. Schedule disabled during generation

---

### OpenSpec Changes Plan

#### Change 001: Report Generation Engine
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/reporting/generator.rs`
- **Description**: Core report generation orchestration
- **Dependencies**: None (foundational)
- **Effort**: 3.5 hours (AI agent)
- **Validation**: Unit tests, integration tests

#### Change 002: PDF Export Engine
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/reporting/exporters/pdf.rs`
- **Description**: PDF generation with Puppeteer/wkhtmltopdf
- **Dependencies**: Change 001
- **Effort**: 3 hours (AI agent)
- **Validation**: PDF output validation

#### Change 003: Excel/CSV Exporters
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/reporting/exporters/excel.rs`
  - `modules/analytics/src/domain/reporting/exporters/csv.rs`
- **Description**: Excel and CSV export implementation
- **Dependencies**: Change 001
- **Effort**: 2 hours (AI agent)
- **Validation**: Export format validation

#### Change 004: Platform Scheduling Integration
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/reporting/scheduling.rs`
- **Description**: Register and manage schedules with Platform Scheduling Service
- **Dependencies**: Change 001
- **Effort**: 2 hours (AI agent)
- **Validation**: Integration tests with mock platform

#### Change 005: Platform Email Integration
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/reporting/delivery.rs`
- **Description**: Send reports via Platform Email Service
- **Dependencies**: Change 001
- **Effort**: 1.5 hours (AI agent)
- **Validation**: Email delivery tests

#### Change 006: Report History Management
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/reporting/history.rs`
  - `modules/analytics/src/api/rest/reporting/handlers.rs`
- **Description**: Track generation history, provide download links
- **Dependencies**: Change 001
- **Effort**: 1.5 hours (AI agent)
- **Validation**: History query tests

#### Change 007: Report Versioning
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/reporting/versioning.rs`
- **Description**: Version tracking and rollback
- **Dependencies**: Change 001
- **Effort**: 1 hour (AI agent)
- **Validation**: Rollback tests

#### Change 008: Async Generation Queue
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/reporting/queue.rs`
- **Description**: Job queue for large report generation
- **Dependencies**: Change 001
- **Effort**: 2 hours (AI agent)
- **Validation**: Queue processing tests

#### Change 009: OpenAPI Specification
- **Type**: openapi
- **Files**: 
  - `architecture/openapi/v1/api.yaml`
- **Description**: Document report generation endpoints
- **Dependencies**: All previous changes
- **Effort**: 0.5 hours (AI agent)
- **Validation**: Swagger validation

#### Change 010: Integration Testing Suite
- **Type**: rust (tests)
- **Files**: 
  - `tests/integration/reporting_test.rs`
- **Description**: End-to-end report lifecycle tests
- **Dependencies**: All previous changes
- **Effort**: 2 hours (AI agent)
- **Validation**: 100% scenario coverage

**Total Effort**: 18 hours (AI agent + OpenSpec)

---

## Dependencies

- **Depends On**: 
  - feature-report-layouts (report layout type)
  - feature-dashboards (widget management patterns)
  - feature-query-execution (widget query execution)
- **Platform Dependencies**:
  - Hyperspot Platform Scheduling Service
  - Hyperspot Platform Email Service
- **Blocks**: (none)

---

## References

- Overall Design: `architecture/DESIGN.md` Section 2 (Actors), Section 3 (System Capabilities)
- GTS Types: Report layout schemas (owned by feature-report-layouts)
- OpenAPI Spec: `architecture/openapi/v1/api.yaml` (reporting endpoints)
- Feature Manifest: `architecture/features/FEATURES.md` (feature-reporting entry)
- Platform Services: Scheduling Service, Email Service integration
