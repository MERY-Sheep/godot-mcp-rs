//! GraphQL schema for Godot Query Language (GQL)
//!
//! Single source of truth: `docs/gql/schema.graphql`
//! This module implements the schema in Rust using async-graphql.

pub mod context;
pub mod dependency_resolver;
pub mod error;
pub mod live_resolver;

// Domain-specific resolvers (decomposed from monolithic resolver.rs)
mod codegen_resolver;
mod mutation_resolver;
mod node_type_resolver;
mod project_resolver;
mod refactoring_resolver;
mod scene_resolver;
mod script_resolver;
mod shader_resolver;
mod test_resolver;

// Facade module re-exporting all resolvers
mod resolver;
mod schema;
mod types;

pub use context::GqlContext;
pub use schema::{build_schema, build_schema_with_context, GqlSchema, MutationRoot, QueryRoot};
pub use types::*;
