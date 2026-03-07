# 要件定義書

## はじめに

Web アプリの目的を「Nonogram の問題作成支援アプリ」に方針転換する。従来のソルバ入力UIを廃止し、白黒ドット絵エディタを中心とした問題作成ワークフローを提供する。ユーザーはドット絵を描き（または画像から変換し）、ソルバで一意解を確認し、問題をエクスポートするという一連の流れを1つのアプリで完結できるようにする。

**スコープ**: 本スペックの実装対象は `apps/web` および関連する Rust クレートのみ。Desktop App（`apps/desktop`）は現時点では未実装のため改修対象外とする。将来 Desktop App を実装する際に、本スペックと同等の仕様を適用すること。

変更の影響範囲:
- `crates/nonogram-core` — 画像→グリッド変換ロジックの追加
- `crates/nonogram-wasm` — 変換機能の WASM エクスポート追加
- `crates/nonogram-format` — Grid の JSON スキーマ定義追加
- `apps/web` — UI 全面再構成、新機能実装

---

## 要件

### 要件 1: ドット絵エディタ

**Objective:** 問題作成者として、白黒のドット絵を直感的に編集できるようにしたい。それにより、Nonogram の元になる絵を自由に描けるようにする。

#### 受け入れ基準

1. The Nonogram Maker App shall グリッドサイズ（幅・高さ）をユーザーが指定して新規グリッドを作成できる機能を提供する。
2. When ユーザーがグリッドのセルをクリックまたはドラッグする, the Nonogram Maker App shall そのセルの状態を塗りつぶし（黒）と空白（白）でトグルする。
3. While ドラッグ操作が継続中, the Nonogram Maker App shall ドラッグ開始時のセル状態変化方向（塗りつぶし/消去）を維持してカーソルが通過したセルすべてに同じ操作を適用する。
4. The Nonogram Maker App shall 全セルをリセット（すべて空白）する機能を提供する。
5. The Nonogram Maker App shall 現在のグリッド状態に基づいて行・列のヒント（クルー）をリアルタイムで表示する。
6. When ユーザーが操作を行う, the Nonogram Maker App shall undo/redo 機能によって操作を1ステップずつ取り消し・やり直しできる。

---

### 要件 2: 画像からドット絵への変換機能（Convert 機能）

**Objective:** 問題作成者として、既存の画像ファイルを元にドット絵グリッドを自動生成したい。それにより、手描きより効率よく絵のベースを作成できるようにする。

#### 受け入れ基準

1. When ユーザーが画像ファイルを読み込む, the Nonogram Maker App shall PNG・JPEG・WebP・GIF 等の一般的なラスター画像フォーマットを受け付ける。
2. When アルファチャンネルを持つ画像が読み込まれる, the Nonogram Maker App shall 白背景にアルファ合成してから処理する（`result = rgb * (alpha/255) + white * (1 - alpha/255)`）。
3. When 画像が読み込まれる, the Nonogram Maker App shall 384×384 px の境界ボックスに収まるようアスペクト比を保ったままリサイズし、リサイズ済み画像をキャッシュして以降のパラメータ変更に再利用する。384 px 未満の画像はスケールアップしない。
   - When リサイズ後の画像の幅が 50 px を下回る, the Nonogram Maker App shall `grid_width` の上限をリサイズ後の画像幅（px）に強制する。
   - When リサイズ後の画像の高さが 50 px を下回る, the Nonogram Maker App shall `grid_height` の上限をリサイズ後の画像高さ（px）に強制する。
4. The Nonogram Maker App shall 以下のパラメータをスライダーで調整できるUIを提供する:
   - `grid_width` (5–50, デフォルト 20): 列数
   - `grid_height` (5–50, デフォルト 20): 行数
   - `smooth_strength` (0–5, デフォルト 1.0): ガウシアンブラーのシグマ値
   - `threshold` (0–255, デフォルト 128): 明度しきい値
   - `edge_strength` (0–1, デフォルト 0.3): エッジ強調係数
   - `noise_removal` (0–20, デフォルト 0): 最小連結領域サイズ（セル数）
5. When いずれかのパラメータが変更される, the Nonogram Maker App shall グレースケール変換 → ガウシアンブラー → Canny エッジ検出 → エッジマージ → セル平均ダウンサンプリング → しきい値処理 → ノイズ除去の順でパイプラインを実行し、リアルタイムにドットグリッドプレビューを更新する。
6. The Nonogram Maker App shall 変換元の画像プレビューと生成されたドットグリッドプレビューを並べて表示する。
7. When ユーザーが変換結果を承認する, the Nonogram Maker App shall 生成されたグリッドをエディタに読み込み、引き続き手動編集できる状態にする。
8. If 画像の読み込みに失敗する, the Nonogram Maker App shall 読み込み失敗のメッセージをユーザーに表示する。

---

### 要件 3: ソルバーによる一意解検証

**Objective:** 問題作成者として、作成したドット絵から生成される Nonogram 問題が一意解を持つか確認したい。それにより、解けるパズルとして成立しているかを確認しながら問題を完成させられるようにする。

#### 受け入れ基準

1. When ユーザーがソルバー検証を起動する, the Nonogram Maker App shall 現在のグリッドから行・列のクルーを計算してソルバーに渡す。
2. While ソルバーが実行中, the Nonogram Maker App shall 処理中であることを示すインジケーターを表示し、UIをブロックしない。
3. When ソルバーが唯一解と判定する, the Nonogram Maker App shall その旨をユーザーに通知し、解答グリッドを表示する。
4. When ソルバーが複数解と判定する, the Nonogram Maker App shall 複数解である旨を通知し、解答のうち最大2つを参考として表示する。
5. When ソルバーが解なしと判定する, the Nonogram Maker App shall 解なしである旨をユーザーに表示する。
6. If ソルバーの実行中にエラーが発生する, the Nonogram Maker App shall エラーメッセージを表示してエディタ操作に戻れる状態を維持する。

---

### 要件 4: 問題のエクスポート

**Objective:** 問題作成者として、完成した Nonogram 問題を複数のフォーマットで出力したい。それにより、他のツールや媒体で問題を利用できるようにする。

#### 受け入れ基準

1. When ユーザーが JSON エクスポートを選択する, the Nonogram Maker App shall `nonogram-format` クレートのスキーマに準拠した JSON ファイルをダウンロードする。
2. When ユーザーが画像エクスポートを選択する, the Nonogram Maker App shall 行・列のクルーと問題グリッドを視覚的に表現した PNG 画像ファイルをダウンロードする。
3. The Nonogram Maker App shall エクスポート可能な状態（グリッドが空でないこと）をチェックし、条件を満たさない場合はエクスポートボタンを無効化またはエラーを表示する。
4. Where デスクトップアプリとして動作する場合, the Nonogram Maker App shall ファイル保存ダイアログを通じてエクスポート先を指定できる。

---

### 要件 5: Grid の JSON スキーマ定義（nonogram-format）

**Objective:** ライブラリ利用者として、`nonogram-format` クレートで Grid の構造を統一した JSON スキーマとして参照したい。それにより、アプリ間・ツール間でグリッドデータを一貫した形式でやり取りできるようにする。

#### 受け入れ基準

1. The nonogram-format crate shall Grid（セルの2次元配列）を表す Rust 型および対応する JSON スキーマを提供する。
2. The nonogram-format crate shall Grid の JSON 表現に行数・列数・各セルの塗りつぶし状態（bool または 0/1）を含める。
3. The nonogram-format crate shall Grid の JSON を Rust 型にデシリアライズ、および Rust 型を JSON にシリアライズする機能を `serde` を用いて提供する。
4. When Grid の JSON が不正なフォーマットである, the nonogram-format crate shall デシリアライズ時に明確なエラーを返す。

---

### 要件 6: 画像→Grid 変換機能（nonogram-core / nonogram-wasm）

**Objective:** アプリ開発者として、画像データとパラメータを与えるだけで Grid を取得できる API を利用したい。それにより、変換ロジックをアプリ層に書くことなく再利用できるようにする。

#### 受け入れ基準

1. The nonogram-core crate shall 画像バイト列（またはピクセルデータ）と変換パラメータ（`grid_width`, `grid_height`, `smooth_strength`, `threshold`, `edge_strength`, `noise_removal`）を受け取り、binary Grid を返す関数を提供する。
2. The nonogram-core crate shall 要件 2 の処理パイプライン（グレースケール → ブラー → エッジ検出 → エッジマージ → ダウンサンプリング → しきい値 → ノイズ除去）をこの順に実行する。
3. The nonogram-wasm crate shall nonogram-core の画像→Grid 変換機能を WASM としてエクスポートし、JavaScript から呼び出せるようにする。
4. If 画像のデコードに失敗する, the nonogram-core crate shall `Result::Err` を返し、エラー内容を示す情報を含める。
5. The nonogram-core crate shall 画像→Grid 変換関数の単体テストを提供し、既知の入力に対して期待されるグリッドが生成されることを検証する。

---

### 要件 7: UI/UX フローの再構築（apps/web）

**Objective:** 問題作成者として、問題作成の一連のフローが自然な順序で画面に反映されたアプリを使いたい。それにより、迷わず効率よく問題を作成できるようにする。

#### 受け入れ基準

1. The Nonogram Maker App shall 問題作成ワークフロー（新規作成 → 編集 → ソルバー検証 → エクスポート）をユーザーが段階的に進められる画面フローを提供する。
2. The Nonogram Maker App shall 既存のソルバー入力UIを廃止し、グリッドエディタを主画面として提供する。
3. The Nonogram Maker App shall 画像変換（Convert）機能へのアクセス手段（ボタンまたはモーダル）を主画面から提供する。
4. The Nonogram Maker App shall 変換・編集・エクスポートの各操作をシングルページアプリ（SPA）として画面遷移なしに提供する。
5. The Nonogram Maker App shall モバイルおよびデスクトップのブラウザ幅でレスポンシブに動作する。

---

### 備考: Desktop App への将来的な適用

Desktop App（`apps/desktop`）は現時点では未実装のため、本スペックの実装対象外とする。将来 Desktop App を実装する際には、要件 1〜7 と同等の仕様（ドット絵エディタ・画像変換・ソルバー検証・エクスポート）を適用すること。その際、ネイティブのファイル保存ダイアログなど Tauri 固有の機能を活用することが望ましい。
