//! GraphQL Resolvers - Facade Module
//!
//! This module re-exports all resolver functions from domain-specific modules.
//! The implementation has been decomposed into:
//! - project_resolver: Project information, file collection, validation
//! - scene_resolver: Scene parsing, conversion, creation
//! - script_resolver: Script parsing, conversion, creation
//! - mutation_resolver: Mutation validation, preview, application
//! - node_type_resolver: Node type information from static database
//! - test_resolver: GdUnit4 test execution
//! - refactoring_resolver: Code understanding, refactoring operations
//! - codegen_resolver: Code generation (input handlers, state machines, tests)
//! - shader_resolver: Shader validation

// Allow unused imports in this facade module - these are re-exported for external use
#![allow(unused_imports)]

// Re-export all public APIs from domain-specific resolvers

// Project operations
pub use super::project_resolver::{
    collect_project_files, count_resources, parse_project_name, resolve_project, to_res_path,
    validate_project,
};

// Scene operations
pub use super::scene_resolver::{convert_godot_scene_to_gql, create_scene, resolve_scene};

// Script operations
pub use super::script_resolver::{
    convert_gdscript_to_gql, create_script, parse_signal_definition, res_path_to_fs_path,
    resolve_script,
};

// Mutation operations
pub use super::mutation_resolver::{apply_mutation, preview_mutation, validate_mutation};

// Node type info
pub use super::node_type_resolver::resolve_node_type_info;

// Test execution
pub use super::test_resolver::{parse_test_output, resolve_run_tests};

// Refactoring operations
pub use super::refactoring_resolver::{
    resolve_autoloads, resolve_class_hierarchy, resolve_extract_function, resolve_find_references,
    resolve_rename_symbol,
};

// Code generation
pub use super::codegen_resolver::{
    resolve_generate_input_handler, resolve_generate_state_machine, resolve_generate_test_script,
};

// Shader validation
pub use super::shader_resolver::resolve_validate_shader;
