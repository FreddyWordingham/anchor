//! `anchor` is a library for declaratively managing Docker containers and clusters.

#![deny(absolute_paths_not_starting_with_crate)]
#![deny(ambiguous_negative_literals)]
// #![deny(dead_code)] ////////// Turn on before release
#![deny(deprecated_safe_2024)]
#![deny(deref_into_dyn_supertrait)]
#![deny(edition_2024_expr_fragment_specifier)]
// #![deny(elided_lifetimes_in_paths)]
#![deny(explicit_outlives_requirements)]
#![deny(ffi_unwind_calls)]
#![deny(future_incompatible)]
#![deny(if_let_rescope)]
#![deny(impl_trait_overcaptures)]
#![deny(impl_trait_redundant_captures)]
#![deny(improper_ctypes)]
#![deny(keyword_idents_2018)]
#![deny(keyword_idents_2024)]
#![deny(keyword_idents)]
#![deny(let_underscore_drop)]
#![deny(macro_use_extern_crate)]
#![deny(meta_variable_misuse)]
#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(missing_unsafe_on_extern)]
#![deny(non_ascii_idents)]
#![deny(nonstandard_style)]
#![deny(path_statements)]
#![deny(redundant_imports)]
#![deny(redundant_lifetimes)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(single_use_lifetimes)]
#![deny(tail_expr_drop_order)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unit_bindings)]
#![deny(unnameable_types)]
#![deny(unreachable_code)]
// #![deny(unreachable_pub)]
#![deny(unsafe_attr_outside_unsafe)]
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unstable_features)]
#![deny(unused_assignments)]
// #![deny(unused_crate_dependencies)] ////////// Turn on before release
// #![deny(unused_extern_crates)] ////////// Turn on before release
// #![deny(unused_import_braces)] ////////// Turn on before release
// #![deny(unused_imports)] ////////// Turn on before release
// #![deny(unused_lifetimes)] ////////// Turn on before release
// #![deny(unused_macro_rules)] ////////// Turn on before release
// #![deny(unused_must_use)] ////////// Turn on before release
// #![deny(unused_mut)] ////////// Turn on before release
// #![deny(unused_qualifications)] ////////// Turn on before release
// #![deny(unused_results)] ////////// Turn on before release
// #![deny(unused_variables)] ////////// Turn on before release
// #![deny(unused)] ////////// Turn on before release
#![deny(variant_size_differences)]
// #![deny(warnings)]
#![deny(clippy::all)]
#![deny(clippy::cargo)]
#![deny(clippy::complexity)]
#![deny(clippy::correctness)]
#![deny(clippy::nursery)]
#![deny(clippy::pedantic)]
#![deny(clippy::perf)]
// // #![deny(clippy::restriction)]
#![deny(clippy::style)]
#![deny(clippy::suspicious)]
#![expect(
    clippy::multiple_crate_versions,
    reason = "Multiple versions of some dependencies are used in the workspace, but they are compatible and do not cause issues."
)]

#[cfg(feature = "aws_ecr")]
mod credentials;

mod anchor_error;
mod client;
mod container_metrics;
mod format;
mod health_status;
mod mount_type;
mod resource_status;
mod start_docker_daemon;

/// Re-export the main types and traits for easy access
pub mod prelude {
    #[cfg(feature = "aws_ecr")]
    pub use crate::credentials::get_ecr_credentials;

    pub use crate::{
        anchor_error::{AnchorError, AnchorResult},
        client::Client,
        container_metrics::ContainerMetrics,
        health_status::HealthStatus,
        mount_type::MountType,
        resource_status::ResourceStatus,
        start_docker_daemon::start_docker_daemon,
    };
}
