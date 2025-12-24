//! GraphQL schema for Godot Query Language (GQL)
//!
//! Single source of truth: `docs/gql/schema.graphql`
//! This module implements the schema in Rust using async-graphql.

pub mod context;
pub mod dependency_resolver;
pub mod live_resolver;
mod resolver;
mod schema;
mod types;

pub use context::GqlContext;
pub use schema::{build_schema, build_schema_with_context, GqlSchema, MutationRoot, QueryRoot};
pub use types::*;
