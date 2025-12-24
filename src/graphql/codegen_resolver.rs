//! Code Generation Resolver
//!
//! Handles code generation for input handlers, state machines, and test scripts.

use std::fs;

use crate::godot::gdscript::GDScript;

use super::context::GqlContext;
use super::script_resolver::res_path_to_fs_path;
use super::types::*;

/// Generate input handler code
pub fn resolve_generate_input_handler(
    ctx: &GqlContext,
    input: &GenerateInputHandlerInput,
) -> CodeGenerationResult {
    let file_path = res_path_to_fs_path(&ctx.project_path, &input.script_path);

    // Read existing script or create new one
    let existing_content = fs::read_to_string(&file_path).unwrap_or_default();
    let has_content = !existing_content.trim().is_empty();

    // Determine handler function name
    let handler_func = match input.handler_type {
        Some(InputHandlerType::Process) | Option::None => "_process",
        Some(InputHandlerType::PhysicsProcess) => "_physics_process",
        Some(InputHandlerType::UnhandledInput) => "_unhandled_input",
        Some(InputHandlerType::Input) => "_input",
    };

    // Check if function already exists
    let pattern = format!(r"\bfunc\s+{}\b", handler_func);
    let regex = regex::Regex::new(&pattern).unwrap();
    if regex.is_match(&existing_content) {
        return CodeGenerationResult {
            success: false,
            path: input.script_path.clone(),
            message: Some(format!(
                "Function {} already exists in {}",
                handler_func, input.script_path
            )),
        };
    }

    // Generate input handling code
    let mut handler_body = String::new();
    for action in &input.actions {
        handler_body.push_str(&format!(
            r#"
	if Input.is_action_just_pressed("{}"):
		pass # TODO: Handle {} pressed
	if Input.is_action_just_released("{}"):
		pass # TODO: Handle {} released
"#,
            action, action, action, action
        ));
    }

    // Build function
    let delta_param = if handler_func == "_process" || handler_func == "_physics_process" {
        "delta: float"
    } else {
        "event: InputEvent"
    };

    let new_function = format!(
        "\n\nfunc {}({}) -> void:{}",
        handler_func, delta_param, handler_body
    );

    // Append to file
    let new_content = if has_content {
        format!("{}{}", existing_content.trim_end(), new_function)
    } else {
        format!("extends Node\n{}", new_function)
    };

    if let Err(e) = fs::write(&file_path, &new_content) {
        return CodeGenerationResult {
            success: false,
            path: input.script_path.clone(),
            message: Some(format!("Failed to write: {}", e)),
        };
    }

    CodeGenerationResult {
        success: true,
        path: input.script_path.clone(),
        message: Some(format!(
            "Generated {} handler for {} actions",
            handler_func,
            input.actions.len()
        )),
    }
}

/// Generate state machine boilerplate
pub fn resolve_generate_state_machine(
    ctx: &GqlContext,
    input: &GenerateStateMachineInput,
) -> CodeGenerationResult {
    let file_path = res_path_to_fs_path(&ctx.project_path, &input.script_path);

    let initial_state = input.initial_state.clone().unwrap_or_else(|| {
        input
            .states
            .first()
            .cloned()
            .unwrap_or_else(|| "IDLE".to_string())
    });

    let use_enum = input.use_enum.unwrap_or(true);

    let mut content = String::from("extends Node\n\n");

    // Generate enum if requested
    if use_enum {
        content.push_str("enum State {\n");
        for state in &input.states {
            content.push_str(&format!("\t{},\n", state.to_uppercase()));
        }
        content.push_str("}\n\n");

        content.push_str(&format!(
            "var current_state: State = State.{}\n\n",
            initial_state.to_uppercase()
        ));
    } else {
        content.push_str(&format!(
            "var current_state: String = \"{}\"\n\n",
            initial_state
        ));
    }

    // Generate state machine logic
    content.push_str("func _ready() -> void:\n");
    content.push_str(&format!(
        "\t_enter_state({})\n\n",
        if use_enum {
            format!("State.{}", initial_state.to_uppercase())
        } else {
            format!("\"{}\"", initial_state)
        }
    ));

    content.push_str("func _process(delta: float) -> void:\n");
    content.push_str("\t_state_process(delta)\n\n");

    content.push_str("func _physics_process(delta: float) -> void:\n");
    content.push_str("\t_state_physics_process(delta)\n\n");

    // Change state function
    let state_type = if use_enum { "State" } else { "String" };
    content.push_str(&format!(
        "func change_state(new_state: {}) -> void:\n",
        state_type
    ));
    content.push_str("\tif new_state == current_state:\n");
    content.push_str("\t\treturn\n");
    content.push_str("\t_exit_state(current_state)\n");
    content.push_str("\tcurrent_state = new_state\n");
    content.push_str("\t_enter_state(new_state)\n\n");

    // Enter state
    content.push_str(&format!(
        "func _enter_state(state: {}) -> void:\n",
        state_type
    ));
    content.push_str("\tmatch state:\n");
    for state in &input.states {
        if use_enum {
            content.push_str(&format!("\t\tState.{}:\n", state.to_uppercase()));
        } else {
            content.push_str(&format!("\t\t\"{}\":\n", state));
        }
        content.push_str(&format!("\t\t\t_enter_{}()\n", state.to_lowercase()));
    }
    content.push('\n');

    // Exit state
    content.push_str(&format!(
        "func _exit_state(state: {}) -> void:\n",
        state_type
    ));
    content.push_str("\tmatch state:\n");
    for state in &input.states {
        if use_enum {
            content.push_str(&format!("\t\tState.{}:\n", state.to_uppercase()));
        } else {
            content.push_str(&format!("\t\t\"{}\":\n", state));
        }
        content.push_str(&format!("\t\t\t_exit_{}()\n", state.to_lowercase()));
    }
    content.push('\n');

    // State process
    content.push_str("func _state_process(delta: float) -> void:\n");
    content.push_str("\tmatch current_state:\n");
    for state in &input.states {
        if use_enum {
            content.push_str(&format!("\t\tState.{}:\n", state.to_uppercase()));
        } else {
            content.push_str(&format!("\t\t\"{}\":\n", state));
        }
        content.push_str(&format!("\t\t\t_process_{}(delta)\n", state.to_lowercase()));
    }
    content.push('\n');

    // State physics process
    content.push_str("func _state_physics_process(delta: float) -> void:\n");
    content.push_str("\tmatch current_state:\n");
    for state in &input.states {
        if use_enum {
            content.push_str(&format!("\t\tState.{}:\n", state.to_uppercase()));
        } else {
            content.push_str(&format!("\t\t\"{}\":\n", state));
        }
        content.push_str(&format!(
            "\t\t\t_physics_process_{}(delta)\n",
            state.to_lowercase()
        ));
    }
    content.push('\n');

    // Generate stub functions for each state
    content.push_str("# State implementations\n\n");
    for state in &input.states {
        let lower = state.to_lowercase();
        content.push_str(&format!("func _enter_{}() -> void:\n\tpass\n\n", lower));
        content.push_str(&format!("func _exit_{}() -> void:\n\tpass\n\n", lower));
        content.push_str(&format!(
            "func _process_{}(delta: float) -> void:\n\tpass\n\n",
            lower
        ));
        content.push_str(&format!(
            "func _physics_process_{}(delta: float) -> void:\n\tpass\n\n",
            lower
        ));
    }

    // Ensure directory exists
    if let Some(parent) = file_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Err(e) = fs::write(&file_path, &content) {
        return CodeGenerationResult {
            success: false,
            path: input.script_path.clone(),
            message: Some(format!("Failed to write: {}", e)),
        };
    }

    CodeGenerationResult {
        success: true,
        path: input.script_path.clone(),
        message: Some(format!(
            "Generated state machine with {} states",
            input.states.len()
        )),
    }
}

/// Generate test script from target script
pub fn resolve_generate_test_script(
    ctx: &GqlContext,
    input: &GenerateTestScriptInput,
) -> CodeGenerationResult {
    let target_path = res_path_to_fs_path(&ctx.project_path, &input.target_script);

    // Parse target script
    let content = match fs::read_to_string(&target_path) {
        Ok(c) => c,
        Err(e) => {
            return CodeGenerationResult {
                success: false,
                path: input.target_script.clone(),
                message: Some(format!("Failed to read target script: {}", e)),
            };
        }
    };

    let script = GDScript::parse(&content);

    // Determine output path
    let output_path = input.output_path.clone().unwrap_or_else(|| {
        let target_name = target_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("script");
        format!("res://tests/test_{}.gd", target_name)
    });

    let test_file_path = res_path_to_fs_path(&ctx.project_path, &output_path);

    // Generate test script based on framework
    let test_content = match input.test_framework {
        Some(TestFramework::Gut) => generate_gut_test(&script, &input.target_script),
        Some(TestFramework::Custom) => generate_basic_test(&script, &input.target_script),
        Some(TestFramework::GdUnit4) | Option::None => {
            generate_gdunit4_test(&script, &input.target_script)
        }
    };

    // Ensure directory exists
    if let Some(parent) = test_file_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Err(e) = fs::write(&test_file_path, &test_content) {
        return CodeGenerationResult {
            success: false,
            path: output_path,
            message: Some(format!("Failed to write test: {}", e)),
        };
    }

    CodeGenerationResult {
        success: true,
        path: output_path,
        message: Some(format!(
            "Generated test script with {} test cases",
            script.functions.len()
        )),
    }
}

fn generate_gdunit4_test(script: &GDScript, target_path: &str) -> String {
    let mut content = format!(
        r#"extends GdUnitTestSuite

# Target script under test
const TargetScript = preload("{}")

var _instance: Node

func before_test() -> void:
	_instance = auto_free(TargetScript.new())

func after_test() -> void:
	_instance = null

"#,
        target_path
    );

    // Generate test for each function
    for func in &script.functions {
        if func.name.starts_with('_') && func.name != "_ready" && func.name != "_init" {
            continue; // Skip private functions except _ready/_init
        }

        content.push_str(&format!(
            r#"func test_{}() -> void:
	# TODO: Implement test for {}
	assert_that(_instance).is_not_null()
	# assert_that(_instance.{}(...)).is_equal(expected)

"#,
            func.name, func.name, func.name
        ));
    }

    content
}

fn generate_gut_test(script: &GDScript, target_path: &str) -> String {
    let mut content = format!(
        r#"extends GutTest

const TargetScript = preload("{}")
var _instance: Node

func before_each() -> void:
	_instance = TargetScript.new()
	add_child_autofree(_instance)

func after_each() -> void:
	_instance = null

"#,
        target_path
    );

    for func in &script.functions {
        if func.name.starts_with('_') && func.name != "_ready" && func.name != "_init" {
            continue;
        }

        content.push_str(&format!(
            r#"func test_{}() -> void:
	# TODO: Implement test for {}
	assert_not_null(_instance)
	# var result = _instance.{}(...)
	# assert_eq(result, expected)

"#,
            func.name, func.name, func.name
        ));
    }

    content
}

fn generate_basic_test(script: &GDScript, target_path: &str) -> String {
    let mut content = format!(
        r#"extends Node

# Basic test script for {}

const TargetScript = preload("{}")
var _instance: Node
var _passed := 0
var _failed := 0

func _ready() -> void:
	_run_tests()
	print("Tests completed: %d passed, %d failed" % [_passed, _failed])

func _run_tests() -> void:
"#,
        target_path, target_path
    );

    for func in &script.functions {
        if func.name.starts_with('_') && func.name != "_ready" && func.name != "_init" {
            continue;
        }
        content.push_str(&format!("\t_test_{}()\n", func.name));
    }

    content.push('\n');

    for func in &script.functions {
        if func.name.starts_with('_') && func.name != "_ready" && func.name != "_init" {
            continue;
        }

        content.push_str(&format!(
            r#"func _test_{}() -> void:
	_instance = TargetScript.new()
	add_child(_instance)
	# TODO: Add assertions for {}
	if true: # Replace with actual assertion
		_passed += 1
		print("PASS: {}")
	else:
		_failed += 1
		print("FAIL: {}")
	_instance.queue_free()

"#,
            func.name, func.name, func.name, func.name
        ));
    }

    content
}
