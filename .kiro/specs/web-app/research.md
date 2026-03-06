# リサーチ・設計決定記録

---
**目的**: 技術設計に影響を与える調査結果、アーキテクチャ調査、決定根拠を記録する。

---

## サマリ

- **フィーチャー**: `web-app`
- **ディスカバリスコープ**: Extension（既存 Vite + React スキャフォールドへの本格実装追加）
- **主要な発見**:
  - Vite 7 で wasm-bindgen 生成モジュールを利用するには `vite-plugin-wasm` + `vite-plugin-top-level-await` が必要
  - WASM `solve()` は同期実行のため、React がローディング状態を描画できるよう `setTimeout(0)` で 1 tick 遅延させる必要がある
  - グリッドセル描画のドラッグ操作は外部ライブラリ不要で Pointer Events API のみで実装できる

---

## リサーチログ

### WASM と Vite 7 の統合

- **背景**: wasm-pack がビルドした `nonogram-wasm` を Vite 7 + React 19 に統合する方法の調査
- **参照先**:
  - [vite-plugin-wasm npm](https://www.npmjs.com/package/vite-plugin-wasm) — Vite 2.x〜7.x に対応
  - [GitHub: Menci/vite-plugin-wasm](https://github.com/Menci/vite-plugin-wasm) — wasm-pack 生成モジュールをサポート
- **調査結果**:
  - `vite-plugin-wasm` は Vite 7 に対応済み
  - `vite-plugin-top-level-await` も併用しないと `top-level await` が含まれるコードでビルドエラーが発生する
  - wasm-pack の `bundler` ターゲット（デフォルト）で生成した pkg を `file:` 参照でインストールする方法が最もシンプル
  - 設定例: `vite.config.ts` に `plugins: [react(), wasm(), topLevelAwait()]`
- **影響**: `vite.config.ts` の更新と `package.json` への 2 つの開発依存追加が必要

### WASM 同期実行とローディング表示

- **背景**: 要件 5.2「求解中にローディングインジケーター表示」を実現するための方法調査
- **調査結果**:
  - wasm-bindgen が生成する `solve()` 関数は JavaScript から同期的に呼ばれる
  - 同期処理中は React の再レンダリングが発生しないため、ローディング状態を先に setState しても画面に反映されない
  - `await new Promise(resolve => setTimeout(resolve, 0))` を求解前に挿入することで、React が 1 tick 内にローディング状態を描画してから WASM を呼び出せる
  - 大規模パズル（50×50 超）では UI がブロックされる可能性があるが、Web Worker への移行は将来拡張とする
- **影響**: `useNonogramStore.solve()` は `async` 関数とし、WASM 呼び出し前に `await setTimeout(0)` を挿入する

### グリッドドラッグ操作の実装

- **背景**: 要件 3.3「ドラッグで複数セルに同じ操作を適用」の実装方法調査
- **参照先**:
  - [Pointer Events without libraries (Medium, 2026)](https://medium.com/@aswathyraj/how-i-built-drag-and-drop-in-react-without-libraries-using-pointer-events-a0f96843edb7)
- **調査結果**:
  - ノノグラムのセルグリッドは「セルの並び替え」ではなく「セルのトグル」が目的なので dnd-kit 等の DnD ライブラリは不要
  - Pointer Events API（`onPointerDown` / `onPointerEnter` / `onPointerUp`）で十分に実装できる
  - `element.setPointerCapture(pointerId)` を `onPointerDown` で呼び出すことで、ポインタがセル外に出ても `onPointerMove` を継続受信できる
  - ただしセル間の移動検知には `onPointerEnter` を各セルで購読する方が直感的
- **影響**: 外部ライブラリ追加不要。`GridDrawingPanel` がポインタイベントを直接処理する

### JSON ファイルインポート/エクスポート

- **背景**: 要件 4.1〜4.4, 6.2 のファイル I/O 実装方法調査
- **調査結果**:
  - ダウンロード: `URL.createObjectURL(new Blob([json], { type: 'application/json' }))` + 仮想 `<a>` 要素の `.click()` が標準パターン
  - インポート: `<input type="file" accept=".json">` + `FileReader.readAsText()` が標準パターン
  - 使用後は `URL.revokeObjectURL()` でメモリリークを防ぐ
- **影響**: `PuzzleIOService` モジュールに標準ブラウザ API のみで実装できる。外部依存不要

---

## アーキテクチャパターン評価

| 選択肢 | 説明 | 強み | リスク/制限 | 備考 |
|--------|------|------|-------------|------|
| Provider + Custom Hook + Service Layer | WasmProvider でコンテキスト提供、useNonogramStore で状態集約、PuzzleIOService でI/O分離 | 外部依存なし、React イディオム準拠、テスト容易 | 大規模化時に状態フックが肥大化するリスク | 小規模SPA向けに最適 |
| Zustand による状態管理 | 外部状態管理ライブラリ使用 | DevTools、サブスクリプション最適化 | 依存追加、学習コスト | 現スケールでは過剰 |
| Redux Toolkit | Flux アーキテクチャ | 予測可能な状態遷移 | ボイラープレート多大 | このSPAには過剰 |

**選択**: Provider + Custom Hook + Service Layer。外部ライブラリなし、React 標準パターン、テスト容易性が高い。

---

## 設計決定

### 決定: Vite WASM プラグイン構成

- **背景**: Vite 7 は標準で WASM ESM Integration をサポートしていない
- **検討した選択肢**:
  1. `vite-plugin-wasm` + `vite-plugin-top-level-await` — 実績あり、Vite 7 対応済み
  2. `?init` URL サフィックス (Vite ビルトイン) — 実験的、wasm-bindgen の生成コードとの互換性不明
- **選択したアプローチ**: `vite-plugin-wasm` + `vite-plugin-top-level-await` を devDependencies に追加
- **根拠**: wasm-pack 生成モジュールの確実なサポートが確認されている
- **トレードオフ**: 2 つの devDependency 追加が必要
- **フォローアップ**: WASM pkg ビルド前にインストールエラーが発生する可能性があるため CI で順序を確認する

### 決定: 状態管理に useReducer を使用

- **背景**: パズル状態・求解状態・入力モードを一元管理する方法
- **検討した選択肢**:
  1. `useReducer` + `useContext` — React 標準、外部依存なし
  2. Zustand — 軽量外部ライブラリ、DevTools 対応
- **選択したアプローチ**: `useReducer` による `useNonogramStore` カスタムフック
- **根拠**: 外部依存追加なし、steering の「Bun + minimal deps」方針に合致
- **トレードオフ**: Redux DevTools 相当のデバッグツールがない

### 決定: WASM 呼び出し前の setTimeout(0) 遅延

- **背景**: 同期 WASM 実行時に React ローディング状態が描画されない問題への対処
- **検討した選択肢**:
  1. `setTimeout(0)` — シンプル、0 コスト
  2. Web Worker + WASM — 真の非同期、UI ブロックなし
- **選択したアプローチ**: `setTimeout(0)` による 1 tick 遅延
- **根拠**: 現状の対象パズルサイズ（〜30×30）では実用的。Web Worker 移行は将来拡張
- **トレードオフ**: 非常に大きなパズルでは UI が一時的にフリーズする可能性がある

### 決定: nonogram-wasm のローカル pkg 参照

- **背景**: wasm-pack ビルド成果物を web アプリから参照する方法
- **検討した選択肢**:
  1. `"file:../../crates/nonogram-wasm/pkg"` — 開発時に直接参照
  2. npm publish / monorepo workspace — CI での自動公開が必要
- **選択したアプローチ**: `file:` 参照
- **根拠**: モノリポ内の完結した開発サイクルで十分。`wasm-pack build` 後に `bun install` で反映される
- **フォローアップ**: CI で `build-wasm` ジョブを `build-web` より先に実行する順序を確認する

---

## リスクと軽減策

- **WASM バイナリ未ビルド時のインポートエラー** — CI の `build-wasm` ジョブを `build-web` の前提条件にする
- **大規模パズルでの UI フリーズ** — 将来的な Web Worker 移行パスを設計に残しておく
- **グリッドの再描画パフォーマンス** — 各セルを `React.memo` でメモ化し、不要な再レンダリングを防ぐ
- **TypeScript と wasm-bindgen 型定義の不整合** — wasm-pack が生成する `.d.ts` を型のソースとして使用し、手動での型定義は行わない

---

## 参照

- [vite-plugin-wasm](https://github.com/Menci/vite-plugin-wasm) — Vite 7 WASM 統合プラグイン
- [vite-plugin-top-level-await](https://github.com/Menci/vite-plugin-top-level-await) — Top-level await サポート
- [wasm-bindgen ガイド](https://rustwasm.github.io/docs/wasm-bindgen/) — Rust/WASM JS バインディング
- [Pointer Events API (MDN)](https://developer.mozilla.org/docs/Web/API/Pointer_events) — ドラッグ操作実装の基礎
- `docs/repository-plan.md` — JSON フォーマット仕様（セクション 4.1〜4.3）
