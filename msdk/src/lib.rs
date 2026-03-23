//! memlink SDK - Build modules for the memlink runtime.
//!
//! Provides core infrastructure for memlink modules including arena allocation,
//! method dispatch, request/response handling, serialization, logging, metrics,
//! panic isolation, and persistent state references.

#![allow(missing_docs)]
#![allow(clippy::missing_safety_doc)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod arena;
pub mod caller;
pub mod context;
pub mod dispatch;
pub mod error;
pub mod exports;
pub mod log;
pub mod macros;
pub mod metrics;
pub mod panic;
pub mod r#ref;
pub mod request;
pub mod serialize;

pub use arena::Arena;
pub use caller::ModuleCaller;
pub use context::CallContext;
pub use error::{ModuleError, Result};
pub use macros::{memlink_export, memlink_module};
pub use r#ref::ArenaRef;

pub mod prelude {
    pub use crate::arena::Arena;
    pub use crate::caller::{ModuleCaller, MAX_CALL_DEPTH};
    pub use crate::context::CallContext;
    pub use crate::dispatch::{dispatch, dispatch_with_context, register_method};
    pub use crate::error::{ModuleError, Result};
    pub use crate::log::{debug, error, info, log, Level};
    pub use crate::macros::{memlink_export, memlink_module};
    pub use crate::metrics::{increment_counter, observe_histogram, record_metric, set_gauge, MetricValue};
    pub use crate::panic::catch_module_panic;
    pub use crate::r#ref::ArenaRef;
    pub use crate::serialize::{default_serializer, MessagePackSerializer, Serializer};
    pub use memlink_protocol::{Request, Response, StatusCode};
}
