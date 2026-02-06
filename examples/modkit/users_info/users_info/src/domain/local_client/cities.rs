use std::pin::Pin;
use std::sync::Arc;

use futures_util::{Stream, StreamExt};
use modkit_macros::domain_model;
use modkit_sdk::odata::{QueryBuilder, items_stream_boxed};
use modkit_security::SecurityContext;
use user_info_sdk::odata::CitySchema;
use user_info_sdk::{CitiesStreamingClientV1, City, UsersInfoError};

use crate::module::ConcreteAppServices;

#[domain_model]
pub(crate) struct LocalCitiesStreamingClient {
    services: Arc<ConcreteAppServices>,
}

impl LocalCitiesStreamingClient {
    #[must_use]
    pub fn new(services: Arc<ConcreteAppServices>) -> Self {
        Self { services }
    }
}

impl CitiesStreamingClientV1 for LocalCitiesStreamingClient {
    fn stream(
        &self,
        ctx: SecurityContext,
        query: QueryBuilder<CitySchema>,
    ) -> Pin<Box<dyn Stream<Item = Result<City, UsersInfoError>> + Send + 'static>> {
        let services = Arc::clone(&self.services);
        let stream = items_stream_boxed(
            query,
            Box::new(move |q| {
                let services = Arc::clone(&services);
                let ctx = ctx.clone();
                Box::pin(async move {
                    services
                        .cities
                        .list_cities_page(&ctx, &q)
                        .await
                        .map_err(UsersInfoError::from)
                })
            }),
        );
        Box::pin(stream.map(|res| res.map_err(|err| UsersInfoError::streaming(err.to_string()))))
    }
}
