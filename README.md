# gifmp4

FFmpegを利用してGIFとMP4を相互変換する、Rust製のコマンドラインツールです。
単一ファイルの変換に加え、globパターンを使った一括変換にも対応しています。

## 主な機能

- GIFからMP4への変換
- MP4からGIFへの変換
- 入力拡張子に応じた出力形式と出力パスの自動決定
- `*`、`**`、`?`、`[...]`を使った一括変換
- 一括変換時のディレクトリ構造の維持
- MP4からGIFへ変換する際のFPSと出力幅の指定
- GIFからMP4へ変換する際のH.264品質指定
- 既存ファイルの確認なしでの上書き
- FFmpegの利用可否を確認する診断コマンド

## 必要なもの

- Rust（ソースからビルドする場合）
- FFmpeg
- `curl`と`unzip`（同梱パッケージを作成する場合）

FFmpegは次の優先順位で探索されます。

1. 環境変数`GIFMP4_FFMPEG_PATH`で指定された実行ファイル
2. `gifmp4`実行ファイルのパッケージに同梱された`libexec/ffmpeg`
3. システムの`PATH`にある`ffmpeg`

## ビルド

```bash
cargo build --release
```

ビルドされた実行ファイルは次に生成されます。

```text
target/release/gifmp4
```

開発中に直接実行する場合は、`cargo run --`の後ろに引数を指定します。

```bash
cargo run -- doctor
```

## FFmpeg同梱パッケージの作成

配布先にFFmpegがインストールされていなくても使えるよう、次の構成のローカル配布パッケージを作成できます。

```text
gifmp4/
├── bin/
│   └── gifmp4
├── libexec/
│   └── ffmpeg
└── licenses/
    ├── FFmpeg-NOTICE.md
    └── ffmpeg/
        └── LICENSE・COPYINGファイル
```

対象OS・CPU向けに固定されたFFmpeg 8.1.2をダウンロードしてSHA-256を検証し、`dist/gifmp4`へ出力します。

```bash
./scripts/package-local.sh
```

別の出力先を指定することもできます。既存のディレクトリは誤上書きを防ぐため受け付けません。

```bash
./scripts/package-local.sh /tmp/gifmp4
```

現在対応している対象：

- `darwin-arm64`: Apple Silicon搭載Mac
- `darwin-x64`: Intel搭載Mac
- `linux-x64`: x86_64 Linux

FFmpegのURL、バージョン、チェックサム、ライセンスURLは[`packaging/ffmpeg-artifacts.tsv`](packaging/ffmpeg-artifacts.tsv)で一元管理しています。`latest` URLは使用しないため、配布元の更新によって意図せずFFmpegが変わることはありません。

バージョン付きのリリースアーカイブを作成する場合：

```bash
./scripts/package-release.sh darwin-arm64
```

出力例：

```text
dist/gifmp4-0.1.0-darwin-arm64.tar.gz
```

### ローカル配布テスト

次のスクリプトは一時ディレクトリへパッケージを作成し、FFmpegが`PATH`に存在しない状態で以下を検証します。

- `bin/gifmp4 doctor`が`libexec/ffmpeg`を発見する
- 同梱FFmpegを使って実際にGIFからMP4へ変換できる
- 必要なバイナリとライセンスディレクトリが存在する

```bash
./scripts/test-local-distribution.sh
```

### 実際に配布する際の注意

同梱FFmpegは[Martin Riedl's FFmpeg Build Server](https://ffmpeg.martin-riedl.de/)のFFmpeg 8.1.2固定リリースから取得します。配布物にはgifmp4のMITライセンス、FFmpegのGPLv3ライセンス、第三者配布に関する注意書きが含まれます。

FFmpegと静的リンクされたライブラリにはGPLなどの条件が適用されます。同梱ファイルだけで配布者のすべての義務が自動的に満たされることを保証するものではありません。公開前に、対象バイナリのビルド情報、ソースコード提供義務、特許などの条件を確認してください。

## 単一ファイルの変換

基本構文：

```text
gifmp4 convert <INPUT_PATH> [OUTPUT_PATH] [OPTIONS]
```

### GIFからMP4

```bash
gifmp4 convert animation.gif
```

出力パスを省略すると、入力ファイルと同じ場所に`animation.mp4`が生成されます。

出力パスとH.264の品質を指定する例：

```bash
gifmp4 convert animation.gif video.mp4 --quality 18
```

`--quality`はH.264のCRF値です。`0`から`51`まで指定でき、値が小さいほど高品質かつファイルサイズが大きくなります。デフォルトは`23`です。

### MP4からGIF

```bash
gifmp4 convert video.mp4
```

出力パスを省略すると、入力ファイルと同じ場所に`video.gif`が生成されます。

FPSと出力幅を指定する例：

```bash
gifmp4 convert video.mp4 animation.gif --fps 15 --width 640
```

- `--fps <FPS>`: GIFのフレームレート。デフォルトは`30`
- `--width <PIXELS>`: GIFの出力幅。縦横比は維持される

GIFのパレットは変換時に生成され、ディザリングを適用して出力されます。

## 複数ファイルの一括変換

基本構文：

```text
gifmp4 batch <PATTERN> [OPTIONS]
```

カレントディレクトリにあるすべてのGIFを変換する例：

```bash
gifmp4 batch '*.gif'
```

サブディレクトリを再帰的に検索する例：

```bash
gifmp4 batch 'media/**/*.gif'
```

globパターンは必ずクォートしてください。クォートしない場合、`gifmp4`が受け取る前にシェルがパターンを展開することがあります。

### 出力先とディレクトリ構造

`--output-dir`を省略した場合、各入力ファイルと同じディレクトリに変換結果を出力します。

```bash
gifmp4 batch 'media/**/*.gif'
```

出力先を指定すると、パターンの固定部分を基準にサブディレクトリ構造を維持します。

```bash
gifmp4 batch 'media/**/*.gif' --output-dir converted
```

たとえば、次の入力：

```text
media/animals/cat.gif
media/landscapes/mountain.gif
```

は次のように出力されます。

```text
converted/animals/cat.mp4
converted/landscapes/mountain.mp4
```

必要な出力ディレクトリは自動的に作成されます。

### 一括変換時のエラー

- パターンに一致するファイルが1件もない場合はエラーになる
- あるファイルの変換に失敗しても、残りのファイルの処理は続行される
- すべての処理が終わった後、失敗件数と各ファイルのエラーがまとめて表示される
- 1件以上失敗した場合、コマンド全体はエラー終了する

`batch`でも`--fps`、`--width`、`--quality`を指定できます。

```bash
gifmp4 batch 'videos/**/*.mp4' \
  --output-dir gifs \
  --fps 12 \
  --width 480
```

## 上書き動作

FFmpegには`-y`を指定しているため、出力先に同名ファイルが存在する場合は確認なしで上書きされます。重要なファイルを出力先に指定しないよう注意してください。

入力ファイルと出力ファイルに同じパスを指定することはできません。

## FFmpegのパスを指定する

システムの`PATH`以外にあるFFmpegを利用する場合は、`GIFMP4_FFMPEG_PATH`を指定します。

```bash
GIFMP4_FFMPEG_PATH=/path/to/ffmpeg gifmp4 doctor
```

変換時にも同じ環境変数が利用されます。

## 環境の診断

FFmpegが見つかり、実行できるかを確認します。

```bash
gifmp4 doctor
```

このコマンドは`gifmp4`のバージョン、実行ファイルの場所、FFmpeg候補の場所、FFmpegのバージョン情報を表示します。

## ヘルプ

利用可能なサブコマンドとオプションは、組み込みヘルプでも確認できます。

```bash
gifmp4 --help
gifmp4 convert --help
gifmp4 batch --help
```

## 開発

コードをフォーマットします。

```bash
cargo fmt
```

テストを実行します。

```bash
cargo test
```

Clippyで静的解析を実行します。

```bash
cargo clippy --all-targets -- -D warnings
```

## CI/CD

GitHub ActionsのCIは、`main`へのpush、Pull Request、手動実行で動作します。

- フォーマット確認
- 全テスト
- Clippy
- macOS arm64、macOS x64、Linux x64向け配布アーカイブの作成
- 同梱FFmpegを使ったローカル配布テスト
- ビルド成果物のArtifactへの保存

`Cargo.toml`のバージョンと一致するタグをpushすると、自動的にGitHub Releaseを作成します。

```bash
git tag v0.1.0
git push origin v0.1.0
```

Releaseには次のアーカイブが添付されます。

```text
gifmp4-0.1.0-darwin-arm64.tar.gz
gifmp4-0.1.0-darwin-x64.tar.gz
gifmp4-0.1.0-linux-x64.tar.gz
```

## ライセンス

gifmp4本体は[MIT License](LICENSE)で提供されます。

配布物に同梱されるFFmpegは別のソフトウェアであり、FFmpegおよび静的リンクされたライブラリのライセンス条件が適用されます。詳細は配布物の`licenses/`を確認してください。

## 対応する変換

| 入力 | 出力 |
| --- | --- |
| `.gif` | `.mp4` |
| `.mp4` | `.gif` |

拡張子の大文字と小文字は区別されません。これ以外の形式、または入出力が同じ形式になる組み合わせはサポートされていません。
