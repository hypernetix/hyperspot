use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use modkit::Problem;
use modkit_auth::{AuthDispatcher, Claims};
use modkit_security::{SecurityCtx, AccessScope, Subject};
use std::sync::Arc;

pub async fn jwt_validation_middleware(
    auth_dispatcher: Arc<AuthDispatcher>,
    mut req: Request,
    next: Next,
) -> Result<Response, Problem> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            Problem::new(
                StatusCode::UNAUTHORIZED,
                "Missing Authorization Header",
                "Authorization header is required for all requests",
            )
        })?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            Problem::new(
                StatusCode::UNAUTHORIZED,
                "Invalid Authorization Header",
                "Authorization header must start with 'Bearer '",
            )
        })?;

    let claims = auth_dispatcher
        .validate_jwt(token)
        .await
        .map_err(|e| {
            Problem::new(
                StatusCode::UNAUTHORIZED,
                "Invalid JWT Token",
                format!("JWT validation failed: {}", e),
            )
        })?;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

pub async fn security_ctx_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, Problem> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| {
            Problem::new(
                StatusCode::UNAUTHORIZED,
                "Missing JWT Claims",
                "JWT claims not found in request extensions",
            )
        })?;

    let tenant_uuid = claims
        .extras
        .get("tenant_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .ok_or_else(|| {
            Problem::new(
                StatusCode::UNAUTHORIZED,
                "Missing or Invalid Tenant ID",
                "JWT token must contain valid tenant_id UUID claim",
            )
        })?;

    let scope = AccessScope::tenant(tenant_uuid);
    let subject = Subject::new(claims.subject);
    let security_ctx = SecurityCtx::new(scope, subject);
    req.extensions_mut().insert(security_ctx);

    Ok(next.run(req).await)
}

pub async fn odata_parser_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, Problem> {
    let query_string = req.uri().query().unwrap_or("");
    
    if !query_string.is_empty() {
        let odata_params = parse_odata_params(query_string).map_err(|e| {
            Problem::new(
                StatusCode::BAD_REQUEST,
                "Invalid OData Parameters",
                format!("Failed to parse OData query parameters: {}", e),
            )
        })?;

        req.extensions_mut().insert(odata_params);
    }

    Ok(next.run(req).await)
}

#[derive(Debug, Clone)]
pub struct ODataParams {
    pub filter: Option<String>,
    pub select: Option<Vec<String>>,
    pub orderby: Option<String>,
    pub top: Option<usize>,
    pub skip: Option<usize>,
    pub count: bool,
}

fn parse_odata_params(query_string: &str) -> Result<ODataParams, String> {
    let mut params = ODataParams {
        filter: None,
        select: None,
        orderby: None,
        top: None,
        skip: None,
        count: false,
    };

    if query_string.is_empty() {
        return Ok(params);
    }

    for pair in query_string.split('&') {
        if pair.is_empty() {
            continue;
        }
        
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().ok_or("Invalid query parameter")?;
        let value = parts.next().ok_or("Missing value for parameter")?;

        match key {
            "$filter" => params.filter = Some(urlencoding::decode(value).map_err(|e| e.to_string())?.into_owned()),
            "$select" => {
                params.select = Some(
                    urlencoding::decode(value)
                        .map_err(|e| e.to_string())?
                        .split(',')
                        .map(|s| s.to_string())
                        .collect()
                );
            }
            "$orderby" => params.orderby = Some(urlencoding::decode(value).map_err(|e| e.to_string())?.into_owned()),
            "$top" => params.top = Some(value.parse().map_err(|e| format!("Invalid $top value: {}", e))?),
            "$skip" => params.skip = Some(value.parse().map_err(|e| format!("Invalid $skip value: {}", e))?),
            "$count" => params.count = value == "true",
            _ => {}
        }
    }

    Ok(params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{header, Request as HttpRequest};
    use axum::body::Body;

    #[test]
    fn test_parse_odata_params_filter() {
        let query = "$filter=entity/name eq 'test'";
        let params = parse_odata_params(query).unwrap();
        assert_eq!(params.filter, Some("entity/name eq 'test'".to_string()));
    }

    #[test]
    fn test_parse_odata_params_select() {
        let query = "$select=entity/name,entity/age";
        let params = parse_odata_params(query).unwrap();
        assert_eq!(
            params.select,
            Some(vec!["entity/name".to_string(), "entity/age".to_string()])
        );
    }

    #[test]
    fn test_parse_odata_params_complex() {
        let query = "$filter=entity/name eq 'test'&$select=entity/name&$top=10&$count=true";
        let params = parse_odata_params(query).unwrap();
        assert_eq!(params.filter, Some("entity/name eq 'test'".to_string()));
        assert_eq!(params.select, Some(vec!["entity/name".to_string()]));
        assert_eq!(params.top, Some(10));
        assert_eq!(params.count, true);
    }

    #[test]
    fn test_parse_odata_params_with_orderby_and_skip() {
        let query = "$orderby=entity/created_at desc&$skip=20";
        let params = parse_odata_params(query).unwrap();
        assert_eq!(params.orderby, Some("entity/created_at desc".to_string()));
        assert_eq!(params.skip, Some(20));
    }

    #[test]
    fn test_parse_odata_params_invalid_top() {
        let query = "$top=invalid";
        let result = parse_odata_params(query);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid $top value"));
    }

    #[test]
    fn test_parse_odata_params_empty_query() {
        let query = "";
        let params = parse_odata_params(query).unwrap();
        assert!(params.filter.is_none());
        assert!(params.select.is_none());
        assert!(params.orderby.is_none());
        assert!(params.top.is_none());
        assert!(params.skip.is_none());
        assert_eq!(params.count, false);
    }
}
