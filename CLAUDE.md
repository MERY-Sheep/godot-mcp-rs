# Godot MCP Server (godot-mcp-rs) Guide

This project is a Godot 4 Assistant powered by a Rust-based MCP server.
**It uses GraphQL (GQL) as the primary interface for all operations.**

## Core Capabilities

- **Project Structure**: Read scenes (.tscn), scripts (.gd), and resources (.tres) via structured queries.
- **Context Gathering**: Use `gatherContext` to pull related files and dependencies in one shot.
- **Safe Mutations**: Validate -> Preview -> Apply flow for modifying scenes and files.
- **Live Integration**: manipulate the running Godot Editor (add nodes, connect signals, etc.).

## 🛠 Tools (The "Big 3")

You only need these three tools. Forget about file system calls for Godot assets.

1.  **`godot_query`**: Read-only operations (Project info, Scene structure, Script analysis).
2.  **`godot_mutate`**: Write operations (Create/Edit Scene, Add Node, Live commands).
3.  **`godot_introspect`**: Get the schema or type details (Use this if you are unsure about args).

## 💡 Quick Start / Common Patterns

### 1. Understand the Project (Context)

Instead of `ls -R`, use:

```graphql
query {
  project {
    name
    path
    stats {
      sceneCount
      scriptCount
    }
  }
}
```

To read a specific file and its dependencies (The **BEST** way to start a task):

```graphql
query {
  gatherContext(input: { entryPoint: "res://scenes/main.tscn", depth: 1 }) {
    main {
      path
      script {
        path
      }
    }
    dependencies {
      path
      type
    }
    summary {
      totalFiles
    }
  }
}
```

### 2. Read a Scene or Script

```graphql
query {
  scene(path: "res://player.tscn") {
    root {
      name
      type
      children {
        name
        type
      }
    }
  }
  script(path: "res://player.gd") {
    className
    functions {
      name
      arguments
    }
  }
}
```

### 3. Make Changes (The Safe Way)

For complex changes, use the Transaction Flow: `validateMutation` -> `previewMutation` -> `applyMutation`.

**Example: Add a Timer node**

```graphql
mutation {
  applyMutation(input: {
    operations: [
      {
        type: ADD_NODE,
        args: {
            "parent": ".",
            "name": "AttackTimer",
            "type": "Timer"
        }
      }
    ]
  }) {
    success
    appliedCount
  }
}
```

## ⚠️ Important Rules

1.  **Single Source of Truth**: The schema is at `docs/gql/schema.graphql`. If you HALLUCINATE a field, the query will fail. Check schema if stuck.
2.  **Live vs File**:
    - `currentScene`, `node(path: ...)` are **Live** (Editor must be open).
    - `scene(path: ...)` is **File-based** (Static analysis, works anytime).
3.  **Arguments**: Most mutations take a `JSON` blob for `args`. This is flexible but requires you to match the expected keys (see `docs/DESIGN_GQL.md` or schema comments).
4.  **Do not write .tscn files manually**: Use `godot_mutate`. The server handles UID generation and format correctness.

## Development

- **Build**: `cargo build`
- **Test**: `cargo test`
- **Schema**: `docs/gql/schema.graphql` (Update this if you change `src/graphql/schema.rs`)
