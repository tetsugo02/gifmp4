# gifmp4

FFmpegを利用してGIFとMP4を相互変換するコマンドラインツールです。単一ファイルの変換と、globパターンによる一括変換に対応しています。

## クイックスタート

macOSまたはx86_64 Linuxでは、次の1行で最新版をインストールできます。

```bash
curl -fsSL https://github.com/tetsugo02/gifmp4/releases/latest/download/install.sh | sh && export PATH="$HOME/.local/bin:$PATH"
```

インストールできたことを確認します。

```bash
gifmp4 doctor
```

GIFをMP4へ変換します。

```bash
gifmp4 convert animation.gif
```

同じディレクトリに`animation.mp4`が作成されます。MP4からGIFへ変換する場合も、入力ファイルを指定するだけです。

```bash
gifmp4 convert video.mp4
```

## インストール

インストーラーはOSとCPUを自動判定し、GitHub Releaseから対応するアーカイブを取得します。アーカイブは`SHA256SUMS`で検証され、FFmpegも一緒にインストールされます。システムへFFmpegを別途インストールする必要はありません。

対応環境：

- Apple Silicon搭載Mac
- Intel搭載Mac
- x86_64 Linux

ファイルは`~/.local/share/gifmp4`へ配置され、`~/.local/bin/gifmp4`から実行できるようになります。`~/.local/bin`が`PATH`にない場合、インストーラーが使用中のシェル設定へ追記します。

特定バージョンをインストールする場合：

```bash
curl -fsSL https://github.com/tetsugo02/gifmp4/releases/download/v0.1.1/install.sh | GIFMP4_VERSION=0.1.1 sh
```

## 単一ファイルを変換する

基本構文：

```text
gifmp4 convert <INPUT_PATH> [OUTPUT_PATH] [OPTIONS]
```

対応する変換は次の2種類です。

| 入力 | 出力 |
| --- | --- |
| `.gif` | `.mp4` |
| `.mp4` | `.gif` |

拡張子の大文字と小文字は区別されません。出力パスを省略すると、入力ファイルと同じディレクトリに拡張子だけを変更して出力します。

### GIFからMP4へ変換する

```bash
gifmp4 convert animation.gif
```

出力ファイル名とH.264の品質を指定できます。

```bash
gifmp4 convert animation.gif video.mp4 --quality 18
```

`--quality`はH.264のCRF値です。指定できる範囲は`0`から`51`で、値が小さいほど高品質になり、ファイルサイズも大きくなります。デフォルトは`23`です。

### MP4からGIFへ変換する

```bash
gifmp4 convert video.mp4
```

フレームレートと出力幅を指定できます。

```bash
gifmp4 convert video.mp4 animation.gif --fps 15 --width 640
```

| オプション | 内容 | デフォルト |
| --- | --- | --- |
| `--fps <FPS>` | GIFのフレームレート | `30` |
| `--width <PIXELS>` | GIFの出力幅。縦横比は維持される | 入力幅 |

GIF用のカラーパレットは変換時に生成され、ディザリングを適用して出力されます。

## 複数ファイルを一括変換する

`batch`へglobパターンを渡します。

```text
gifmp4 batch <PATTERN> [OPTIONS]
```

カレントディレクトリのすべてのGIFを変換する例：

```bash
gifmp4 batch '*.gif'
```

サブディレクトリを含めて再帰的に変換する例：

```bash
gifmp4 batch 'media/**/*.gif'
```

使用できるパターンは`*`、`**`、`?`、`[...]`です。シェルによる事前展開を防ぐため、パターンは必ずクォートしてください。

### 出力先を指定する

`--output-dir`を省略すると、それぞれの入力ファイルと同じディレクトリに出力します。出力先を指定した場合は、パターンの固定部分を基準にサブディレクトリ構造を維持します。

```bash
gifmp4 batch 'media/**/*.gif' --output-dir converted
```

たとえば、次の入力：

```text
media/animals/cat.gif
media/landscapes/mountain.gif
```

は次の場所へ出力されます。

```text
converted/animals/cat.mp4
converted/landscapes/mountain.mp4
```

必要な出力ディレクトリは自動的に作成されます。

### 変換オプションを指定する

`batch`でも`--fps`、`--width`、`--quality`を利用できます。

```bash
gifmp4 batch 'videos/**/*.mp4' \
  --output-dir gifs \
  --fps 12 \
  --width 480
```

### 一部のファイルで変換に失敗した場合

- パターンに一致するファイルが1件もない場合はエラーになる
- あるファイルの変換に失敗しても、残りの変換は続行される
- 全処理の終了後に、失敗件数と各ファイルのエラーがまとめて表示される
- 1件以上失敗した場合、コマンド全体はエラー終了する

## 上書き動作

出力先に同名ファイルがある場合は、確認なしで上書きします。重要なファイルを出力先に指定しないでください。

入力ファイルと出力ファイルに同じパスを指定することはできません。

## FFmpegを確認・指定する

インストール状態と、使用されるFFmpegを確認できます。

```bash
gifmp4 doctor
```

FFmpegは次の順序で探索されます。

1. `GIFMP4_FFMPEG_PATH`で指定された実行ファイル
2. gifmp4と一緒にインストールされた`libexec/ffmpeg`
3. システムの`PATH`にある`ffmpeg`

任意のFFmpegを使用する場合：

```bash
GIFMP4_FFMPEG_PATH=/path/to/ffmpeg gifmp4 convert animation.gif
```

## ヘルプ

利用可能なコマンドとオプションは、組み込みヘルプで確認できます。

```bash
gifmp4 --help
gifmp4 convert --help
gifmp4 batch --help
```

## アンインストール

```bash
rm "$HOME/.local/bin/gifmp4"
rm -rf "$HOME/.local/share/gifmp4"
```

シェル設定ファイルに追加された`# Added by the gifmp4 installer`と、その次のPATH設定行も削除してください。

## コントリビューション

開発環境の準備、テスト、Pull Request、配布パッケージとリリースの手順は[CONTRIBUTING.md](CONTRIBUTING.md)を参照してください。

## ライセンス

gifmp4本体は[MIT License](LICENSE)で提供されます。同梱されるFFmpegには、FFmpegおよび静的リンクされたライブラリのライセンス条件が適用されます。詳細は配布物の`licenses/`を確認してください。
