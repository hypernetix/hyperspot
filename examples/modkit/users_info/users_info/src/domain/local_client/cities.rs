use std::pin::Pin;
use std::sync::Arc;

use futures_util::{Stream, StreamExt};
use modkit_sdk::odata::{items_stream_boxed, QueryBuilder};
use modkit_security::SecurityContext;
use user_info_sdk::odata::CitySchema;
use user_info_sdk::{api::CitiesStreamingClient, City, UsersInfoError};

use crate::domain::service::Service;

pub struct LocalCitiesStreamingClient {
    service: Arc<Service>,
}

impl LocalCitiesStreamingClient {
    #[must_use]
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

impl CitiesStreamingClient for LocalCitiesStreamingClient {
    fn stream(
        &self,
        ctx: SecurityContext,
        query: QueryBuilder<CitySchema>,
    ) -> Pin<Box<dyn Stream<Item = Result<City, UsersInfoError>> + Send + 'static>> {
        let service = Arc::clone(&self.service);
        let stream = items_stream_boxed(
            query,
            Box::new(move |q| {
                let service = Arc::clone(&service);
                let ctx = ctx.clone();
                Box::pin(async move {
                    service
                        .list_cities_page(&ctx, &q)
                        .await
                        .map_err(UsersInfoError::from)
                })
            }),
        );
        Box::pin(stream.map(|res| res.map_err(|err| UsersInfoError::streaming(err.to_string()))))
    }
}
