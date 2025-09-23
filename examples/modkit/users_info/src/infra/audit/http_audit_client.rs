use anyhow::Context;
use async_trait::async_trait;
use tracing::instrument;
use url::Url;
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::ports::AuditPort;
use modkit::TracedClient;

/// Single HTTP adapter implementing the AuditPort.
/// Holds two base URLs:
///  - audit_base (e.g., http://audit.local)
///  - notify_base (e.g., http://notifications.local)
pub struct HttpAuditClient {
    client: TracedClient,
    audit_base: Url,
    notify_base: Url,
}

impl HttpAuditClient {
    pub fn new(client: TracedClient, audit_base: Url, notify_base: Url) -> Self {
        Self {
            client,
            audit_base,
            notify_base,
        }
    }
}

#[async_trait]
impl AuditPort for HttpAuditClient {
    #[instrument(
        name = "users_info.http.audit.get_user_access",
        skip_all,
        fields(audit_base = %self.audit_base, user_id = %id)
    )]
    async fn get_user_access(&self, id: Uuid) -> Result<(), DomainError> {
        let mut url = self.audit_base.clone();
        url.path_segments_mut()
            .map_err(|_| DomainError::validation("user_access", "invalid audit base URL"))?
            .extend(&["api", "user-access", &id.to_string()]);

        let response = self
            .client
            .get(url.as_str())
            .await
            .with_context(|| format!("GET /api/user-access/{}", id))
            .map_err(|e| DomainError::validation("user_access", e.to_string()))?;

        // Check HTTP status
        if !response.status().is_success() {
            return Err(DomainError::validation(
                "user_access",
                format!("HTTP {}", response.status()),
            ));
        }

        Ok(())
    }

    #[instrument(
        name = "users_info.http.notifications.user_created",
        skip_all,
        fields(notify_base = %self.notify_base)
    )]
    async fn notify_user_created(&self) -> Result<(), DomainError> {
        let mut url = self.notify_base.clone();
        url.path_segments_mut()
            .map_err(|_| {
                DomainError::validation("notifications", "invalid notifications base URL")
            })?
            .extend(&["api", "user-created"]);

        let response = self
            .client
            .post(url.as_str())
            .await
            .with_context(|| "POST /api/user-created")
            .map_err(|e| DomainError::validation("notifications", e.to_string()))?;

        // Check HTTP status
        if !response.status().is_success() {
            return Err(DomainError::validation(
                "notifications",
                format!("HTTP {}", response.status()),
            ));
        }

        Ok(())
    }
}
