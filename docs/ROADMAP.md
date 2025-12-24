# Godot MCP Server ロードマップ 🚀

> **目標**: 最強の Godot MCP サーバーを構築する

## 現在のステータス

| 指標                 | 現状                        |
| :------------------- | :-------------------------- |
| ツール数             | **56+**                     |
| 実装言語             | Rust                        |
| リアルタイム操作     | ✅ 対応 (`live-*` ツール群) |
| Undo/Redo 統合       | ✅ 完全対応                 |
| エディタープラグイン | ✅ GDScript                 |
| CLI モード           | ✅ 対応                     |

---

## Phase 1: 開発サイクル支援 (GQL Native) 🔄

**優先度: 🔴 最高** | **目標: 実践的なゲーム開発サイクルの統合**

GQL の構造化されたインターフェースを活かし、テスト、設定、翻訳などの「開発ループ」を直接支援します。これにより、AI が自律的に機能を実装・検証するループを確立します。

### 1.1 テスト駆動開発 (TDD) 支援

- [x] `mutation runTests` - GdUnit4 等と連携し、テスト結果を構造化データ(成功数/失敗リスト/行番号)で返却。
- [x] AI が「失敗したテストのみを修正して再実行」するループを確立。

### 1.2 プロジェクト設定 & 入力マップ

- [ ] `mutation addInputAction` - InputMap へのアクション・イベント追加。
- [ ] `mutation setProjectSetting` - ProjectSettings の安全な変更。

### 1.3 国際化 (i18n)

- [x] `mutation addTranslationKey` - 翻訳キーと訳文を管理(CSV/PO)。コード生成との整合性を保つ。

---

## Phase 2: 高度なデバッグ統合 🐛

**優先度: 🟠 高** | **目標: AI による完全自動デバッグの実現**

### 2.1 エラー・ログ情報の構造化取得

- [x] `query debuggerErrors` - デバッガーパネルからスタックトレース付きで取得
- [x] `query logs` - ログ出力元（ファイル・行）を含めたログ取得
- [ ] `live_get_parse_errors` - スクリプトの書き込み直後の構文エラー検知 (GDScript 解析強化)

### 2.2 ブレークポイント & 実行制御

- [x] `mutation setBreakpoint` / `mutation removeBreakpoint`
- [x] `mutation pause` / `mutation resume` / `mutation step` - ステップ実行によるロジック追跡

### 2.3 実行時状態のインスペクション

- [ ] `live_get_stack_frame_vars` - 特定のスタックフレームのローカル変数取得
- [x] `query objectById` - ID を指定したオブジェクトの詳細状態（プロパティ一覧）取得

---

## Phase 3: コード生成・リファクタリング 🔧

**優先度: 🟡 中** | **目標: AI による高度なコード操作**

### 3.1 コード理解

- [ ] `get_class_hierarchy` - クラス階層の取得
- [ ] `find_references` - シンボル参照検索
- [ ] `get_autoloads` - オートロード一覧
- [ ] `analyze_dependencies` - 依存関係分析

### 3.2 リファクタリング

- [ ] `rename_symbol` - シンボル名変更（ファイル横断）
- [ ] `extract_function` - 関数の抽出
- [ ] `move_node_to_scene` - ノードを別シーンに移動

### 3.3 コード生成

- [ ] `generate_input_handler` - 入力ハンドラー自動生成
- [ ] `generate_state_machine` - ステートマシン雛形生成
- [ ] `generate_test_script` - テストスクリプト生成

### 3.4 Native Shader サポート

- [ ] `query validateShader` - シェーダーコードのコンパイル事前検証。
- [ ] `mutation createVisualShaderNode` - ビジュアルシェーダーのグラフ操作（長期目標）。

---

## Phase 4: アセット管理強化 🎨

**優先度: � 中** | **目標: 完全なアセットパイプライン**

### 4.1 リソース操作

- [ ] `create_material` - マテリアル作成
- [ ] `create_mesh` - プロシージャルメッシュ生成
- [ ] `import_asset` - 外部アセットのインポート
- [ ] `live_set_texture` - テクスチャの動的変更

### 4.2 プリセット・テンプレート拡張

- [ ] `vehicle_3d` - 車両テンプレート
- [ ] `npc_3d` - NPC テンプレート（NavMesh 対応）
- [ ] `collectible` - 収集アイテムテンプレート
- [ ] `projectile` - 弾丸・プロジェクタイルテンプレート

### 4.3 シェーダー操作

- [ ] `create_shader` - シェーダー作成
- [ ] `analyze_shader` - シェーダー解析
- [ ] `live_set_shader_param` - シェーダーパラメータ変更

---

## Phase 5: ビジュアル検証システム � 　＊一時凍結

**優先度: 低** | **目標: 競合 GDAI MCP との機能差を埋める**

### 5.1 スクリーンショットキャプチャ

- [ ] `live_capture_screenshot` - エディタービューポートのスクリーンショット取得
- [ ] `live_capture_game_screenshot` - 実行中ゲームのスクリーンショット取得
- [ ] 画像を Base64 で MCP レスポンスに含める

### 5.2 ビジュアル検証ループ

- [ ] AI がゲームを実行 → スクリーンショット撮影 → 結果検証のフロー
- [ ] 期待する状態との差分検出（例: 壁の色が変わったか確認）

### 5.3 実装詳細

```
Godot Plugin側:
  - get_viewport().get_texture().get_image() でキャプチャ
  - PNG/WebP形式でBase64エンコード

Rust CLI側:
  - 画像データをMCPレスポンスとして返却
  - オプションでファイル保存
```

---

## Phase 6: パフォーマンス分析 📊

**優先度: 低** | **目標: パフォーマンス問題の自動検出**

### 6.1 プロファイリング

- [ ] `start_profiler` - プロファイラー開始
- [ ] `stop_profiler` - プロファイラー停止
- [ ] `get_profiler_data` - プロファイリング結果取得

### 6.2 分析ツール

- [ ] `analyze_draw_calls` - ドローコール分析
- [ ] `analyze_memory_usage` - メモリ使用量分析
- [ ] `find_performance_issues` - パフォーマンス問題検出

### アーキテクチャ改善案

### 短期

- [ ] **WebSocket 対応**: HTTP→WebSocket に移行し、リアルタイム双方向通信を強化
- [ ] **接続プーリング**: 複数コマンドの効率的な処理
- [ ] **エラーリカバリー**: プラグイン切断時の自動再接続

### 中期

- [ ] **イベントストリーム**: エディターイベントをリアルタイムで AI に通知
- [ ] **並列コマンド実行**: 複数ノード操作の同時実行

### 長期

- [ ] **Godot 5 対応準備**: 次期バージョンへの移行計画
- [ ] **Language Server Protocol 統合**: IDE との連携強化

---

---

## マイルストーン

| フェーズ  | 目標期間 | 主な成果物                        |
| :-------- | :------- | :-------------------------------- |
| Phase 1   | 3 週間   | 開発サイクル支援 (TDD/Input/i18n) |
| Phase 2   | 3 週間   | 完全デバッグ統合                  |
| Phase 3   | 2 週間   | コード生成・リファクタリング      |
| Phase 4   | 3 週間   | アセット管理ツール群              |
| Phase 5   | 2 週間   | スクリーンショット検証システム    |
| Phase 6   | 2 週間   | パフォーマンス分析                |
| Phase 7-8 | 継続的   | 拡張機能                          |

---

## 貢献ガイド

新機能を追加する際は:

1. **Rust 側** (`src/tools/`): 新しいリクエスト型とハンドラーを追加
2. **プラグイン側** (`addons/godot_mcp/`): `command_handler.gd`にコマンド追加
3. **ドキュメント**: `USAGE.md`を更新
4. **テスト**: 動作確認とエッジケースのテスト

---

> 💡 **フィードバック歓迎**: このロードマップは進化し続けます。新しいアイデアや優先順位の変更は Issue で議論しましょう！
