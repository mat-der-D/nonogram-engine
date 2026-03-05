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
