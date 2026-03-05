# 技術スタック

## アーキテクチャ

Cargo workspaceによる単一リポジトリ。Rustライブラリ層とTypeScript/Reactフロントエンド層の2層構造。

## コア技術

- **言語（バックエンド）**: Rust (edition 2024)
- **言語（フロントエンド）**: TypeScript ~5.8+
- **フロントエンドフレームワーク**: React 19 + Vite 7
- **デスクトップフレームワーク**: Tauri 2
- **JSパッケージマネージャ**: Bun（npmではなくbunを使用）

## 主要ライブラリ

- `clap` — CLIの引数パース
- `serde` / `serde_json` — JSONシリアライゼーション
- `wasm-pack` — WASMビルドツール
- `@tauri-apps/api` — TauriフロントエンドAPI

## 開発規約

### Rust
- パブリックAPIはすべてAmerican Englishでドキュメント化する
- 各ソルバは `Solver` トレイトを実装する
- テストカバレッジ目標: `nonogram-core` ≥ 80%（cargo-llvm-cov）

#### モジュール構成（mod.rs を使わない方法）

**古い方法（非推奨）**:
```
src/
  solver/
    mod.rs       ← 避ける
    line.rs
    cell.rs
```

**現代の方法（推奨）**:
```
src/
  solver.rs      ← モジュールのエントリポイント（mod.rs の代わり）
  solver/
    line.rs
    cell.rs
```

`solver.rs` の中でサブモジュールを宣言する:
```rust
// src/solver.rs
pub mod line;
pub mod cell;
```

**AIが間違えやすいパターン**:

| ❌ 間違い | ✅ 正しい |
|---|---|
| `src/solver/mod.rs` を作る | `src/solver.rs` を作る |
| ディレクトリ内に `mod.rs` を置く | ディレクトリと同名の `.rs` ファイルをその親に置く |
| `src/lib.rs` + `src/solver/mod.rs` | `src/lib.rs` + `src/solver.rs` + `src/solver/` |

`mod.rs` 方式はRust 2018以降は非推奨。新規ファイル作成時は必ず `モジュール名.rs` + 同名ディレクトリの構成を使うこと。

### TypeScript
- TypeScript strict mode
- ESLintによる静的解析（`apps/web`）

### テスト
- Rust: `cargo test --workspace`
- 単体テストは各クレートと同一ファイル内（`#[cfg(test)]`）

## 開発コマンド

```bash
# Rust全体テスト
cargo test --workspace

# WASMビルド
wasm-pack build crates/nonogram-wasm

# Webアプリ開発
cd apps/web && bun install && bun run dev

# デスクトップアプリ開発
cd apps/desktop && bun install && bun run tauri dev

# CLIビルド
cargo build -p cli --release
```

## 主要技術決定

- **nonogram-coreはnonogram-formatに依存しない**: 変換責務はアプリ/バインディング層に置く
- **Webは完全クライアントサイド**: WASMでバックエンドなし動作
- **全UIで共通JSONフォーマット**: `nonogram-format`クレートで一元管理

---
_標準とパターンを記録する。依存ライブラリの網羅リストではない。_
