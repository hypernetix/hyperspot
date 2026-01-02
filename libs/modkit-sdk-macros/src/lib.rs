#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod odata_schema;

#[proc_macro_derive(ODataSchema, attributes(odata))]
#[proc_macro_error]
pub fn derive_odata_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    odata_schema::expand_derive_odata_schema(&input).into()
}
