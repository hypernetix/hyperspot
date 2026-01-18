use std::pin::Pin;
use std::sync::Arc;

use futures_util::{Stream, StreamExt};
use modkit_sdk::odata::{items_stream_boxed, QueryBuilder};
use modkit_security::SecurityContext;
use user_info_sdk::odata::AddressSchema;
use user_info_sdk::{client::AddressesStreamingClient, Address, UsersInfoError};

use crate::module::ConcreteAppServices;

pub(crate) struct LocalAddressesStreamingClient {
    services: Arc<ConcreteAppServices>,
}

impl LocalAddressesStreamingClient {
    #[must_use]
    pub fn new(services: Arc<ConcreteAppServices>) -> Self {
        Self { services }
    }
}

impl AddressesStreamingClient for LocalAddressesStreamingClient {
    fn stream(
        &self,
        ctx: SecurityContext,
        query: QueryBuilder<AddressSchema>,
    ) -> Pin<Box<dyn Stream<Item = Result<Address, UsersInfoError>> + Send + 'static>> {
        let services = Arc::clone(&self.services);
        let stream = items_stream_boxed(
            query,
            Box::new(move |q| {
                let services = Arc::clone(&services);
                let ctx = ctx.clone();
                Box::pin(async move {
                    services
                        .addresses
                        .list_addresses_page(&ctx, &q)
                        .await
                        .map_err(UsersInfoError::from)
                })
            }),
        );
        Box::pin(stream.map(|res| res.map_err(|err| UsersInfoError::streaming(err.to_string()))))
    }
}
