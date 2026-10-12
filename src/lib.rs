pub mod api;
pub mod attachments;
pub mod auth;
pub mod code;
pub mod config;
pub mod error;
pub mod models;
pub mod ones;
pub mod work_hours;
pub mod workflow;

#[cfg(feature = "http")]
pub mod http_api;
#[cfg(feature = "http")]
pub(crate) mod http_work_hours;
#[cfg(feature = "http")]
pub(crate) mod http_workflow;
#[cfg(feature = "http")]
pub mod openapi;

#[cfg(feature = "mcp")]
pub mod mcp;

pub use config::{AppConfig, StdioAuthConfig};
pub use error::{AppError, AppResult};
pub use ones::service::OnesService;

#[cfg(feature = "http")]
pub use http_api::build_http_app;
#[cfg(feature = "mcp-http")]
pub use mcp::build_mcp_service;
#[cfg(feature = "mcp")]
pub use mcp::{McpAuthMode, OnesMcpServer};
#[cfg(feature = "http")]
pub use openapi::document as openapi_document;
