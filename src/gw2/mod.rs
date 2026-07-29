mod client;
mod endpoint;
mod path;
mod public;
mod request;

pub use client::Gw2Client;
pub use endpoint::EndpointSpec;
pub use path::normalize_api_path;
pub use public::{PublicEndpointKind, PublicEndpointSpec, public_endpoints};
pub use request::ApiRequest;

pub(crate) use path::validate_lang;
pub(crate) use public::{excluded_endpoint, public_endpoint};
