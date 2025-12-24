# Phase 1-5 完了チェックリスト

## ✅ 完了確認（2025-01-XX）

### Phase 0: 仕様固定
- [x] `docs/gql/schema.graphql` が単一ソースとして存在
- [x] `DESIGN_GQL.md` が契約を明記
- [x] ユースケース例がスキーマと矛盾しない

### Phase 1: スキーマ契約テスト
- [x] SDLスナップショットテスト (`tests/schema_contract_test.rs`)
- [x] 代表クエリの型検証テスト
- [x] 代表ミューテーションの型検証テスト
- **DoD達成**: ✅ SDLの契約テストがCI相当で緑

### Phase 2: Read（Query）実装
- [x] `project` Query実装・テスト
- [x] `scene` Query実装・テスト
- [x] `script` Query実装・テスト
- [x] `test_project/` を使ったゴールデンテスト
- **DoD達成**: ✅ `test_project/` を使ったゴールデンが緑

### Phase 3: validate / preview / apply（安全フロー）
- [x] `validateMutation` 実装・テスト（失敗ケース含む）
- [x] `previewMutation` 実装・テスト
- [x] `applyMutation` 実装・テスト
- **DoD達成**: ✅ validate/preview/apply がスタブ環境でE2E緑

### Phase 4: Live 操作統合
- [x] Live Query (`currentScene`, `node`) 実装・テスト
- [x] Live Mutation (`addNode`, `setProperty`, `connectSignal` 等) 実装・テスト
- [x] HTTP接続エラー時の挙動テスト
- **DoD達成**: ✅ Live系Mutationが代表ケースで緑

### Phase 5: 依存関係グラフ / gatherContext
- [x] `gatherContext` Query実装・テスト
- [x] `dependencyGraph` Query実装・テスト
- [x] 出力形式（MERMAID等）の実装
- **DoD達成**: ✅ 小規模プロジェクトでの依存抽出が再現性を持つ

---

## ⚠️ 確認事項・改善提案

### 1. MCPツール統合（設計書には記載、Phase 3のDoDには未記載）

**現状**: `godot_query`, `godot_mutate`, `godot_introspect` が `src/tools/mod.rs` の `list_tools` に含まれていない

**判断**: 
- Phase 1-5 のDoDには含まれていないため、**Phase 6（統合）として扱う**のが適切
- ただし設計書（`DESIGN_GQL.md`）には「MCPツール定義（最小化）」として記載されている

**推奨アクション**:
- Phase 6 として実装計画に追加
- または、既存56ツールとの共存方針を明確化（GQL優先の告知など）

### 2. TODO の残存（重要度別）

#### 高優先度（Phase 1-5 のDoDに影響）
- なし（Phase 1-5 のDoDは達成済み）

#### 中優先度（機能拡張）
- `src/graphql/schema.rs`: `nodeTypeInfo` resolver（TODO）
  - 設計には含まれているが、Phase 1-5 のDoDには必須ではない
- `src/graphql/schema.rs`: ファイルベース操作（`createScene`, `createScript` 等）のTODO
  - 設計には含まれているが、Phase 1-5 のDoDには必須ではない

#### 低優先度（実装の細部）
- `src/graphql/dependency_resolver.rs`: GraphML export（TODO）
  - MERMAID/DOT/JSONは実装済み、GraphMLは未実装
- `src/graphql/resolver.rs`: シーン解析の細部（groups, signals のパース）
  - 基本機能は動作、完全性向上の余地

### 3. 設計との整合性チェック

#### ✅ 禁止事項の遵守
- **文字列JSONでargsを渡す**: ✅ 遵守
  - `PlannedOperation.args` は `JSON!` 型を使用
  - `resolver.rs:336` の `args: Vec<String>` はシグナル引数のパース処理で、禁止事項とは別文脈（問題なし）

- **設計外挙動の追加**: ✅ 遵守
  - 実装は設計書とスキーマSDLに沿っている

#### ⚠️ 設計との整合（要確認）
- **Live とファイルの整合**: 設計書に「どちらが正か」の明記が必要
  - 現状: `scene(path)`（ファイル）と `currentScene`（live）が併存
  - 推奨: `DESIGN_GQL.md` に「未保存変更の扱い」を追記

### 4. コンパイル警告（軽微）

以下の警告は機能に影響しないが、クリーンアップ推奨:
- `src/graphql/types.rs:7`: 未使用import (`Deserialize`, `Serialize`)
- `src/tools/live.rs:29`: 未使用変数 (`cmd_json`)
- `src/graphql/resolver.rs:566`: 未使用変数 (`index`, `op`)
- `src/graphql/resolver.rs:552`: 不要な `mut`
- `src/main.rs:10`: 未使用import (`graphql`)
- `tests/dependency_test.rs:301`: 未使用変数 (`edges`)

**推奨**: `cargo fix --lib -p godot-mcp-rs` で自動修正可能なものは修正

---

## 📋 次のステップ（Phase 6以降）

### Phase 6: MCPツール統合（推奨）
- [ ] `godot_query` ツール実装
- [ ] `godot_mutate` ツール実装
- [ ] `godot_introspect` ツール実装
- [ ] 既存56ツールとの共存方針の明確化

### Phase 7: 機能拡張（任意）
- [ ] `nodeTypeInfo` resolver実装
- [ ] ファイルベース操作（`createScene`, `createScript`）の完全実装
- [ ] GraphML export実装
- [ ] シーン解析の完全性向上（groups, signals の完全パース）

### Phase 8: クリーンアップ（任意）
- [ ] コンパイル警告の解消
- [ ] 設計書への「Live/ファイル整合」の明記
- [ ] ドキュメントの最終確認

---

## 🎯 結論

**Phase 1-5 は完了**: すべてのDoDが達成されています。

**次のアクション**:
1. **Phase 6（MCPツール統合）**を実装計画に追加して着手
2. または、既存56ツールとの共存方針を明確化して「GQL優先」を告知
3. コンパイル警告のクリーンアップ（任意）

**完了判定**: Phase 1-5 の実装とテストは完了。Phase 6以降は「次のマイルストーン」として扱うのが適切。

