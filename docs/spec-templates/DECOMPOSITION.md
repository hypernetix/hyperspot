# Decomposition: {PROJECT_NAME}

## 1. Overview

{ Description of how the DESIGN was decomposed into specs, the decomposition strategy, and any relevant decomposition rationale. }


## 2. Entries

**Overall implementation status:**
- [ ] `p1` - **ID**: `spd-{system}-status-overall`

### 1. [{Spec Title 1}](spec-{slug}/) - HIGH

- [ ] `p1` - **ID**: `spd-{system}-spec-{slug}`

- **Purpose**: {Few sentences describing what this spec accomplishes and why it matters}

- **Depends On**: None

- **Scope**:
  - {in-scope item 1}
  - {in-scope item 2}

- **Out of scope**:
  - {out-of-scope item 1}
  - {out-of-scope item 2}

- **Requirements Covered**:
  - [ ] `p1` - `spd-{system}-fr-{slug}`
  - [ ] `p1` - `spd-{system}-nfr-{slug}`

- **Design Principles Covered**:
  - [ ] `p1` - `spd-{system}-principle-{slug}`

- **Design Constraints Covered**:
  - [ ] `p1` - `spd-{system}-constraint-{slug}`

- **Domain Model Entities**:
  - {entity 1}
  - {entity 2}

- **Design Components**:
  - [ ] `p1` - `spd-{system}-component-{slug}`

- **API**:
  - POST /api/{resource}
  - GET /api/{resource}/{id}
  - {CLI command}

- **Sequences**:
  - [ ] `p1` - `spd-{system}-seq-{slug}`

- **Data**:
  - [ ] `p1` - `spd-{system}-dbtable-{slug}`

### 2. [{Spec Title 2}](spec-{slug}/) - MEDIUM

- [ ] `p2` - **ID**: `spd-{system}-spec-{slug}`

- **Purpose**: {Few sentences describing what this spec accomplishes and why it matters}

- **Depends On**: `spd-{system}-spec-{slug}` (previous spec)

- **Scope**:
  - {in-scope item 1}
  - {in-scope item 2}

- **Out of scope**:
  - {out-of-scope item 1}

- **Requirements Covered**:
  - [ ] `p2` - `spd-{system}-fr-{slug}`

- **Design Principles Covered**:
  - [ ] `p2` - `spd-{system}-principle-{slug}`

- **Design Constraints Covered**:
  - [ ] `p2` - `spd-{system}-constraint-{slug}`

- **Domain Model Entities**:
  - {entity}

- **Design Components**:
  - [ ] `p2` - `spd-{system}-component-{slug}`

- **API**:
  - PUT /api/{resource}/{id}
  - DELETE /api/{resource}/{id}

- **Sequences**:
  - [ ] `p2` - `spd-{system}-seq-{slug}`

- **Data**:
  - [ ] `p2` - `spd-{system}-dbtable-{slug}`

### 3. [{Spec Title 3}](spec-{slug}/) - LOW

- [ ] `p3` - **ID**: `spd-{system}-spec-{slug}`

- **Purpose**: {Few sentences describing what this spec accomplishes and why it matters}

- **Depends On**: `spd-{system}-spec-{slug}`

- **Scope**:
  - {in-scope item}

- **Out of scope**:
  - {out-of-scope item}

- **Requirements Covered**:
  - [ ] `p3` - `spd-{system}-fr-{slug}`

- **Design Principles Covered**:
  - [ ] `p3` - `spd-{system}-principle-{slug}`

- **Design Constraints Covered**:
  - [ ] `p3` - `spd-{system}-constraint-{slug}`

- **Domain Model Entities**:
  - {entity}

- **Design Components**:
  - [ ] `p3` - `spd-{system}-component-{slug}`

- **API**:
  - GET /api/{resource}/stats

- **Sequences**:
  - [ ] `p3` - `spd-{system}-seq-{slug}`

- **Data**:
  - [ ] `p3` - `spd-{system}-dbtable-{slug}`
