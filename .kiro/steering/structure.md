# プロジェクト構造

## 組織方針

**役割別分離**: `crates/`（再利用可能なライブラリ）と `apps/`（エンドユーザー向けアプリ）を明確に分ける。

## ディレクトリパターン

### ライブラリクレート (`crates/`)
**目的**: 共有可能なビジネスロジック・データ型・バインディング
**例**:
- `nonogram-core/` — ソルバロジック（フォーマット依存なし）
- `nonogram-format/` — JSON入出力の型と変換
- `nonogram-wasm/` — WASMエクスポート

### アプリケーション (`apps/`)
**目的**: ユーザー向けエントリーポイント。クレートを組み合わせて機能を提供する
**例**:
- `apps/cli/` — Rustバイナリ（clapベースCLI）
- `apps/desktop/` — Tauriアプリ（Rustバックエンド + Reactフロントエンド）
- `apps/web/` — Vite + React SPA（WASMを直接利用）

### ドキュメント (`docs/`)
**目的**: 設計文書・アーキテクチャ決定記録・アルゴリズム調査

## 依存関係のルール

```
nonogram-core <── nonogram-wasm <── apps/web
              <── apps/cli
              <── apps/desktop/src-tauri

nonogram-format <── nonogram-wasm
                <── apps/cli
                <── apps/desktop/src-tauri
```

- `nonogram-core` → `nonogram-format` の依存は禁止

## 命名規則

- **Rustクレート**: `kebab-case`（例: `nonogram-core`）
- **Rust型・トレイト**: `PascalCase`（例: `SolveResult`, `Solver`）
- **TypeScriptファイル**: `PascalCase`（コンポーネント）、`camelCase`（ユーティリティ）

## Frontendのインポート規則

```typescript
// 外部ライブラリ
import React from 'react'

// 内部モジュール（相対パス）
import { Component } from './Component'
```

## フロントエンド構成

デスクトップとWebはともに `src/` にReactソースを置き、`vite.config.ts` でビルド設定を行う。

### apps/web のソース構成パターン

```
src/
  utils/        # 純粋関数ユーティリティ（clueParseUtils, clueComputeUtils など）
  services/     # 副作用を伴うサービス層（PuzzleIOService など）
  contexts/     # React Context + Provider（WasmContext など）
  hooks/        # カスタムフック（useNonogramStore など）
  components/   # UIコンポーネント（PascalCase）
```

- ユーティリティ・サービスのテストは各ディレクトリ内の `__tests__/` サブディレクトリに配置する

---
_パターンを記録する。ファイルツリーの列挙ではない。新しいファイルがパターンに従う限り、更新不要。_
