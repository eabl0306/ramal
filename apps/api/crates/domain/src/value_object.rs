//! Value objects compartidos por varios contextos.

mod component_name;
mod latency;
mod service_version;

pub use component_name::{ComponentName, ComponentNameError};
pub use latency::Latency;
pub use service_version::{ServiceVersion, ServiceVersionError};
