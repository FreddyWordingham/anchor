//! `anchor` is a library for declaratively managing Docker containers and clusters.

#![deny(absolute_paths_not_starting_with_crate)]
#![deny(ambiguous_negative_literals)]
#![deny(dead_code)]
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
#![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]
#![deny(unused_import_braces)]
#![deny(unused_imports)]
#![deny(unused_lifetimes)]
#![deny(unused_macro_rules)]
#![deny(unused_must_use)]
#![deny(unused_mut)]
#![deny(unused_qualifications)]
#![deny(unused_results)]
#![deny(unused_variables)]
#![deny(unused)]
#![deny(variant_size_differences)]
#![deny(warnings)]
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

mod cluster;
mod cluster_status;
mod command;
mod configuration;
mod container;
mod container_state;
mod docker_client;
mod docker_error;
mod launch_docker;
mod manifest;
mod manifest_error;

#[cfg(feature = "aws_ecr")]
mod credentials;

/// Re-export the main types and traits for easy access
pub mod prelude {
    pub use crate::{
        cluster::Cluster, cluster_status::ClusterStatus, command::Command, configuration::Configuration, container::Container,
        docker_client::DockerClient, docker_error::DockerError, launch_docker::start_docker_daemon, manifest::Manifest,
        manifest_error::ManifestError,
    };

    #[cfg(feature = "aws_ecr")]
    pub use crate::credentials::*;
}
