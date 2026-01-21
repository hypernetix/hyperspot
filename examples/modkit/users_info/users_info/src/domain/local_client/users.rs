use std::pin::Pin;
use std::sync::Arc;

use futures_util::{Stream, StreamExt};
use modkit_sdk::odata::{QueryBuilder, items_stream_boxed};
use modkit_security::SecurityContext;
use user_info_sdk::odata::UserSchema;
use user_info_sdk::{User, UsersInfoError, client::UsersStreamingClient};

use crate::module::ConcreteAppServices;

pub(crate) struct LocalUsersStreamingClient {
    services: Arc<ConcreteAppServices>,
}

impl LocalUsersStreamingClient {
    #[must_use]
    pub fn new(services: Arc<ConcreteAppServices>) -> Self {
        Self { services }
    }
}

impl UsersStreamingClient for LocalUsersStreamingClient {
    fn stream(
        &self,
        ctx: SecurityContext,
        query: QueryBuilder<UserSchema>,
    ) -> Pin<Box<dyn Stream<Item = Result<User, UsersInfoError>> + Send + 'static>> {
        let services = Arc::clone(&self.services);
        let stream = items_stream_boxed(
            query,
            Box::new(move |q| {
                let services = Arc::clone(&services);
                let ctx = ctx.clone();
                Box::pin(async move {
                    services
                        .users
                        .list_users_page(&ctx, &q)
                        .await
                        .map_err(UsersInfoError::from)
                })
            }),
        );
        Box::pin(stream.map(|res| res.map_err(|err| UsersInfoError::streaming(err.to_string()))))
    }
}
