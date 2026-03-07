# リサーチ & 設計決定ログ

---
**Purpose**: 技術設計を裏付ける調査内容・アーキテクチャ評価・設計決定の根拠を記録する。

---

## サマリー

- **Feature**: `nonogram-maker`
- **Discovery Scope**: Complex Integration（既存ソルバー SPA を問題作成アプリへ全面刷新）
- **Key Findings**:
  - 既存 `nonogram-core` は画像処理依存を持たず、`image` + `imageproc` クレートの追加で処理パイプラインを実装できる
  - `nonogram-format` はすでに `nonogram-core` に依存しているため、`GridDto` の追加は依存関係変更なしで可能
  - Undo/Redo は React reducer で全グリッドスナップショットを履歴スタックに積む方式が最も単純かつ安全（50×50 グリッド × 50 ステップ ≈ 125KB）
  - PNG エクスポートは Canvas 2D API のみで実現可能（Rust/WASM 不要）
  - 画像変換パイプラインは Rust 側（nonogram-core）に実装し WASM 経由で呼び出す（要件 6 の明示要件）

---

## Research Log

### 画像処理クレートの選定（nonogram-core）

- **Context**: 要件 6 が nonogram-core に画像→Grid 変換関数を求めており、Rust で画像デコード・処理が必要
- **Sources Consulted**:
  - `image` クレート公式ドキュメント（version 0.25）
  - `imageproc` クレート README（version 0.25）
- **Findings**:
  - `image = "0.25"`: PNG, JPEG, WebP, GIF（`image-gif` feature）のデコードをサポート。アルファ合成 API も内包
  - `imageproc = "0.25"`: `filters::gaussian_blur_f32`, `edges::canny`, `region_labelling::connected_components` が要件パイプラインに対応
  - WASM ターゲットでのビルド: `image` クレートは `default-features = false` + `png, jpeg, webp, gif` features 指定が必要（スレッド依存機能を除外）
  - `imageproc` は `image` クレートへの依存を持つため、バージョン統一が必要
- **Implications**:
  - `nonogram-core/Cargo.toml` に `image` と `imageproc` を追加
  - WASM ビルド時はフィーチャーフラグを慎重に設定し、rayon（並列処理）依存を排除する

### Canny エッジ検出と Edge Merge 実装

- **Context**: 要件 2.5 のパイプラインで「Canny エッジ → エッジマージ」が必要
- **Findings**:
  - `imageproc::edges::canny(image, low_threshold, high_threshold)`: 入力は `GrayImage`、出力も `GrayImage`（エッジが白）
  - エッジマージ計算: `merged_pixel = gray_pixel * (1 - edge_strength) + edge_pixel * edge_strength`（各ピクセル）
  - ダウンサンプリング: グリッドの各セルに対応するピクセル矩形の平均輝度を計算
- **Implications**:
  - `smooth_strength` を Gaussian blur の sigma 値として直接 `imageproc::filters::gaussian_blur_f32` に渡す
  - Canny のしきい値は `low_threshold = threshold * 0.5`, `high_threshold = threshold` の比率で派生させる（パラメータを減らす）

### ノイズ除去（connected component filtering）

- **Context**: 要件 2.5 の最終ステップとして、最小サイズ以下の連結領域を除去する
- **Findings**:
  - `imageproc::region_labelling::connected_components(image, connectivity, background)` で連結成分にラベルを付与
  - ラベルごとのサイズカウント → `noise_removal` 未満のラベルを背景色に置換
  - 接続性: 4-connectivity（対角を含まない）がノイズ除去では一般的
- **Implications**:
  - 変換関数内で連結成分カウントマップを構築し、サイズフィルタリングを実施

### undo/redo の実装戦略

- **Context**: 要件 1.6 が 1 ステップずつの undo/redo を求める
- **Findings**:
  - グリッドサイズが最大 50×50 = 2500 セル（boolean）。スナップショット 1 個 ≈ 2.5KB
  - 履歴 50 ステップでも ≈ 125KB — メモリ上問題なし
  - React の `useReducer` パターンで `history: boolean[][][]` / `future: boolean[][][]` を state に持たせると実装が単純
  - ドラッグ操作（複数セル）は 1 ドラッグ = 1 undo ステップとする（ドラッグ開始時にスナップショット保存、終了後は保存しない）
- **Implications**:
  - `useMakerStore` の reducer に `COMMIT_HISTORY` action を追加（ドラッグ開始時に現在グリッドを `history` にプッシュ）
  - `undo` / `redo` は `history` / `future` スタックを操作

### Canvas API による PNG エクスポート

- **Context**: 要件 4.2 が行・列クルーを含む PNG 画像出力を求める
- **Findings**:
  - ブラウザ Canvas 2D API で描画し `canvas.toBlob('image/png')` → `URL.createObjectURL()` → `<a download>` click で実現
  - クルーテキストは `ctx.fillText()` で描画。フォントサイズをセルサイズに応じて調整
  - WASM 不要でブラウザ標準 API のみで完結
- **Implications**:
  - `ExportService.exportPng(cells, rowClues, colClues)` を TypeScript で実装
  - サービス層で Canvas を動的生成（`document.createElement('canvas')`）し DOM に追加しない

### Grid JSON スキーマの設計（nonogram-format）

- **Context**: 要件 5 が `nonogram-format` での Grid JSON スキーマ提供を求める
- **Findings**:
  - `nonogram-format` はすでに `nonogram-core` に依存しており、`Grid` 型を受け取れる
  - 既存の `PuzzleDto` / `SolutionDto` パターンに倣い `GridDto { rows, cols, cells }` を追加
  - `cells` を `Vec<Vec<bool>>` とすることで行優先の 2 次元配列として表現（要件 5.2 に合致）
  - デシリアライズ時の行/列数不一致バリデーションが必要（新 `FormatError::ShapeMismatch` variant）
- **Implications**:
  - `nonogram-format/src/lib.rs` に `GridDto`, `grid_to_json`, `json_to_grid` を追加
  - WASM 側で `image_to_grid` の結果を `grid_to_json` でシリアライズして返す

### アプリ UI 全体フロー

- **Context**: 要件 7 が問題作成ワークフローの段階的画面フローを求める
- **Findings**:
  - 既存 App は Solver 中心の UI。ConvertModal, SolverPanel 等の新コンポーネントを追加し、主画面はグリッドエディタに変更
  - シングルページで `appPhase: 'editor' | 'solver-result'` と `isConvertOpen: boolean` で状態管理
  - ルーティング（react-router 等）は不要 — ステート制御のみで画面切り替え可能
- **Implications**:
  - `useMakerStore` にアプリ全体フェーズ管理を統合
  - 既存コンポーネント（`ImportExportPanel`, `GridDrawingPanel` 等）を廃止して新コンポーネントに置換

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 全スナップショット Undo/Redo | reducer state に `history: Grid[]` / `future: Grid[]` を持つ | 実装単純、バグが少ない、immutable | メモリ使用量（最大 125KB — 問題なし） | 採用 |
| コマンドパターン Undo/Redo | 各操作を `Command` オブジェクトとして記録し `execute`/`undo` を持つ | メモリ効率 | 実装複雑、全操作に Command クラスが必要 | 不採用 |
| JS 側で画像変換 | Canvas API + JS で全パイプラインを実装 | WASM ビルド不要 | 要件 6 違反（nonogram-core での実装が必須）, パフォーマンス | 不採用 |
| Rust 側で画像変換（採用） | nonogram-core に変換ロジック、nonogram-wasm で WASM export | 要件準拠、再利用可能、型安全 | WASM ビルド複雑性 | 採用 |
| PNG エクスポートを Rust で実装 | `image` クレートで PNG 生成し WASM export | サーバーサイド再利用可 | 実装コスト高、Canvas API で十分 | 不採用 |
| 別途 React Context で Convert 状態管理 | `ConvertContext` を独立して設ける | 関心分離 | Context 増加、コンポーネント間連携が複雑 | 不採用 — `useMakerStore` + local state で十分 |

---

## Design Decisions

### Decision: `useMakerStore` の新規作成（既存 `useNonogramStore` を廃止）

- **Context**: 既存 `useNonogramStore` はソルバー入力 UI 向けの設計で、clue mode / grid mode トグル・clue string 入力等が含まれる。要件 7.2 がソルバー入力 UI の廃止を明示
- **Alternatives Considered**:
  1. 既存 `useNonogramStore` を拡張し undo/redo・変換機能を追加
  2. 新規 `useMakerStore` を作成し既存ファイルを廃止
- **Selected Approach**: `useMakerStore` を新規作成。既存の `useNonogramStore` と関連コンポーネントを廃止
- **Rationale**: 既存ストアの `InputMode`, `clueErrors`, `rowClueInputs`, `colClueInputs` 等は問題作成 UI では不要。拡張より刷新の方が技術的負債が少ない
- **Trade-offs**: 既存テスト（`useNonogramStore.test.ts`）は廃止または移行が必要
- **Follow-up**: `PuzzleIOService` は JSON import/export に継続使用可能か検討

### Decision: 画像変換の ResizedImage キャッシュ戦略

- **Context**: 要件 2.3 がリサイズ済み画像をキャッシュし、パラメータ変更時に再利用を求める
- **Alternatives Considered**:
  1. React `useRef` で TypeScript 側にキャッシュ
  2. WASM 側でキャッシュ（グローバル Mutex）
- **Selected Approach**: TypeScript 側 `useConvertState` の `useRef` でキャッシュ（`resizedImageBytes: Uint8Array | null`）
- **Rationale**: WASM のグローバル状態はテストが困難でスレッドセーフの懸念がある。JS 側のキャッシュは React ライフサイクルと自然に統合
- **Trade-offs**: キャッシュが JS ヒープに載るが、384×384 RGBA = 589KB — 許容範囲
- **Follow-up**: 実装時に `useEffect` のクリーンアップで Uint8Array を解放するか確認

### Decision: Grid JSON スキーマを nonogram-format に追加

- **Context**: 要件 5 が Grid の JSON スキーマを `nonogram-format` で提供することを明示
- **Alternatives Considered**:
  1. `nonogram-wasm` の WASM 層で inline にシリアライズ
  2. `nonogram-format` に `GridDto` を追加
- **Selected Approach**: `nonogram-format` に `GridDto` + `grid_to_json` + `json_to_grid` を追加
- **Rationale**: 再利用性（CLI・Desktop App からも利用可能）、要件 5 の明示的な指定
- **Trade-offs**: nonogram-format の責務が広がるが、既存パターン（PuzzleDto, SolutionDto）に準じており一貫性がある

---

## Risks & Mitigations

- **WASM ビルドサイズ増大（image クレート）** — `default-features = false` で必要フォーマットのみ有効化; `wasm-opt` でサイズ最適化
- **imageproc の WASM 非対応 API** — `rayon` フィーチャーを無効化; テスト時は native + wasm-pack test 両方で確認
- **Canny パラメータ調整の難しさ** — ユーザーは `smooth_strength` と `threshold` を調整; Canny 内部しきい値は `threshold` から自動算出してパラメータ数を最小化
- **大サイズ画像のパフォーマンス** — 384×384 にリサイズ後に処理するため入力サイズ非依存; リサイズはブラウザ Canvas API で実施してから WASM に渡す選択肢も検討
- **undo 履歴の上限なし** — 最大履歴件数（例: 100）を設定してメモリ上限を保証

---

## References

- image クレート（Rust 画像処理基盤）
- imageproc クレート（Gaussian blur, Canny, connected components）
- wasm-bindgen 公式ガイド — `&[u8]` の JS↔WASM 受け渡し
- Canvas API MDN — `toBlob`, `fillText`, `fillRect`
- 既存 steering: `.kiro/steering/tech.md`（WASM ビルド規約、TypeScript strict mode）
- 既存 steering: `.kiro/steering/structure.md`（依存関係ルール: nonogram-core → nonogram-format 禁止）
