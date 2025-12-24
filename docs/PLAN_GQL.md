## GQL 実装計画（When / Order / DoD）

この計画は `docs/DESIGN_GQL.md`（契約）と `docs/gql/schema.graphql`（SDL単一ソース）を前提に、**テスト駆動（TDD）**で安全に実装するための順序を定義します。  
`docs/ROADMAP.md` は大枠のままでOK。GQLの詳細は本ドキュメントに集約します。

---

## 現状（前提）
- リポジトリ内に `tests/` がまだ無い（少なくとも命名で見つからない）
- `Cargo.toml` の `dev-dependencies` は最小（`tempfile` のみ）
- 既存資産として、静的解析（`src/godot/`）と live（`src/tools/live.rs`）がある

---

## フェーズ設計（推奨）

### Phase 0: 仕様を固定（ドキュメントのDoD）
- **DoD**
  - `docs/gql/schema.graphql` が単一ソースとして存在
  - `DESIGN_GQL.md` が契約（用語・エラー・原子性・整合・制限）を明記
  - 例（ユースケース）がスキーマと矛盾しない

### Phase 1: スキーマ契約テスト（最優先）
- **狙い**: 以降の実装で“勝手にスキーマが変わる”事故を防ぐ
- **テスト**
  - SDLのスナップショット/差分検知（例: ゴールデンテスト）
  - 代表クエリが「型レベルで通る」ことの検証
- **DoD**
  - SDLの契約テストがCI相当で緑

### Phase 2: Read（Query）から実装
- **優先順**
  - `project`（統計/一覧/validation）
  - `scene`（最小の構造取得）
  - `script`（最小の解析）
- **DoD**
  - `test_project/` を使ったゴールデンが緑（レスポンス形状が安定）

### Phase 3: validate / preview / apply（安全フロー）
- **狙い**: “AIが壊す”懸念を最小にする中核機能
- **テスト**
  - validate: 失敗ケース（存在しないノード、プロパティ名ミス、型不整合）
  - preview: diff生成（最小ケース）
  - apply: undoActionId の扱い（契約に沿う）
- **DoD**
  - validate/preview/apply が少なくともスタブ環境でE2E緑

### Phase 4: Live 操作統合（HTTP境界）
- **テスト**
  - HTTPスタブでの統合テスト
  - タイムアウト/再試行/排他（設計で決めた範囲）
- **DoD**
  - Live系Mutationが代表ケースで緑

### Phase 5: 依存関係グラフ / gatherContext（拡張）
- **DoD**
  - 小規模プロジェクトでの依存抽出が再現性を持つ
  - 出力形式（MERMAID等）が安定

---

## AIに渡すTDD運用ルール（コピペ用）

### 実装手順
1. 追加/変更したい契約（要件）を `DESIGN_GQL.md` と `schema.graphql` で確認
2. **テストを先に追加**（失敗する状態を作る）
3. 最小実装でテストを通す
4. 必要ならリファクタ（テストを維持）

### 禁止事項
- 設計に無い挙動の追加（必要なら **設計へ提案して止まる**）
- 文字列JSONの生成で args を渡す（`args: JSON!` に統一）

---

## リスクと緩和
- **スキーマが肥大化**: SDLを単一ソース化し、設計本文はリンク中心にする
- **原子性/Undoの期待ズレ**: applyの契約（部分適用/rollback/undo粒度）を設計で固定し、テストで拘束
- **Liveとファイル整合**: “どちらが正か”を設計に明記し、混在APIでは注意書きを必須にする


