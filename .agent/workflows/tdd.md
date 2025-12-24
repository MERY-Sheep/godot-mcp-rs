---
description: godot-mcp-rsの開発をTDD（テスト駆動開発）で進める
---

`godot-mcp-rs` の機能を TDD で実装する際の手順です。

1. **Schema 定義**: `docs/gql/schema.graphql` にクエリまたはミューテーションを追加する。
2. **型定義**: `src/graphql/types.rs` に対応する Rust の構造体を追加する。
3. **テストの作成**: `src/graphql/resolver.rs` または `src/graphql/live_resolver.rs` のテストモジュールに、失敗するテストケース（Red）を追加する。
4. **リゾルバの実装**: テストが通る最小限の実装を行う（Green）。
5. **リファクタリング**: コードを整理し、重複を排除する（Refactor）。
6. **Godot 側実装**: 必要に応じて `addons/godot_mcp/command_handler.gd` にハンドラーを追加する。

// turbo 7. `cargo test` を実行して、すべてのテストがパスすることを確認する。
