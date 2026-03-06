# nonogram-engine
Nonogram solvers and applications

## 開発セットアップ

### 必要なツール

- [Rust](https://rustup.rs/)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [Bun](https://bun.sh/)

### 初回セットアップ

クローン後、**まず WASM パッケージをビルドする必要があります**。これを行わないとWebアプリの求解機能が動作しません。

```bash
# 1. WASMパッケージをビルド（bundlerターゲット必須）
wasm-pack build --target bundler crates/nonogram-wasm

# 2. Webアプリの依存関係をインストール
cd apps/web
bun install

# 3. 開発サーバーを起動
bun run dev
```

### WASMを変更した場合

`crates/nonogram-wasm` 以下のRustコードを変更したときは、手順1〜2を再実行してください。

```bash
wasm-pack build --target bundler crates/nonogram-wasm
cd apps/web && bun install
```

## License

This project is licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
