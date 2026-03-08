# 要件定義書

## はじめに

`nonokit` は、nonogram パズルの解法・テンプレート生成・画像変換・グリッド→パズル変換をすべて JSON インターフェースで提供するコマンドラインツールである。`apps/cli` クレートとして実装され、`nonogram-core` および `nonogram-format` クレートに依存する。対象機能は `docs/repository-plan.md`（solve / template コマンド）、`docs/figure_converter_spec.md`（convert コマンド：画像→ドットグリッド変換）、および Grid→Puzzle 変換コマンドの三領域からなる。

---

## 要件

### 要件 1: solve コマンド

**目的:** パズル設計者・研究者として、JSON 形式のパズルファイルをソルバーに渡して解答 JSON を得たい。そうすることで、手動計算なしにパズルの解を確認できる。

#### 受け入れ基準

1. When `nonokit solve --input <path>` が実行されたとき、the CLI shall 指定 JSON ファイルをパズルとして読み込む。
2. When `--input` オプションが省略されたとき、the CLI shall 標準入力から JSON を読み込む。
3. When パズルの解が求まったとき、the CLI shall 解答 JSON（`{"status": "...", "solutions": [...]}` 形式）を標準出力に出力する。
4. When `--solver csp` が指定されたとき、the CLI shall `CspSolver` を使用して解く。
5. When `--solver probing` が指定されたとき、the CLI shall `ProbingSolver` を使用して解く。
6. When `--solver` オプションが省略されたとき、the CLI shall `CspSolver` をデフォルトソルバーとして使用する。
7. When 解が存在しないとき、the CLI shall `{"status": "none", "solutions": []}` を出力する。
8. When 唯一解が存在するとき、the CLI shall `{"status": "unique", "solutions": [<grid>]}` を出力する。
9. When 複数解が存在するとき、the CLI shall `{"status": "multiple", "solutions": [<grid1>, <grid2>]}` を出力する（少なくとも 2 例）。
10. If 入力ファイルが存在しないか読み込めないとき、the CLI shall エラーメッセージを標準エラー出力に表示し、ゼロでない終了コードで終了する。
11. If 入力 JSON が不正な形式であるとき、the CLI shall 解析エラーメッセージを標準エラー出力に表示し、ゼロでない終了コードで終了する。
12. When `--output <path>` オプションが指定されたとき、the CLI shall 解答 JSON をファイルに書き込む（標準出力への出力は行わない）。

---

### 要件 2: template コマンド

**目的:** パズル制作者として、指定サイズの空パズルテンプレート JSON を生成したい。そうすることで、クルー入力の出発点として使えるファイルを素早く作成できる。

#### 受け入れ基準

1. When `nonokit template --rows <N> --cols <M>` が実行されたとき、the CLI shall `row_clues` に長さ N の空配列リスト、`col_clues` に長さ M の空配列リストを持つ JSON を生成する。
2. When `--output <path>` が指定されたとき、the CLI shall テンプレート JSON を指定ファイルに書き込む。
3. When `--output` が省略されたとき、the CLI shall テンプレート JSON を標準出力に出力する。
4. If `--rows` または `--cols` に 1 未満の値が指定されたとき、the CLI shall エラーメッセージを標準エラー出力に表示し、ゼロでない終了コードで終了する。
5. The CLI shall 生成するテンプレートが `docs/repository-plan.md` Section 4.3 で定義された Problem Template Format に準拠することを保証する。

---

### 要件 3: convert コマンド（画像→ドットグリッド）

**目的:** ノノグラム制作者として、ラスター画像を処理してドットグリッド JSON を生成したい。そうすることで、手描き画像から自動的にノノグラムの素材を作成できる。

#### 受け入れ基準

1. When `nonokit convert --input <image-path>` が実行されたとき、the CLI shall 画像ファイルを読み込み、ドットグリッド JSON を生成する。
2. When 入力画像にアルファチャンネルが含まれるとき、the CLI shall 白背景でアルファコンポジットを行ってからグレースケール変換を行う（計算式: `result = rgb * (alpha/255) + white * (1 - alpha/255)`）。
3. The CLI shall グレースケール変換 → ガウスぼかし → Canny エッジ検出 → エッジ合成 → セル平均ダウンサンプリング → 閾値処理 → ノイズ除去の順で処理パイプラインを実行する。
4. When `--smooth-strength <value>`（範囲: 0–5、デフォルト: 1.0）が指定されたとき、the CLI shall sigma = smooth_strength のガウスぼかしを適用する（0 のとき無効）。
5. When `--edge-strength <value>`（範囲: 0–1、デフォルト: 0.3）が指定されたとき、the CLI shall Canny エッジ検出（low=50, high=150）を実行し、`clamp(gray - edge_map * edge_strength, 0, 255)` でエッジを合成する（0 のとき無効）。
6. When `--grid-width <W>` と `--grid-height <H>`（各範囲: 5–50、デフォルト: 20）が指定されたとき、the CLI shall 画像を W×H セルのグリッドに分割し、各セルの平均輝度を計算する。
7. When `--threshold <value>`（範囲: 0–255、デフォルト: 128）が指定されたとき、the CLI shall セル平均輝度が threshold 未満のセルを filled（`true`）、以上のセルを blank（`false`）に分類する。
8. When `--noise-removal <value>`（範囲: 0–20、デフォルト: 0）が指定されたとき、the CLI shall 4 連結で面積が noise_removal 未満の filled 連結成分を blank に変換する（0 のとき無効）。
9. The CLI shall セル平均計算において、グリッドに均等に収まらない端数ピクセル（右端・下端の余り）を除外する。
10. When 変換が完了したとき、the CLI shall ドットグリッド JSON（`[[true/false, ...], ...]` 形式、行優先 2D boolean 配列、SolveResult の solution グリッドと同一フォーマット）を標準出力または `--output <path>` で指定したファイルに出力する。
11. If 入力ファイルが存在しないか対応していない画像形式のとき、the CLI shall エラーメッセージを標準エラー出力に表示し、ゼロでない終了コードで終了する。
12. If `--grid-width` または `--grid-height` が範囲外のとき、the CLI shall エラーメッセージを標準エラー出力に表示し、ゼロでない終了コードで終了する。

---

### 要件 4: grid-to-puzzle コマンド（ドットグリッド→パズル）

**目的:** ノノグラム制作者として、ドットグリッド JSON からパズルの行・列クルーを自動生成したい。そうすることで、画像変換結果を直接ソルバーに渡せるパズル JSON に変換できる。

#### 受け入れ基準

1. When `nonokit grid-to-puzzle --input <grid-json-path>` が実行されたとき、the CLI shall ドットグリッド JSON（`[[true/false, ...], ...]` 形式、SolveResult の solution グリッドと同一フォーマット）を読み込む。
2. When `--input` が省略されたとき、the CLI shall 標準入力からドットグリッド JSON を読み込む。
3. When グリッドが読み込まれたとき、the CLI shall 各行について filled セルの連続した群のサイズ列を行クルーとして計算する。
4. When グリッドが読み込まれたとき、the CLI shall 各列について filled セルの連続した群のサイズ列を列クルーとして計算する。
5. When 行または列にすべて blank のセルしかないとき、the CLI shall その行・列のクルーを空配列 `[]` とする。
6. When 変換が完了したとき、the CLI shall パズル JSON（`{"row_clues": [...], "col_clues": [...]}` 形式）を標準出力または `--output <path>` で指定したファイルに出力する。
7. If 入力 JSON が不正な形式または 2D boolean 配列として解析できないとき、the CLI shall エラーメッセージを標準エラー出力に表示し、ゼロでない終了コードで終了する。
8. If 配列内の行の長さが一致しない（不正な矩形形状）とき、the CLI shall エラーメッセージを標準エラー出力に表示し、ゼロでない終了コードで終了する。

---

### 要件 5: 共通 I/O・ヘルプ・終了コード仕様

**目的:** CLI ユーザーとして、すべてのコマンドに統一されたインターフェースを持ちたい。そうすることで、コマンドをパイプラインで組み合わせて活用できる。

#### 受け入れ基準

1. The CLI shall サブコマンド `solve`, `template`, `convert`, `grid-to-puzzle` をすべてサポートする。
2. When `nonogram-cli --help` が実行されたとき、the CLI shall 利用可能なサブコマンドと説明を標準出力に表示する。
3. When 各サブコマンドに `--help` が指定されたとき、the CLI shall そのサブコマンドのオプション一覧と説明を標準出力に表示する。
4. When コマンドが正常に完了したとき、the CLI shall 終了コード 0 で終了する。
5. When コマンドがエラーで終了するとき、the CLI shall ゼロでない終了コードで終了する。
6. The CLI shall すべてのエラーメッセージを標準エラー出力（stderr）に出力し、標準出力（stdout）には結果 JSON のみを出力する。
7. The CLI shall `clap` ライブラリを使用して引数パースを実装する。
8. Where `solve` と `grid-to-puzzle` の両コマンドが `--input` を省略できる場合、the CLI shall 標準入力からのパイプを許容することで `nonokit convert | nonokit grid-to-puzzle | nonokit solve` のようなパイプライン構成を可能にする。
