# Decomposition: {PROJECT_NAME}

## 1. Overview

{ Description of how the DESIGN was decomposed into specs, the decomposition strategy, and any relevant decomposition rationale. }


## 2. Entries

**Overall implementation status:**
- [ ] `p1` - **ID**: `cpt-{system}-status-overall`

### 1. [{Spec Title 1}](spec-{slug}/) - HIGH

- [ ] `p1` - **ID**: `cpt-{system}-spec-{slug}`

- **Purpose**: {Few sentences describing what this feature accomplishes and why it matters}

- **Depends On**: None

- **Scope**:
  - {in-scope item 1}
  - {in-scope item 2}

- **Out of scope**:
  - {out-of-scope item 1}
  - {out-of-scope item 2}

- **Requirements Covered**:
  - [ ] `p1` - `cpt-{system}-fr-{slug}`
  - [ ] `p1` - `cpt-{system}-nfr-{slug}`

- **Design Principles Covered**:
  - [ ] `p1` - `cpt-{system}-principle-{slug}`

- **Design Constraints Covered**:
  - [ ] `p1` - `cpt-{system}-constraint-{slug}`

- **Domain Model Entities**:
  - {entity 1}
  - {entity 2}

- **Design Components**:
  - [ ] `p1` - `cpt-{system}-component-{slug}`

- **API**:
  - POST /api/{resource}
  - GET /api/{resource}/{id}
  - {CLI command}

- **Sequences**:
  - [ ] `p1` - `cpt-{system}-seq-{slug}`

- **Data**:
  - [ ] `p1` - `cpt-{system}-dbtable-{slug}`

### 2. [{Spec Title 2}](spec-{slug}/) - MEDIUM

- [ ] `p2` - **ID**: `cpt-{system}-spec-{slug}`

- **Purpose**: {Few sentences describing what this feature accomplishes and why it matters}

- **Depends On**: `cpt-{system}-spec-{slug}` (previous spec)

- **Scope**:
  - {in-scope item 1}
  - {in-scope item 2}

- **Out of scope**:
  - {out-of-scope item 1}

- **Requirements Covered**:
  - [ ] `p2` - `cpt-{system}-fr-{slug}`

- **Design Principles Covered**:
  - [ ] `p2` - `cpt-{system}-principle-{slug}`

- **Design Constraints Covered**:
  - [ ] `p2` - `cpt-{system}-constraint-{slug}`

- **Domain Model Entities**:
  - {entity}

- **Design Components**:
  - [ ] `p2` - `cpt-{system}-component-{slug}`

- **API**:
  - PUT /api/{resource}/{id}
  - DELETE /api/{resource}/{id}

- **Sequences**:
  - [ ] `p2` - `cpt-{system}-seq-{slug}`

- **Data**:
  - [ ] `p2` - `cpt-{system}-dbtable-{slug}`

### 3. [{Spec Title 3}](spec-{slug}/) - LOW

- [ ] `p3` - **ID**: `cpt-{system}-spec-{slug}`

- **Purpose**: {Few sentences describing what this feature accomplishes and why it matters}

- **Depends On**: `cpt-{system}-spec-{slug}`

- **Scope**:
  - {in-scope item}

- **Out of scope**:
  - {out-of-scope item}

- **Requirements Covered**:
  - [ ] `p3` - `cpt-{system}-fr-{slug}`

- **Design Principles Covered**:
  - [ ] `p3` - `cpt-{system}-principle-{slug}`

- **Design Constraints Covered**:
  - [ ] `p3` - `cpt-{system}-constraint-{slug}`

- **Domain Model Entities**:
  - {entity}

- **Design Components**:
  - [ ] `p3` - `cpt-{system}-component-{slug}`

- **API**:
  - GET /api/{resource}/stats

- **Sequences**:
  - [ ] `p3` - `cpt-{system}-seq-{slug}`

- **Data**:
  - [ ] `p3` - `cpt-{system}-dbtable-{slug}`
