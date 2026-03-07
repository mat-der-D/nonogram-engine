# 実装計画

## 並行実行フェーズ概要

- **フェーズ 1（並行）**: タスク 1・2・3 はすべて独立して並行実行可能
- **フェーズ 2**: タスク 4 はタスク 1・2 完了後に実行
- **フェーズ 3（並行）**: タスク 5・6・7 はタスク 3 完了後に並行実行可能
- **フェーズ 4**: タスク 8 はタスク 3・4 完了後に実行
- **フェーズ 5**: タスク 9 はタスク 5・6・7・8 完了後に実行

---

- [x] 1. (P) nonogram-format に Grid JSON スキーマを追加する

- [x] 1.1 (P) GridDto 型とシリアライズ関数を実装する
  - `GridDto` 構造体（`rows: usize`、`cols: usize`、`cells: Vec<Vec<bool>>`）を定義し、`serde` の `Serialize`・`Deserialize` を導出する
  - `grid_to_json(grid: &Grid) -> Result<String, FormatError>` を実装する
  - `json_to_grid(json: &str) -> Result<Grid, FormatError>` を実装し、`rows`/`cols` と `cells` の次元不一致を検出する
  - `FormatError` に `ShapeMismatch { rows, cols, actual_rows, actual_cols }` バリアントを追加する
  - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [x] 1.2 (P) Grid シリアライズのテストを実装する
  - `grid_to_json` → `json_to_grid` のラウンドトリップを検証するテストを追加する
  - `rows`/`cols` と `cells` 次元が不一致な JSON を渡した際に `FormatError::ShapeMismatch` が返ることを検証する
  - 不正 JSON フォーマット（フィールド欠落など）で `FormatError::Json` が返ることを検証する
  - _Requirements: 5.3, 5.4_

- [x] 2. (P) nonogram-core に画像変換パイプラインを実装する

- [x] 2.1 (P) 画像デコードとアルファ合成・グレースケール変換を実装する
  - `Cargo.toml` に `image = { version = "0.25", default-features = false, features = ["png", "jpeg", "webp", "gif"] }` と `imageproc = "0.25"` を追加する
  - `ImageConvertParams` 構造体（`grid_width`、`grid_height`、`smooth_strength`、`threshold`、`edge_strength`、`noise_removal`）を定義する
  - `ImageError::Decode` エラー型を `thiserror` で定義する
  - `image::load_from_memory` で画像をデコードし、アルファチャンネルを白背景に合成してグレースケール画像を生成する処理を実装する（`result = rgb * (alpha/255) + white * (1 - alpha/255)`）
  - モジュールファイル構成は `crates/nonogram-core/src/image.rs` + `crates/nonogram-core/src/image/convert.rs` とする（`mod.rs` 禁止規約に準拠）
  - _Requirements: 6.1, 6.4_

- [x] 2.2 (P) 変換パイプライン（ブラー〜ノイズ除去）の本体を実装する
  - `image_to_grid(image_bytes: &[u8], params: &ImageConvertParams) -> Result<Grid, ImageError>` の全パイプラインを実装する
  - パイプライン順序: グレースケール → ガウシアンブラー（sigma = `smooth_strength`、0 の場合スキップ）→ Canny エッジ検出（`low = threshold * 0.5`、`high = threshold as f32`）→ エッジマージ（`merged = gray * (1 - edge_strength) + edge * edge_strength`）→ セル平均ダウンサンプリング → 閾値処理（平均輝度 < threshold → `true`）→ ノイズ除去（4-connectivity connected components でサイズ < `noise_removal` の領域を除去）
  - `smooth_strength = 0` の場合はガウシアンブラーをスキップして処理時間を短縮する
  - _Requirements: 6.1, 6.2_

- [x] 2.3 (P) 画像変換関数のユニットテストを実装する
  - 既知の白黒 PNG バイト列（テスト用ミニマム画像）に対して期待するグリッドが生成されることを検証するテストを追加する
  - `noise_removal = 0` と `noise_removal > 0` で出力差が生じることを検証する
  - 不正バイト列を渡した際に `Err(ImageError::Decode)` が返ることを検証する
  - _Requirements: 6.4, 6.5_

- [x] 3. (P) useMakerStore をグリッドエディタの中央ストアとして実装する

- [x] 3.1 (P) グリッド状態管理と基本操作の reducer を実装する
  - `MakerState` インタフェース（`gridWidth`、`gridHeight`、`cells`、`history`、`future`、`solvePhase`、`isConvertOpen`、`isSolverOpen`）を定義する
  - `MakerAction` 型（`SET_DIMENSIONS`、`TOGGLE_CELL`、`COMMIT_HISTORY`、`DRAG_CELL`、`RESET_GRID`、`LOAD_GRID`、`SET_SOLVE_PHASE`、`SET_CONVERT_OPEN`、`SET_SOLVER_OPEN`）を定義する
  - `useReducer` ベースで reducer を実装し、`SET_DIMENSIONS`・`TOGGLE_CELL`・`RESET_GRID`・`LOAD_GRID` を処理する
  - `useMakerStore` フックとして `rowClues`・`colClues`（`useMemo` で `cells` 変更時のみ再計算）、`canUndo`、`canRedo`、`isExportable` の計算済みプロパティを提供する
  - `setDimensions`・`toggleCell`・`resetGrid`・`loadGrid` の各アクション関数を公開する
  - _Requirements: 1.1, 1.2, 1.4, 1.5, 7.1, 7.4_

- [x] 3.2 (P) undo/redo 履歴スタックとドラッグ操作を実装する
  - `COMMIT_HISTORY` アクションで現在の `cells` を `history` にプッシュし `future` をクリアする（上限 100 ステップ超で最古エントリを破棄）
  - `UNDO` アクションで `history` 末尾を pop して `future` 先頭に現在 `cells` をプッシュする
  - `REDO` アクションで `future` 先頭を pop して `history` に現在 `cells` をプッシュする
  - `startDrag(row, col, action)` で `COMMIT_HISTORY` を dispatch してからドラッグ開始セルに操作を適用し、`dragActionRef`（`useRef`）でドラッグ方向を保持する
  - `dragCell(row, col)` で `DRAG_CELL` を dispatch しドラッグ中セルに一方向の操作（塗り/消し）を適用する（history への追加なし）
  - `endDrag()` で `dragActionRef` をクリアする
  - _Requirements: 1.3, 1.6_

- [x] 3.3 ソルバー非同期実行とフェーズ管理を実装する
  - `solve()` 非同期関数を実装し、`SET_SOLVER_OPEN: true` と `SET_SOLVE_PHASE: solving` を同時に dispatch してから `WasmContext.solve` を呼び出す
  - `computeRowClues`・`computeColClues` で現在の `cells` からクルーを算出し、puzzle JSON を生成してソルバーに渡す
  - WASM ソルバーの結果（unique/multiple/none/error）を解析して `SET_SOLVE_PHASE: done` を dispatch する
  - ソルバーエラー時も `solvePhase.status = 'error'` に遷移させてエディタ操作に戻れる状態を維持する
  - `setConvertOpen`・`setSolverOpen` の UI 状態制御関数を公開する
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

- [x] 3.4 useMakerStore reducer のユニットテストを実装する
  - `COMMIT_HISTORY` + `TOGGLE_CELL` でドラッグ 1 回が 1 undo ステップになることを検証する
  - `UNDO`/`REDO` で `history`/`future` スタックが正しく操作されることを検証する
  - `RESET_GRID` で `cells` が全 `false` になり、`history` にスナップショットが追加されることを検証する
  - `SET_DIMENSIONS` でグリッドリサイズ後も既存 `cells` が保持またはトリミングされることを検証する
  - _Requirements: 1.1, 1.4, 1.6_

- [x] 4. nonogram-wasm に画像変換 WASM バインディングを追加する
  - `nonogram_core::image_to_grid` を呼び出し、成功時は `nonogram_format::grid_to_json` で Grid を JSON にシリアライズして `{"status":"ok","grid":{...}}` を返す `image_to_grid` WASM 関数を実装する
  - 関数シグネチャ: `image_to_grid(image_bytes: &[u8], grid_width: u32, grid_height: u32, smooth_strength: f32, threshold: u8, edge_strength: f32, noise_removal: u32) -> String`
  - エラー時は既存の `error_json(message)` ユーティリティを再利用して `{"status":"error","message":"..."}` を返す
  - `nonogram-wasm/Cargo.toml` に `nonogram-core` への依存が反映されていることを確認する（`nonogram-format` は既存依存）
  - _Requirements: 6.3_

- [ ] 5. (P) グリッドエディタ UI コンポーネントを実装する

- [ ] 5.1 (P) EditorGrid コンポーネントを実装する
  - CSS Grid を使った 4 領域レイアウト（角・列クルー・行クルー・セルグリッド）でグリッドを描画するコンポーネントを実装する
  - `onMouseDown` → `startDrag`、`onMouseEnter` → `dragCell`、`window` の `onMouseUp` → `endDrag` のマウスイベントを接続する
  - `Ctrl+Z` → `undo()`、`Ctrl+Shift+Z` / `Ctrl+Y` → `redo()` のキーボードショートカットをウィンドウ単位で登録する
  - ドラッグ中はカーソルを `crosshair` に変更し、塗り/消しをリアルタイム反映する
  - セルサイズは CSS カスタムプロパティで管理する（デスクトップ: 24px、モバイル: 16px）
  - 塗りつぶしセル: `#111`、空白: `#fff`、ホバー: `#ccc` で色を制御する
  - _Requirements: 1.2, 1.3_

- [ ] 5.2 (P) クルー表示と EditorToolbar コンポーネントを実装する
  - `useMakerStore.rowClues`・`colClues` をセル外（左・上）にリアルタイムで表示するクルー描画を実装する
  - クルーテキストはセルサイズに比例したフォントサイズで、行クルーは右寄せ、列クルーは下寄せとする
  - `EditorToolbar` に幅・高さ入力、Convert ボタン、Undo/Redo/リセットボタン、検証ボタン、Export ドロップダウン（JSON / PNG）を配置する
  - 各ボタンの無効化条件を制御する（`canUndo`・`canRedo`・`isExportable` に基づく）
  - Export ドロップダウンから `ExportService.exportJson` / `ExportService.exportPng` を呼び出す
  - _Requirements: 1.1, 1.4, 1.5, 4.3, 7.3_

- [ ] 6. (P) SolverModal コンポーネントを実装する
  - `solvePhase.phase === 'solving'` 時はスピナーと「解析中...」を表示し、「閉じる」ボタンを無効化する（エディタ・ツールバーはブロックしない）
  - `status === 'unique'` 時は「唯一解」バッジと解グリッド 1 枚を表示する
  - `status === 'multiple'` 時は「複数解」バッジと解グリッド最大 2 枚を横並びで表示する
  - `status === 'none'` / `'error'` 時はそれぞれのメッセージ（エラー時は詳細も）を表示する
  - モーダルを閉じた後も `solvePhase` を `done` のまま保持し、再度開いた際に前回結果を表示する
  - ConvertModal と同じフルスクリーンオーバーレイパターン（背景: 半透明黒、中央パネル）を踏襲する
  - _Requirements: 3.2, 3.3, 3.4, 3.5, 3.6_

- [ ] 7. (P) ExportService を実装する

- [ ] 7.1 (P) JSON エクスポートを実装する
  - `cells` から `computeRowClues`・`computeColClues` でクルーを計算し、`{ "row_clues": number[][], "col_clues": number[][] }` 形式の JSON を生成してブラウザダウンロードを発火する `exportJson(cells, filename?)` 関数を実装する
  - `<a download>` 要素と `URL.createObjectURL` を使ってダウンロードを行う
  - _Requirements: 4.1_

- [ ] 7.2 (P) PNG エクスポートを実装する
  - 動的に `<canvas>` を生成して DOM に追加せず、行・列クルーと問題グリッドを描画する `exportPng(cells, rowClues, colClues, filename?)` 非同期関数を実装する
  - グリッドサイズに応じてセルサイズを自動調整する（推奨: 最小 10px/セル）
  - クルーはグリッドの左（行）・上（列）に配置する
  - `canvas.toBlob` → `URL.createObjectURL` → `<a download>` でファイルをダウンロードする
  - _Requirements: 4.2_

- [ ] 7.3* ExportService のユニットテストを実装する
  - `exportJson` が `{ row_clues, col_clues }` 形式の正しい JSON を生成することを検証する
  - `isExportable` が全 `false` のグリッドで `false` を返すことを検証する
  - _Requirements: 4.1, 4.2, 4.3_

- [ ] 8. 画像変換 UI を実装する（タスク 3・4 完了後に実行）

- [ ] 8.1 useConvertState を実装する
  - `ConvertState`（`resizedBytes`・`params`・`previewGrid`・`isConverting`・`imageError`・`originalPreviewUrl`）と `ConvertParams` インタフェースを定義する
  - `loadImage(file)` で Canvas 2D API を使って画像を 384×384 境界ボックスにアスペクト比を保ちながらリサイズし（384px 未満はスケールアップしない）、`canvas.toBlob('image/png')` → `Uint8Array` に変換して `resizedBytes` にキャッシュする
  - リサイズ後の画像幅・高さが 50px 未満の場合は `params.gridWidth`・`params.gridHeight` の上限を制限する
  - `updateParam` でパラメータを更新し、`resizedBytes` が非 null の場合に 100ms デバウンス後に `WasmContext.image_to_grid` を呼び出して `previewGrid` を更新する
  - 画像読み込み失敗時は `imageError` にエラーメッセージを設定する
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.8_

- [ ] 8.2 ConvertModal と子コンポーネントを実装する
  - `ImageUploader`: ファイル選択 UI を実装し、`loadImage` を呼び出す。`imageError` が非 null の場合はエラーメッセージを表示する
  - `ParamSliders`: `grid_width`（5–50）・`grid_height`（5–50）・`smooth_strength`（0–5）・`threshold`（0–255）・`edge_strength`（0–1）・`noise_removal`（0–20）のスライダーを実装し、変換処理中はスライダーを無効化してスピナーを表示する
  - `PreviewComparison`: 元画像プレビューと生成済みドットグリッドプレビューを横並び（モバイルは縦積み）で表示する
  - `ConvertModal`: フルスクリーンオーバーレイで上記コンポーネントを統合し、「適用」ボタン押下時に `useMakerStore.loadGrid(previewGrid)` を呼び出してモーダルを閉じる。`useConvertState` はモーダル内でインスタンス化しモーダルを閉じると状態をクリアする
  - _Requirements: 2.1, 2.4, 2.6, 2.7, 2.8_

- [ ] 9. MakerApp 統合と既存 UI の廃止

- [ ] 9.1 既存コンポーネントを廃止してソースを整理する
  - `ClueInputPanel`・`ModeToggle`・`GridDrawingPanel`・`SolveButton`・`ResultPanel`・`ImportExportPanel`・`PuzzleSizeInput`・`useNonogramStore` を削除する
  - 継続利用する `clueComputeUtils`・`clueParseUtils`・`PuzzleIOService` を確認し、不要になった参照を除去する
  - _Requirements: 7.2_

- [ ] 9.2 MakerApp ルートコンポーネントを組み立てて全コンポーネントを統合する
  - `WasmProvider` → `MakerApp` の構造を構築し、`useMakerStore` から状態を取得して `EditorToolbar`・`EditorGrid`・`SolverModal`・`ConvertModal` に props を渡す
  - `isConvertOpen` に応じて `ConvertModal` をマウント・アンマウントし、`isSolverOpen` に応じて `SolverModal` の表示を制御する
  - `ExportService` を `EditorToolbar` の Export ドロップダウンと接続する
  - WASM 未初期化エラー時は既存パターンの赤背景エラーバナーを表示する
  - SPA として画面遷移なしに全機能へアクセスできることを確認する
  - _Requirements: 7.1, 7.3, 7.4_

- [ ] 9.3 レスポンシブ CSS とインタラクション状態を実装する
  - CSS カスタムプロパティでセルサイズを管理し、`@media (max-width: 767px)` でセルを 24px → 16px に縮小する
  - モバイルでツールバーをアイコンのみ表示に切り替えるメディアクエリを実装する
  - ConvertModal のプレビュー 2 列をモバイルで縦積みにする CSS を実装する
  - ドラッグ中のカーソル `crosshair`、変換処理中のスライダー無効化、Export ドロップダウンの 2 択メニューなどインタラクション状態の視覚フィードバックを整備する
  - _Requirements: 7.5_

---

## 要件カバレッジ確認

| 要件 | カバータスク |
|------|------------|
| 1.1 | 3.1, 5.2 |
| 1.2 | 3.1, 5.1 |
| 1.3 | 3.2, 5.1 |
| 1.4 | 3.1, 5.2 |
| 1.5 | 3.1, 5.2 |
| 1.6 | 3.2, 3.4 |
| 2.1 | 8.1, 8.2 |
| 2.2 | 2.1, 8.1 |
| 2.3 | 8.1 |
| 2.4 | 8.2 |
| 2.5 | 2.2, 8.1 |
| 2.6 | 8.2 |
| 2.7 | 8.2 |
| 2.8 | 8.1, 8.2 |
| 3.1 | 3.3 |
| 3.2 | 3.3, 6 |
| 3.3 | 3.3, 6 |
| 3.4 | 3.3, 6 |
| 3.5 | 3.3, 6 |
| 3.6 | 3.3, 6 |
| 4.1 | 7.1 |
| 4.2 | 7.2 |
| 4.3 | 5.2, 7 |
| 4.4 | スコープ外（Desktop App — 将来スペックで対応） |
| 5.1 | 1.1 |
| 5.2 | 1.1 |
| 5.3 | 1.1 |
| 5.4 | 1.1 |
| 6.1 | 2.1, 2.2 |
| 6.2 | 2.2 |
| 6.3 | 4 |
| 6.4 | 2.1, 2.3 |
| 6.5 | 2.3 |
| 7.1 | 3.1, 9.2 |
| 7.2 | 9.1 |
| 7.3 | 5.2, 9.2 |
| 7.4 | 3.1, 9.2 |
| 7.5 | 9.3 |
