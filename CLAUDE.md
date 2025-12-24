## Claude Code / LLM Agent ガイド（GQL）

このリポジトリで GQL（Godot Query Language）関連の作業をする LLM エージェント向けの入口です。  
**迷ったらまずここを読み、指示に従ってください。**

---

## 何を参照すべきか（読む順）

- **契約（設計 / Why・What）**: `docs/DESIGN_GQL.md`
- **スキーマ（単一ソース / SDL）**: `docs/gql/schema.graphql`
- **実装ノート（How）**: `docs/IMPLEMENTATION_GQL.md`
- **実装計画（順序 / DoD / TDD 運用）**: `docs/PLAN_GQL.md`

> 重要: スキーマの正は **`docs/gql/schema.graphql`** です。  
> `DESIGN_GQL.md` 内のスキーマ断片は説明用であり、差分管理対象ではありません。

---

## このリポジトリの基本構造（既存資産）

- **静的解析（ファイル）**: `src/godot/`（`.tscn/.gd/.tres`）
- **Live 操作（Editor 連携）**: `src/tools/live.rs`（HTTP/JSON → Godot plugin）
- **MCP プロトコル**: `rmcp`

---

## 作業ルール（TDD 前提）

### 優先順位

1. **契約を壊さない**（`schema.graphql` と `DESIGN_GQL.md`）
2. **テストを先に追加**
3. 最小実装でテストを通す
4. 必要ならリファクタ（テストを維持）

### 禁止事項

- 設計に無い挙動を、実装で勝手に追加しない
  - 必要なら「設計への提案」を先に出す
- `args` を **文字列 JSON** で扱わない（`args: JSON!` に統一）
- Live とファイルの “どちらが正か” を曖昧にしたまま混在 API を増やさない

---

## 変更時のチェックリスト（最小）

- スキーマ変更を伴う場合:
  - `docs/gql/schema.graphql` を更新
  - 影響と互換性を `docs/DESIGN_GQL.md` に追記
  - `docs/PLAN_GQL.md` の DoD/テスト方針に反映

---

## 開発コマンド

```bash
cargo build          # ビルド
cargo test           # テスト実行
cargo fmt            # フォーマット
cargo clippy         # リント
cargo run -- --help  # CLI ヘルプ表示
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
