## Claude Code / LLM Agent ガイド（GQL 中心）

このリポジトリは Godot プロジェクトを GraphQL (GQL) 経由で操作するための MCP サーバーです。  
**迷ったらまずここを読み、指示に従ってください。**

---

## 開発の要点（Source of Truth）

- **スキーマ (定数)**: [docs/gql/schema.graphql](file:///c:/Work/godot-mcp-rs/docs/gql/schema.graphql)
  - 全機能の定義、型、エラーコードの唯一の正典です。
- **設計契約**: [docs/DESIGN_GQL.md](file:///c:/Work/godot-mcp-rs/docs/DESIGN_GQL.md)
- **移行ガイド**: [docs/gql/MIGRATION_GUIDE.md](file:///c:/Work/godot-mcp-rs/docs/gql/MIGRATION_GUIDE.md)

---

## MCP ツール（GQL 一本化）

従来の個別ツール（56 個以上）は廃止され、以下の **3 つの GQL ツール** に集約されました。

1. `godot_query`: データの読み取り（シーン、スクリプト、プロジェクト統計等）
2. `godot_mutate`: データの変更（ノード追加、プロパティ設定、ファイル作成等）
3. `godot_introspect`: スキーマ（SDL）の取得

LLM は、`godot_introspect` で利用可能なクエリ/ミューテーションを調べ、`godot_query`/`godot_mutate` を実行してください。

---

## 基本構造

- `src/graphql/`: GQL エンジン、リゾルバ、型定義。
- `src/tools/`: MCP ハンドラ、旧ツール群（CLI 互換用）。
- `src/godot/`: Godot ファイルの静的解析（パーサー）。
- `addons/`: Godot 側にインストールする MCP 連携プラグイン。

---

## 作業ルール

- **TDD (最重要)**: 変更時は必ず `tests/` 内にテストを追加/更新してください。
- **後方互換性**: MCP ツール定義からは削除されましたが、旧ツールは CLI モード (`godot-mcp-rs.exe call-tool ...`) 用に実装を保持しています。
- **禁止事項**:
  - `schema.graphql` に無い機能を勝手に追加しない。
  - `args: JSON!` を個別の型にバラさない。

---

## 開発コマンド

```bash
cargo build          # ビルド
cargo test           # テスト実行（TDD 必須）
cargo clippy         # リント（警告ゼロを目指す）
cargo run -- call-tool <NAME> <JSON_ARGS>  # 特定ツールの直接デバッグ
```

## 環境

- **Rust**: Edition 2021
- **主要クレート**: `rmcp`（MCP）, `tokio`（非同期）, `nom`（パーサー）, `reqwest`（HTTP）

## コンテキスト管理

### 長い作業の分割

- 1 つの Phase 完了ごとに会話を分割することを推奨
- 完了時は必ずドキュメントを更新してから終了

### 引き継ぎ情報の提示

タスク完了時、以下を明示的に伝える:

1. 何が完了したか
2. 次に何をすべきか
3. 参照すべきファイル
