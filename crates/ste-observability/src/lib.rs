//! Local-only, bounded, schema-redacted observability.
#![forbid(unsafe_code)]
pub mod diagnostics;
pub mod metrics;
pub mod support_bundle;
pub mod tracing;
pub use diagnostics::{HealthSnapshot, Record, RecordClass, RecordStore, RedactionSchema};
pub use metrics::{MetricError, MetricRegistry};
pub use support_bundle::{BundleError, BundleManifest, BundlePreview, SupportBundleBuilder};
pub use tracing::{LocalSpan, TraceStore};
