//! OData error catalog - centralized error definitions for all OData operations
//!
//! This module provides GTS error codes from the OData error catalog that are used
//! to map modkit_odata::Error to RFC 9457 Problem responses.
use modkit_errors_macro::declare_errors;

declare_errors! {
    path = "gts/errors_odata.json",
    namespace = "odata_errors",
    vis = "pub"
}
