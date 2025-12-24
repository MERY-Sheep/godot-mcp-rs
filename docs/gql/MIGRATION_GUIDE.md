# GQL 移行ガイド

既存の MCP ツールから GraphQL ツール（`godot_query`, `godot_mutate`, `godot_introspect`）への移行ガイドです。

## 重要

> [!IMPORTANT]
> 2025 年 12 月 24 日のアップデートより、従来の 56 個の個別 MCP ツールは **MCP サーバーから削除** されました。
> 現在は `godot_query`, `godot_mutate`, `godot_introspect` の 3 つの GraphQL ツールが推奨される唯二のインターフェースです。
> CLI 版では後方互換性のために引き続き旧コマンドを使用できます。

## ツール対応表

### Read 系（Query）

| 旧ツール            | GraphQL Query                                                             |
| ------------------- | ------------------------------------------------------------------------- |
| `list_all_scenes`   | `{ project { scenes { path } } }`                                         |
| `get_project_stats` | `{ project { stats { sceneCount scriptCount } } }`                        |
| `read_scene`        | `{ scene(path: "res://...") { root { name type children { name } } } }`   |
| `read_script`       | `{ script(path: "res://...") { functions { name } variables { name } } }` |

### Write 系（Mutation）

| 旧ツール              | GraphQL Mutation                                                                                     |
| --------------------- | ---------------------------------------------------------------------------------------------------- |
| `live_add_node`       | `addNode(input: { parent: "...", name: "...", type: "..." }) { success }`                            |
| `live_set_property`   | `setProperty(input: { nodePath: "...", property: "...", value: "..." }) { success }`                 |
| `live_connect_signal` | `connectSignal(input: { fromNode: "...", signal: "...", toNode: "...", method: "..." }) { success }` |

### デバッグ・ログ系 (Phase 2)

| 旧ツール                   | GraphQL                                                                        |
| -------------------------- | ------------------------------------------------------------------------------ |
| `live_get_editor_log`      | `query { logs(limit: 50) { message severity } }`                               |
| `live_get_debugger_errors` | `query { debuggerErrors { message stackInfo { file line } } }`                 |
| `live_pause`               | `mutation { pause { success } }`                                               |
| `live_resume`              | `mutation { resume { success } }`                                              |
| `live_step`                | `mutation { step { success } }`                                                |
| なし (新規)                | `query { objectById(objectId: "...") { id class properties { name value } } }` |

### 開発・テスト系

| 旧ツール          | GraphQL Mutation                                            |
| ----------------- | ----------------------------------------------------------- |
| `run_project`相当 | `runTests(input: { testPath: "res://tests/" }) { success }` |

### バッチ操作

複数の変更をまとめて検証・適用：

```graphql
mutation {
  validateMutation(
    input: {
      operations: [
        {
          type: ADD_NODE
          args: { parent: "/root", name: "Enemy", type: "CharacterBody3D" }
        }
        {
          type: SET_PROPERTY
          args: { nodePath: "/root/Enemy", property: "visible", value: "true" }
        }
      ]
    }
  ) {
    isValid
    errors {
      code
      message
    }
  }
}
```

## 使い方

### godot_query

```json
{
  "query": "{ project { name scenes { path } } }"
}
```

### godot_mutate

```json
{
  "mutation": "mutation { addNode(input: { parent: \".\", name: \"Test\", type: \"Node3D\" }) { success } }"
}
```

### godot_introspect

```json
{
  "format": "SDL"
}
```

## メリット

- **型安全**: GraphQL スキーマによる事前検証
- **柔軟性**: 必要なフィールドだけを取得
- **一貫性**: 統一されたエラー形式
- **発見可能性**: `godot_introspect` でスキーマを確認可能
