## GQL 実装計画（When / Order / DoD）

この計画は `docs/DESIGN_GQL.md`（契約）と `docs/gql/schema.graphql`（SDL 単一ソース）を前提に、**テスト駆動（TDD）**で安全に実装するための順序を定義します。  
`docs/ROADMAP.md` は大枠のままで OK。GQL の詳細は本ドキュメントに集約します。

---

## 現状（前提）

- リポジトリ内に `tests/` がまだ無い（少なくとも命名で見つからない）
- `Cargo.toml` の `dev-dependencies` は最小（`tempfile` のみ）
- 既存資産として、静的解析（`src/godot/`）と live（`src/tools/live.rs`）がある

---

## フェーズ設計（推奨）

### Phase 0: 仕様を固定（ドキュメントの DoD）

- **DoD**
  - `docs/gql/schema.graphql` が単一ソースとして存在
  - `DESIGN_GQL.md` が契約（用語・エラー・原子性・整合・制限）を明記
  - 例（ユースケース）がスキーマと矛盾しない

### Phase 1: スキーマ契約テスト（最優先）

- **狙い**: 以降の実装で“勝手にスキーマが変わる”事故を防ぐ
- **テスト**
  - SDL のスナップショット/差分検知（例: ゴールデンテスト）
  - 代表クエリが「型レベルで通る」ことの検証
- **DoD**
  - SDL の契約テストが CI 相当で緑

### Phase 2: Read（Query）から実装

- **優先順**
  - `project`（統計/一覧/validation）
  - `scene`（最小の構造取得）
  - `script`（最小の解析）
- **DoD**
  - `test_project/` を使ったゴールデンが緑（レスポンス形状が安定）

### Phase 3: validate / preview / apply（安全フロー）

- **狙い**: “AI が壊す”懸念を最小にする中核機能
- **テスト**
  - validate: 失敗ケース（存在しないノード、プロパティ名ミス、型不整合）
  - preview: diff 生成（最小ケース）
  - apply: undoActionId の扱い（契約に沿う）
- **DoD**
  - validate/preview/apply が少なくともスタブ環境で E2E 緑

### Phase 4: Live 操作統合（HTTP 境界）

- **テスト**
  - HTTP スタブでの統合テスト
  - タイムアウト/再試行/排他（設計で決めた範囲）
- **DoD**
  - Live 系 Mutation が代表ケースで緑

### Phase 5: 依存関係グラフ / gatherContext（拡張）

- **DoD**
  - 小規模プロジェクトでの依存抽出が再現性を持つ
  - 出力形式（MERMAID 等）が安定

### Phase 6: MCP ツール統合（GQL を“3 ツール”として提供） [DONE]

- **狙い**: 既存の多数ツール定義を圧縮し、LLM が `godot_query/godot_mutate/godot_introspect` を中心に運用できる状態にする
- **作業**
  - MCP ツールとして以下を追加/公開
    - `godot_query`（GraphQL Query 実行）
    - `godot_mutate`（GraphQL Mutation 実行）
    - `godot_introspect`（スキーマ/SDL 取得、または introspection 実行）
  - 既存 56 ツールとの共存方針: MCP からは削除し、GraphQL に一本化。CLI 経由のみで互換性を維持。
- **DoD**
  - Claude Code 等から **3 ツールだけで**主要ユースケース（Phase 2-5）を実行可能
  - `docs/gql/schema.graphql` と実装が一致（契約が破れていない）
  - 旧ツールは MCP から削除され、LLM のツール選択コストが劇的に低下した

### Phase 7: 仕上げ（任意・品質） [DONE]

- **狙い**: 使い勝手と保守性を上げる
- **作業**
  - コンパイル警告の解消（unused import/変数など） [DONE]
  - ドキュメントの最終整合（Live/ファイル整合、制限、原子性の明文化） [DONE]
- **DoD**
  - 主要警告が解消され、リグレッションなくテストが安定

### Phase 8: 拡張（任意・機能） [DONE]

- **狙い**: 欠落していた実用的な機能の追加
- **作業**
  - `nodeTypeInfo` の実装（Godot 内部データベースの公開） [DONE]
  - ファイルベース操作（`createScene`, `createScript`）の実装 [DONE]
- **DoD**
  - 追加機能が契約（SDL）とテストで拘束され、動作が正常であることを確認済

---

## AI に渡す TDD 運用ルール（コピペ用）

### 実装手順

1. 追加/変更したい契約（要件）を `DESIGN_GQL.md` と `schema.graphql` で確認
2. **テストを先に追加**（失敗する状態を作る）
3. 最小実装でテストを通す
4. 必要ならリファクタ（テストを維持）

### 禁止事項

- 設計に無い挙動の追加（必要なら **設計へ提案して止まる**）
- 文字列 JSON の生成で args を渡す（`args: JSON!` に統一）

---

## リスクと緩和

- **スキーマが肥大化**: SDL を単一ソース化し、設計本文はリンク中心にする
- **原子性/Undo の期待ズレ**: apply の契約（部分適用/rollback/undo 粒度）を設計で固定し、テストで拘束
- **Live とファイル整合**: “どちらが正か”を設計に明記し、混在 API では注意書きを必須にする
