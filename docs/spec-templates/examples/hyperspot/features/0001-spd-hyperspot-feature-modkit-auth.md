# Feature: Module Authentication (modkit-auth)

## 1. Feature Context

**ID**: `spd-hyperspot-feature-modkit-auth`
**Status**: NOT_STARTED

### 1.1 Overview

Provides JWT-based authentication middleware for modules, enabling stateless authentication with role-based access control (RBAC) and tenant isolation.

### 1.2 Purpose

Implements secure authentication for module API endpoints, addresses PRD requirements for multi-tenant access control, and provides DESIGN-compliant JWT validation with claims-based authorization. Ensures tenant data isolation and enables fine-grained access control at the module level.

### 1.3 Actors

| Actor | Role in Feature |
|-------|-----------------|
| `spd-hyperspot-actor-end-user` | Authenticates to access module APIs, receives JWT tokens |
| `spd-hyperspot-actor-saas-developer` | Configures authentication middleware for module endpoints |
| `spd-hyperspot-actor-api-gateway` | Routes authenticated requests to modules, validates JWT tokens |
| `spd-hyperspot-actor-tenant-admin` | Manages user roles and permissions within tenant scope |

### 1.4 References

- **PRD**: [PRD.md](../PRD.md) — Multi-tenant access control requirements
- **Design**: [DESIGN.md](../DESIGN.md) — Authentication architecture
- **Dependencies**: ClientHub, Database Manager, API Gateway

## 2. Actor Flows (SDSL)

### User Login Flow

- [ ] **ID**: `spd-hyperspot-flow-auth-login`

**Actor**: `spd-hyperspot-actor-end-user`

**Success Scenarios**:
- User authenticates with valid credentials and receives JWT access token and refresh token
- JWT contains tenant_id, user_id, roles, and expiration claims

**Error Scenarios**:
- Invalid credentials return 401 Unauthorized
- Account locked/disabled returns 403 Forbidden
- Rate limit exceeded returns 429 Too Many Requests

**Steps**:
1. [ ] - `ph-1` - User submits credentials (email, password) via POST /auth/login - `inst-login-1`
2. [ ] - `ph-1` - API: POST /auth/login { email, password } - `inst-login-2`
3. [ ] - `ph-1` - DB: SELECT user_id, password_hash, tenant_id, status, failed_attempts FROM users WHERE email = :email - `inst-login-3`
4. [ ] - `ph-1` - **IF** user not found OR status = 'disabled' - `inst-login-4`
   1. [ ] - `ph-1` - **RETURN** 401 Unauthorized { error: "Invalid credentials" } - `inst-login-4a`
5. [ ] - `ph-1` - **IF** failed_attempts >= 5 - `inst-login-5`
   1. [ ] - `ph-1` - **RETURN** 403 Forbidden { error: "Account locked" } - `inst-login-5a`
6. [ ] - `ph-1` - Validate password using Argon2id hash comparison - `inst-login-6`
7. [ ] - `ph-1` - **IF** password invalid - `inst-login-7`
   1. [ ] - `ph-1` - DB: UPDATE users SET failed_attempts = failed_attempts + 1 WHERE user_id = :userId - `inst-login-7a`
   2. [ ] - `ph-1` - **RETURN** 401 Unauthorized { error: "Invalid credentials" } - `inst-login-7b`
8. [ ] - `ph-1` - DB: SELECT role_name FROM user_roles WHERE user_id = :userId - `inst-login-8`
9. [ ] - `ph-1` - Generate JWT access token with claims: { sub: user_id, tenant_id, roles: [role1, role2], exp: now + 15min } - `inst-login-9`
10. [ ] - `ph-1` - Generate JWT refresh token with claims: { sub: user_id, tenant_id, exp: now + 7days } - `inst-login-10`
11. [ ] - `ph-1` - DB: INSERT INTO refresh_tokens (token_id, user_id, expires_at) VALUES (:tokenId, :userId, :expiresAt) - `inst-login-11`
12. [ ] - `ph-1` - DB: UPDATE users SET failed_attempts = 0, last_login_at = NOW() WHERE user_id = :userId - `inst-login-12`
13. [ ] - `ph-1` - **RETURN** 200 OK { access_token, refresh_token, expires_in: 900 } - `inst-login-13`


### Token Refresh Flow

- [ ] **ID**: `spd-hyperspot-flow-auth-refresh`

**Actor**: `spd-hyperspot-actor-end-user`

**Success Scenarios**:
- User exchanges valid refresh token for new access token
- Refresh token rotation: old refresh token invalidated, new one issued

**Error Scenarios**:
- Expired/invalid refresh token returns 401 Unauthorized
- Revoked refresh token returns 401 Unauthorized

**Steps**:
1. [ ] - `ph-1` - User submits refresh token via POST /auth/refresh - `inst-refresh-1`
2. [ ] - `ph-1` - API: POST /auth/refresh { refresh_token } - `inst-refresh-2`
3. [ ] - `ph-1` - Validate JWT signature and expiration - `inst-refresh-3`
4. [ ] - `ph-1` - **IF** JWT invalid OR expired - `inst-refresh-4`
   1. [ ] - `ph-1` - **RETURN** 401 Unauthorized { error: "Invalid refresh token" } - `inst-refresh-4a`
5. [ ] - `ph-1` - DB: SELECT user_id, revoked_at FROM refresh_tokens WHERE token_id = :tokenId - `inst-refresh-5`
6. [ ] - `ph-1` - **IF** token not found OR revoked_at IS NOT NULL - `inst-refresh-6`
   1. [ ] - `ph-1` - **RETURN** 401 Unauthorized { error: "Token revoked" } - `inst-refresh-6a`
7. [ ] - `ph-1` - DB: SELECT user_id, tenant_id, status FROM users WHERE user_id = :userId - `inst-refresh-7`
8. [ ] - `ph-1` - **IF** user status = 'disabled' - `inst-refresh-8`
   1. [ ] - `ph-1` - **RETURN** 403 Forbidden { error: "Account disabled" } - `inst-refresh-8a`
9. [ ] - `ph-1` - DB: SELECT role_name FROM user_roles WHERE user_id = :userId - `inst-refresh-9`
10. [ ] - `ph-1` - Generate new JWT access token with current roles - `inst-refresh-10`
11. [ ] - `ph-1` - Generate new JWT refresh token - `inst-refresh-11`
12. [ ] - `ph-1` - DB: UPDATE refresh_tokens SET revoked_at = NOW() WHERE token_id = :oldTokenId - `inst-refresh-12`
13. [ ] - `ph-1` - DB: INSERT INTO refresh_tokens (token_id, user_id, expires_at) VALUES (:newTokenId, :userId, :expiresAt) - `inst-refresh-13`
14. [ ] - `ph-1` - **RETURN** 200 OK { access_token, refresh_token, expires_in: 900 } - `inst-refresh-14`


### Authenticated Request Flow

- [ ] **ID**: `spd-hyperspot-flow-auth-request`

**Actor**: `spd-hyperspot-actor-end-user`

**Success Scenarios**:
- User accesses protected module endpoint with valid JWT
- JWT claims extracted and passed to module handler

**Error Scenarios**:
- Missing/invalid JWT returns 401 Unauthorized
- Insufficient permissions return 403 Forbidden
- Tenant mismatch returns 403 Forbidden

**Steps**:
1. [ ] - `ph-1` - User sends request with Authorization: Bearer {access_token} header - `inst-request-1`
2. [ ] - `ph-1` - Middleware: Extract JWT from Authorization header - `inst-request-2`
3. [ ] - `ph-1` - **IF** JWT missing - `inst-request-3`
   1. [ ] - `ph-1` - **RETURN** 401 Unauthorized { error: "Missing authentication token" } - `inst-request-3a`
4. [ ] - `ph-1` - Validate JWT signature using HMAC-SHA256 with secret key - `inst-request-4`
5. [ ] - `ph-1` - **IF** signature invalid OR token expired - `inst-request-5`
   1. [ ] - `ph-1` - **RETURN** 401 Unauthorized { error: "Invalid or expired token" } - `inst-request-5a`
6. [ ] - `ph-1` - Extract claims: { user_id, tenant_id, roles } - `inst-request-6`
7. [ ] - `ph-1` - **IF** endpoint requires specific role - `inst-request-7`
   1. [ ] - `ph-1` - **IF** required_role NOT IN user_roles - `inst-request-7a`
      1. [ ] - `ph-1` - **RETURN** 403 Forbidden { error: "Insufficient permissions" } - `inst-request-7a1`
8. [ ] - `ph-1` - **IF** endpoint requires tenant isolation - `inst-request-8`
   1. [ ] - `ph-1` - **IF** request tenant_id != JWT tenant_id - `inst-request-8a`
      1. [ ] - `ph-1` - **RETURN** 403 Forbidden { error: "Tenant access denied" } - `inst-request-8a1`
9. [ ] - `ph-1` - Attach AuthContext { user_id, tenant_id, roles } to request context - `inst-request-9`
10. [ ] - `ph-1` - Forward request to module handler with AuthContext - `inst-request-10`


## 3. Processes / Business Logic (SDSL)

### JWT Token Generation

- [ ] **ID**: `spd-hyperspot-algo-auth-generate-jwt`

**Input**: user_id, tenant_id, roles (array), token_type (access | refresh)

**Output**: JWT string (header.payload.signature)

**Steps**:
1. [ ] - `ph-1` - Create JWT header: { alg: "HS256", typ: "JWT" } - `inst-jwt-1`
2. [ ] - `ph-1` - **IF** token_type = "access" - `inst-jwt-2`
   1. [ ] - `ph-1` - Create payload: { sub: user_id, tenant_id, roles, iat: now, exp: now + 15min, jti: uuid() } - `inst-jwt-2a`
3. [ ] - `ph-1` - **ELSE** (token_type = "refresh") - `inst-jwt-3`
   1. [ ] - `ph-1` - Create payload: { sub: user_id, tenant_id, iat: now, exp: now + 7days, jti: uuid() } - `inst-jwt-3a`
4. [ ] - `ph-1` - Encode header as base64url - `inst-jwt-4`
5. [ ] - `ph-1` - Encode payload as base64url - `inst-jwt-5`
6. [ ] - `ph-1` - Concatenate: unsigned_token = base64url(header) + "." + base64url(payload) - `inst-jwt-6`
7. [ ] - `ph-1` - Generate HMAC-SHA256 signature: signature = HMAC-SHA256(unsigned_token, SECRET_KEY) - `inst-jwt-7`
8. [ ] - `ph-1` - Encode signature as base64url - `inst-jwt-8`
9. [ ] - `ph-1` - Concatenate final token: jwt = unsigned_token + "." + base64url(signature) - `inst-jwt-9`
10. [ ] - `ph-1` - **RETURN** jwt - `inst-jwt-10`


### JWT Token Validation

- [ ] **ID**: `spd-hyperspot-algo-auth-validate-jwt`

**Input**: JWT string

**Output**: Validated claims or error

**Steps**:
1. [ ] - `ph-1` - Split JWT by "." delimiter into [header, payload, signature] - `inst-validate-1`
2. [ ] - `ph-1` - **IF** parts count != 3 - `inst-validate-2`
   1. [ ] - `ph-1` - **RETURN** Error("Malformed JWT") - `inst-validate-2a`
3. [ ] - `ph-1` - Decode header from base64url - `inst-validate-3`
4. [ ] - `ph-1` - **IF** header.alg != "HS256" - `inst-validate-4`
   1. [ ] - `ph-1` - **RETURN** Error("Unsupported algorithm") - `inst-validate-4a`
5. [ ] - `ph-1` - Decode payload from base64url - `inst-validate-5`
6. [ ] - `ph-1` - Reconstruct unsigned_token = header + "." + payload - `inst-validate-6`
7. [ ] - `ph-1` - Compute expected_signature = HMAC-SHA256(unsigned_token, SECRET_KEY) - `inst-validate-7`
8. [ ] - `ph-1` - Decode provided signature from base64url - `inst-validate-8`
9. [ ] - `ph-1` - **IF** provided_signature != expected_signature (constant-time comparison) - `inst-validate-9`
   1. [ ] - `ph-1` - **RETURN** Error("Invalid signature") - `inst-validate-9a`
10. [ ] - `ph-1` - Parse claims from payload JSON - `inst-validate-10`
11. [ ] - `ph-1` - **IF** claims.exp <= now - `inst-validate-11`
   1. [ ] - `ph-1` - **RETURN** Error("Token expired") - `inst-validate-11a`
12. [ ] - `ph-1` - **RETURN** { user_id: claims.sub, tenant_id: claims.tenant_id, roles: claims.roles } - `inst-validate-12`


### Password Hashing

- [ ] **ID**: `spd-hyperspot-algo-auth-hash-password`

**Input**: plaintext password

**Output**: Argon2id hash string

**Steps**:
1. [ ] - `ph-1` - Generate 16-byte random salt using crypto-secure RNG - `inst-hash-1`
2. [ ] - `ph-1` - Configure Argon2id: memory_cost=19MB, time_cost=2, parallelism=1 - `inst-hash-2`
3. [ ] - `ph-1` - Compute hash: hash = Argon2id(password, salt, params) - `inst-hash-3`
4. [ ] - `ph-1` - Encode hash with salt and params: output = "$argon2id$v=19$m=19456,t=2,p=1${base64(salt)}${base64(hash)}" - `inst-hash-4`
5. [ ] - `ph-1` - **RETURN** output - `inst-hash-5`


### Password Verification

- [ ] **ID**: `spd-hyperspot-algo-auth-verify-password`

**Input**: plaintext password, stored hash

**Output**: boolean (valid/invalid)

**Steps**:
1. [ ] - `ph-1` - Parse stored hash to extract algorithm, version, params, salt, hash - `inst-verify-1`
2. [ ] - `ph-1` - **IF** algorithm != "argon2id" - `inst-verify-2`
   1. [ ] - `ph-1` - **RETURN** false - `inst-verify-2a`
3. [ ] - `ph-1` - Recompute hash using same salt and params: computed_hash = Argon2id(password, salt, params) - `inst-verify-3`
4. [ ] - `ph-1` - Compare computed_hash with stored hash using constant-time comparison - `inst-verify-4`
5. [ ] - `ph-1` - **RETURN** (computed_hash == stored_hash) - `inst-verify-5`


## 4. States (SDSL)

### User Session State Machine

- [ ] **ID**: `spd-hyperspot-state-auth-session`

**States**: unauthenticated, authenticated, token_expired, session_revoked

**Initial State**: unauthenticated

**Transitions**:
1. [ ] - `ph-1` - **FROM** unauthenticated **TO** authenticated **WHEN** valid login with credentials - `inst-session-1`
2. [ ] - `ph-1` - **FROM** authenticated **TO** token_expired **WHEN** JWT exp claim exceeded - `inst-session-2`
3. [ ] - `ph-1` - **FROM** token_expired **TO** authenticated **WHEN** valid refresh token exchanged - `inst-session-3`
4. [ ] - `ph-1` - **FROM** authenticated **TO** session_revoked **WHEN** admin revokes refresh token - `inst-session-4`
5. [ ] - `ph-1` - **FROM** session_revoked **TO** authenticated **WHEN** new login completed - `inst-session-5`
6. [ ] - `ph-1` - **FROM** token_expired **TO** unauthenticated **WHEN** refresh token expired/invalid - `inst-session-6`


## 5. Implementation Requirements

### Implement JWT Authentication Middleware

- [ ] **ID**: `spd-hyperspot-req-auth-middleware`

**Status**: NOT_STARTED

The system **MUST** provide Axum middleware for JWT authentication that validates tokens, extracts claims, and injects AuthContext into request handlers.

**Implements**:
- `spd-hyperspot-flow-auth-request`
- `spd-hyperspot-algo-auth-validate-jwt`

**Touches**:
- API: All protected module endpoints
- Middleware: `JwtAuthMiddleware`, `AuthContext`
- Config: JWT secret key management

**Phases**:
- [ ] `ph-1`: Core JWT validation middleware
- [ ] `ph-2`: Role-based access control decorators


### Implement Login and Token Endpoints

- [ ] **ID**: `spd-hyperspot-req-auth-endpoints`

**Status**: NOT_STARTED

The system **MUST** provide `/auth/login`, `/auth/refresh`, and `/auth/logout` endpoints with rate limiting and brute-force protection.

**Implements**:
- `spd-hyperspot-flow-auth-login`
- `spd-hyperspot-flow-auth-refresh`

**Touches**:
- API: `POST /auth/login`, `POST /auth/refresh`, `POST /auth/logout`
- DB: `users`, `refresh_tokens`, `user_roles`
- Entities: `User`, `RefreshToken`, `UserRole`

**Phases**:
- [ ] `ph-1`: Login and refresh endpoints
- [ ] `ph-2`: Rate limiting (5 req/min per IP)


### Implement Password Hashing with Argon2id

- [ ] **ID**: `spd-hyperspot-req-auth-password`

**Status**: NOT_STARTED

The system **MUST** use Argon2id for password hashing with secure defaults (19MB memory, 2 iterations, parallelism 1).

**Implements**:
- `spd-hyperspot-algo-auth-hash-password`
- `spd-hyperspot-algo-auth-verify-password`

**Touches**:
- Entities: `User.password_hash` field
- Dependencies: `argon2` crate

**Phases**:
- [ ] `ph-1`: Password hashing functions


### Implement Tenant Isolation Enforcement

- [ ] **ID**: `spd-hyperspot-req-auth-tenant-isolation`

**Status**: NOT_STARTED

The system **MUST** enforce tenant_id matching between JWT claims and requested resources, returning 403 Forbidden for tenant mismatches.

**Implements**:
- `spd-hyperspot-flow-auth-request` (tenant validation step)

**Touches**:
- Middleware: Tenant isolation filter
- DB: All tenant-scoped queries must filter by JWT tenant_id

**Phases**:
- [ ] `ph-1`: Automatic tenant_id injection in queries


## 6. Acceptance Criteria

- [ ] User can successfully login with valid credentials and receive access + refresh tokens
- [ ] Invalid credentials return 401 Unauthorized without leaking user existence
- [ ] Account locks after 5 failed login attempts
- [ ] JWT access tokens expire after 15 minutes
- [ ] JWT refresh tokens expire after 7 days
- [ ] User can exchange valid refresh token for new access token
- [ ] Expired/revoked refresh tokens are rejected with 401 Unauthorized
- [ ] Authenticated requests include AuthContext with user_id, tenant_id, and roles
- [ ] Requests without valid JWT return 401 Unauthorized
- [ ] Requests with valid JWT but insufficient role return 403 Forbidden
- [ ] Requests attempting cross-tenant access return 403 Forbidden
- [ ] Password hashes use Argon2id with secure defaults
- [ ] Rate limiting prevents brute-force attacks (max 5 login attempts per minute per IP)
- [ ] All authentication endpoints respond within 200ms at p95
- [ ] JWT signature validation uses constant-time comparison to prevent timing attacks
