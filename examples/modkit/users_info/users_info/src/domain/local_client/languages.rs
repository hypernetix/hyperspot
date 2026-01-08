use std::pin::Pin;
use std::sync::Arc;

use futures_util::{Stream, StreamExt};
use modkit_sdk::odata::{items_stream_boxed, QueryBuilder};
use modkit_security::SecurityContext;
use user_info_sdk::odata::LanguageSchema;
use user_info_sdk::{api::LanguagesStreamingClient, Language, UsersInfoError};

use crate::domain::service::Service;

pub struct LocalLanguagesStreamingClient {
    service: Arc<Service>,
}

impl LocalLanguagesStreamingClient {
    #[must_use]
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

impl LanguagesStreamingClient for LocalLanguagesStreamingClient {
    fn stream(
        &self,
        ctx: SecurityContext,
        query: QueryBuilder<LanguageSchema>,
    ) -> Pin<Box<dyn Stream<Item = Result<Language, UsersInfoError>> + Send + 'static>> {
        let service = Arc::clone(&self.service);
        let stream = items_stream_boxed(
            query,
            Box::new(move |q| {
                let service = Arc::clone(&service);
                let ctx = ctx.clone();
                Box::pin(async move {
                    service
                        .list_languages_page(&ctx, &q)
                        .await
                        .map_err(UsersInfoError::from)
                })
            }),
        );
        Box::pin(stream.map(|res| res.map_err(|err| UsersInfoError::streaming(err.to_string()))))
    }
}
