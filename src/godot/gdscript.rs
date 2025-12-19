//! GDScript Parser & Generator
//!
//! Parsing GDScript files and generating templates

/// GDScript File Structure
#[derive(Debug, Clone)]
pub struct GDScript {
    /// extends declaration
    pub extends: Option<String>,
    /// class_name declaration
    pub class_name: Option<String>,
    /// Export variables
    pub exports: Vec<ExportVar>,
    /// Normal variables
    pub variables: Vec<Variable>,
    /// Functions
    pub functions: Vec<Function>,
    /// Signals
    pub signals: Vec<String>,
}

/// Export Variable
#[derive(Debug, Clone)]
pub struct ExportVar {
    pub name: String,
    pub var_type: Option<String>,
    pub default_value: Option<String>,
}

/// Variable
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub var_type: Option<String>,
    pub default_value: Option<String>,
}

/// Function
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub return_type: Option<String>,
    pub body: String,
}

/// Function Parameter
#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub param_type: Option<String>,
    pub default_value: Option<String>,
}

impl GDScript {
    /// Create new script
    pub fn new(extends: &str) -> Self {
        Self {
            extends: Some(extends.to_string()),
            class_name: None,
            exports: Vec::new(),
            variables: Vec::new(),
            functions: Vec::new(),
            signals: Vec::new(),
        }
    }

    /// Parse GDScript
    pub fn parse(content: &str) -> Self {
        let mut script = GDScript {
            extends: None,
            class_name: None,
            exports: Vec::new(),
            variables: Vec::new(),
            functions: Vec::new(),
            signals: Vec::new(),
        };

        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // extends
            if line.starts_with("extends ") {
                script.extends = Some(line[8..].trim().to_string());
            }
            // class_name
            else if line.starts_with("class_name ") {
                script.class_name = Some(line[11..].trim().to_string());
            }
            // signal
            else if line.starts_with("signal ") {
                script.signals.push(line[7..].trim().to_string());
            }
            // @export var
            else if line.starts_with("@export") && line.contains("var ") {
                if let Some(var) = parse_export_var(line) {
                    script.exports.push(var);
                }
            }
            // var
            else if line.starts_with("var ") {
                if let Some(var) = parse_var(line) {
                    script.variables.push(var);
                }
            }
            // func
            else if line.starts_with("func ") {
                let (func, consumed) = parse_function(&lines, i);
                if let Some(f) = func {
                    script.functions.push(f);
                }
                i += consumed.max(1) - 1;
            }

            i += 1;
        }

        script
    }

    /// Generate GDScript code
    pub fn to_gdscript(&self) -> String {
        let mut output = String::new();

        // extends
        if let Some(ref ext) = self.extends {
            output.push_str(&format!("extends {}\n", ext));
        }

        // class_name
        if let Some(ref name) = self.class_name {
            output.push_str(&format!("class_name {}\n", name));
        }

        if self.extends.is_some() || self.class_name.is_some() {
            output.push('\n');
        }

        // signals
        for signal in &self.signals {
            output.push_str(&format!("signal {}\n", signal));
        }
        if !self.signals.is_empty() {
            output.push('\n');
        }

        // exports
        for var in &self.exports {
            let type_hint = var
                .var_type
                .as_ref()
                .map(|t| format!(": {}", t))
                .unwrap_or_default();
            let default = var
                .default_value
                .as_ref()
                .map(|v| format!(" = {}", v))
                .unwrap_or_default();
            output.push_str(&format!(
                "@export var {}{}{}\n",
                var.name, type_hint, default
            ));
        }
        if !self.exports.is_empty() {
            output.push('\n');
        }

        // variables
        for var in &self.variables {
            let type_hint = var
                .var_type
                .as_ref()
                .map(|t| format!(": {}", t))
                .unwrap_or_default();
            let default = var
                .default_value
                .as_ref()
                .map(|v| format!(" = {}", v))
                .unwrap_or_default();
            output.push_str(&format!("var {}{}{}\n", var.name, type_hint, default));
        }
        if !self.variables.is_empty() {
            output.push('\n');
        }

        // functions
        for func in &self.functions {
            let params: Vec<String> = func
                .params
                .iter()
                .map(|p| {
                    let type_hint = p
                        .param_type
                        .as_ref()
                        .map(|t| format!(": {}", t))
                        .unwrap_or_default();
                    let default = p
                        .default_value
                        .as_ref()
                        .map(|v| format!(" = {}", v))
                        .unwrap_or_default();
                    format!("{}{}{}", p.name, type_hint, default)
                })
                .collect();

            let return_type = func
                .return_type
                .as_ref()
                .map(|t| format!(" -> {}", t))
                .unwrap_or_default();

            output.push_str(&format!(
                "func {}({}){}:\n",
                func.name,
                params.join(", "),
                return_type
            ));

            for line in func.body.lines() {
                output.push_str(&format!("\t{}\n", line));
            }
            output.push('\n');
        }

        output
    }

    /// Add function
    pub fn add_function(&mut self, func: Function) {
        self.functions.push(func);
    }

    /// Add variable
    pub fn add_variable(&mut self, var: Variable) {
        self.variables.push(var);
    }

    /// Add export variable
    pub fn add_export(&mut self, var: ExportVar) {
        self.exports.push(var);
    }
}

/// Parse @export var
fn parse_export_var(line: &str) -> Option<ExportVar> {
    let var_start = line.find("var ")? + 4;
    let rest = &line[var_start..];

    let (name, rest) = if let Some(colon) = rest.find(':') {
        (&rest[..colon], Some(&rest[colon + 1..]))
    } else if let Some(eq) = rest.find('=') {
        (&rest[..eq], Some(&rest[eq..]))
    } else {
        (rest.trim(), None)
    };

    let (var_type, default_value) = if let Some(rest) = rest {
        if let Some(eq) = rest.find('=') {
            let type_part = rest[..eq].trim();
            let value_part = rest[eq + 1..].trim();
            (
                if type_part.is_empty() {
                    None
                } else {
                    Some(type_part.to_string())
                },
                Some(value_part.to_string()),
            )
        } else {
            (Some(rest.trim().to_string()), None)
        }
    } else {
        (None, None)
    };

    Some(ExportVar {
        name: name.trim().to_string(),
        var_type,
        default_value,
    })
}

/// Parse var
fn parse_var(line: &str) -> Option<Variable> {
    let rest = &line[4..];

    let (name, rest) = if let Some(colon) = rest.find(':') {
        (&rest[..colon], Some(&rest[colon + 1..]))
    } else if let Some(eq) = rest.find('=') {
        (&rest[..eq], Some(&rest[eq..]))
    } else {
        (rest.trim(), None)
    };

    let (var_type, default_value) = if let Some(rest) = rest {
        if let Some(eq) = rest.find('=') {
            let type_part = rest[..eq].trim();
            let value_part = rest[eq + 1..].trim();
            (
                if type_part.is_empty() {
                    None
                } else {
                    Some(type_part.to_string())
                },
                Some(value_part.to_string()),
            )
        } else {
            (Some(rest.trim().to_string()), None)
        }
    } else {
        (None, None)
    };

    Some(Variable {
        name: name.trim().to_string(),
        var_type,
        default_value,
    })
}

/// Parse function
fn parse_function(lines: &[&str], start: usize) -> (Option<Function>, usize) {
    let line = lines[start].trim();

    // func name(params) -> type:
    let func_start = 5; // "func " length
    let paren_start = match line[func_start..].find('(') {
        Some(p) => func_start + p,
        None => return (None, 1),
    };
    let paren_end = match line.find(')') {
        Some(p) => p,
        None => return (None, 1),
    };

    let name = line[func_start..paren_start].trim().to_string();
    let params_str = &line[paren_start + 1..paren_end];

    // Parse parameters
    let params: Vec<FunctionParam> = if params_str.trim().is_empty() {
        Vec::new()
    } else {
        params_str
            .split(',')
            .map(|p| {
                let p = p.trim();
                let (name, rest) = if let Some(colon) = p.find(':') {
                    (&p[..colon], Some(&p[colon + 1..]))
                } else {
                    (p, None)
                };

                let (param_type, default_value) = if let Some(rest) = rest {
                    if let Some(eq) = rest.find('=') {
                        (
                            Some(rest[..eq].trim().to_string()),
                            Some(rest[eq + 1..].trim().to_string()),
                        )
                    } else {
                        (Some(rest.trim().to_string()), None)
                    }
                } else {
                    (None, None)
                };

                FunctionParam {
                    name: name.trim().to_string(),
                    param_type,
                    default_value,
                }
            })
            .collect()
    };

    // Return type
    let return_type = if let Some(arrow) = line.find("->") {
        let colon = line.rfind(':').unwrap_or(line.len());
        Some(line[arrow + 2..colon].trim().to_string())
    } else {
        None
    };

    // Collect function body
    let mut body_lines = Vec::new();
    let mut i = start + 1;
    while i < lines.len() {
        let l = lines[i];
        if l.is_empty() || l.starts_with('\t') || l.starts_with("    ") {
            let trimmed = l
                .strip_prefix('\t')
                .or_else(|| l.strip_prefix("    "))
                .unwrap_or(l);
            body_lines.push(trimmed);
            i += 1;
        } else if l.trim().is_empty() {
            body_lines.push("");
            i += 1;
        } else {
            break;
        }
    }

    let body = body_lines.join("\n").trim_end().to_string();

    (
        Some(Function {
            name,
            params,
            return_type,
            body,
        }),
        i - start,
    )
}

/// Generate script template
pub fn generate_template(extends: &str) -> String {
    match extends {
        "CharacterBody3D" => TEMPLATE_CHARACTER_BODY_3D.to_string(),
        "CharacterBody2D" => TEMPLATE_CHARACTER_BODY_2D.to_string(),
        "Node3D" => TEMPLATE_NODE_3D.to_string(),
        "Node2D" => TEMPLATE_NODE_2D.to_string(),
        "RigidBody3D" => TEMPLATE_RIGID_BODY_3D.to_string(),
        "Area3D" => TEMPLATE_AREA_3D.to_string(),
        _ => format!("extends {}\n\nfunc _ready() -> void:\n\tpass\n", extends),
    }
}

const TEMPLATE_CHARACTER_BODY_3D: &str = r#"extends CharacterBody3D

@export var speed: float = 5.0
@export var jump_velocity: float = 4.5

var gravity: float = ProjectSettings.get_setting("physics/3d/default_gravity")

func _physics_process(delta: float) -> void:
	if not is_on_floor():
		velocity.y -= gravity * delta

	if Input.is_action_just_pressed("jump") and is_on_floor():
		velocity.y = jump_velocity

	var input_dir := Input.get_vector("left", "right", "forward", "back")
	var direction := (transform.basis * Vector3(input_dir.x, 0, input_dir.y)).normalized()
	
	if direction:
		velocity.x = direction.x * speed
		velocity.z = direction.z * speed
	else:
		velocity.x = move_toward(velocity.x, 0, speed)
		velocity.z = move_toward(velocity.z, 0, speed)

	move_and_slide()
"#;

const TEMPLATE_CHARACTER_BODY_2D: &str = r#"extends CharacterBody2D

@export var speed: float = 300.0
@export var jump_velocity: float = -400.0

var gravity: float = ProjectSettings.get_setting("physics/2d/default_gravity")

func _physics_process(delta: float) -> void:
	if not is_on_floor():
		velocity.y += gravity * delta

	if Input.is_action_just_pressed("jump") and is_on_floor():
		velocity.y = jump_velocity

	var direction := Input.get_axis("left", "right")
	if direction:
		velocity.x = direction * speed
	else:
		velocity.x = move_toward(velocity.x, 0, speed)

	move_and_slide()
"#;

const TEMPLATE_NODE_3D: &str = r#"extends Node3D

func _ready() -> void:
	pass

func _process(delta: float) -> void:
	pass
"#;

const TEMPLATE_NODE_2D: &str = r#"extends Node2D

func _ready() -> void:
	pass

func _process(delta: float) -> void:
	pass
"#;

const TEMPLATE_RIGID_BODY_3D: &str = r#"extends RigidBody3D

func _ready() -> void:
	pass

func _physics_process(delta: float) -> void:
	pass
"#;

const TEMPLATE_AREA_3D: &str = r#"extends Area3D

func _ready() -> void:
	body_entered.connect(_on_body_entered)
	body_exited.connect(_on_body_exited)

func _on_body_entered(body: Node3D) -> void:
	pass

func _on_body_exited(body: Node3D) -> void:
	pass
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_script() {
        let content = r#"extends Node3D

var health: int = 100

func _ready() -> void:
	pass
"#;
        let script = GDScript::parse(content);
        assert_eq!(script.extends, Some("Node3D".to_string()));
        assert_eq!(script.variables.len(), 1);
        assert_eq!(script.functions.len(), 1);
    }

    #[test]
    fn test_generate_script() {
        let mut script = GDScript::new("Node3D");
        script.add_variable(Variable {
            name: "speed".to_string(),
            var_type: Some("float".to_string()),
            default_value: Some("5.0".to_string()),
        });

        let output = script.to_gdscript();
        assert!(output.contains("extends Node3D"));
        assert!(output.contains("var speed: float = 5.0"));
    }
}
