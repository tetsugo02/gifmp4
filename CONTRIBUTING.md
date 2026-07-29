# gifmp4へのコントリビューション

バグ報告、改善提案、ドキュメント修正、コード変更を歓迎します。この文書では、ローカル開発からPull Request、配布とリリースまでの手順を説明します。

## 開発に必要なもの

- Git
- Rustのstable toolchain
- FFmpeg（`cargo run`で変換や`doctor`を実行する場合）
- `curl`
- `unzip`
- `tar`

Linux配布物を作成する場合は、musl toolchain、`musl-tools`、`readelf`、Dockerも必要です。gifmp4本体は`x86_64-unknown-linux-musl`向けにビルドされます。

配布パッケージ関連のスクリプトは、対応するOS・CPU向けの固定されたFFmpegをダウンロードします。そのため、実行時にはネットワーク接続が必要です。

対応する配布ターゲット：

- `darwin-arm64`: Apple Silicon搭載Mac
- `darwin-x64`: Intel搭載Mac
- `linux-x64`: x86_64 Linux

クロスコンパイルには対応していません。配布パッケージは対象と同じOS・CPU上で作成してください。

## 開発環境を準備する

リポジトリをcloneして、プロジェクトへ移動します。

```bash
git clone https://github.com/tetsugo02/gifmp4.git
cd gifmp4
```

デバッグビルドを作成します。

```bash
cargo build --locked
```

開発中のバイナリは`cargo run --`に続けて引数を渡して実行できます。

```bash
cargo run -- doctor
cargo run -- convert animation.gif
cargo run -- batch 'media/**/*.gif' --output-dir converted
```

## 変更を作成する

作業用ブランチを作成してください。

```bash
git switch -c fix/short-description
```

変更時には次の点を確認してください。

- 既存のコマンドやデフォルト動作との互換性を考慮する
- 不具合修正や新しい動作には、可能な限りテストを追加する
- 利用方法が変わる場合は`README.md`も更新する
- 配布や開発手順が変わる場合は`CONTRIBUTING.md`も更新する
- FFmpeg取得元を変更する場合は、チェックサムとライセンスも確認する

## コード品質を確認する

フォーマットを確認します。

```bash
cargo fmt -- --check
```

すべてのRustテストを実行します。

```bash
cargo test --locked
```

Clippyを警告なしで実行します。

```bash
cargo clippy --locked --all-targets -- -D warnings
```

変更を送る前に、上記3コマンドがすべて成功することを確認してください。

## 配布パッケージをテストする

gifmp4とFFmpegを次の構成でローカルにパッケージできます。

```text
gifmp4/
├── bin/
│   └── gifmp4
├── libexec/
│   └── ffmpeg
└── licenses/
    ├── gifmp4-MIT.txt
    ├── FFmpeg-LICENSE.txt
    └── FFmpeg-NOTICE.md
```

現在の環境向けに`dist/gifmp4`を作成します。

```bash
./scripts/package-local.sh
```

別の出力先も指定できます。

```bash
./scripts/package-local.sh /tmp/gifmp4
```

既存の出力ディレクトリは上書きされません。再実行する場合は、不要になった出力を確認してから削除するか、別の出力先を指定してください。

### 同梱FFmpegを使った変換テスト

システムの`PATH`にあるFFmpegを利用せず、パッケージ内のFFmpegだけで診断とGIFからMP4への変換を行います。

```bash
./scripts/test-local-distribution.sh
```

### Releaseアーカイブとインストーラーのテスト

現在のバージョンとターゲット名を含むReleaseアーカイブを作成します。

```bash
./scripts/package-release.sh darwin-arm64
```

生成されたアーカイブを使い、隔離された一時HOMEへインストールして動作を確認します。

```bash
./scripts/test-install.sh dist/gifmp4-<VERSION>-darwin-arm64.tar.gz
```

`<VERSION>`は`Cargo.toml`のバージョンに置き換えてください。

Linux版では、gifmp4とFFmpegに動的ELFインタープリタがないことを確認し、Ubuntu 20.04コンテナ内で`doctor`を実行します。

```bash
./scripts/test-linux-compatibility.sh dist/gifmp4-<VERSION>-linux-x64.tar.gz
```

## FFmpegの固定バージョンを更新する

FFmpegのURL、バージョン、アーカイブのSHA-256、ライセンスURLは[`packaging/ffmpeg-artifacts.tsv`](packaging/ffmpeg-artifacts.tsv)で管理しています。

更新時には、すべてのターゲットについて次を確認してください。

1. URLがバージョン固定であり、`latest`など可変のURLではない
2. ダウンロードしたアーカイブのSHA-256が一致する
3. FFmpegのビルド設定と有効なライブラリを確認する
4. 再配布に必要なライセンスと通知を更新する
5. ローカル配布テストとGitHub Actionsが成功する

macOS版FFmpegは[Martin Riedl's FFmpeg Build Server](https://ffmpeg.martin-riedl.de/)から、Linux x64版の静的FFmpegは[`eugeneware/ffmpeg-static`](https://github.com/eugeneware/ffmpeg-static/releases/tag/b6.1.1)の固定GitHub Releaseから取得しています。配布物にはgifmp4のMITライセンス、FFmpegのGPLv3ライセンス、第三者配布に関する注意書きが含まれます。

FFmpegや静的リンクされたライブラリの配布条件が、同梱ファイルだけですべて満たされるとは限りません。公開前にビルド情報、対応するソースコードの提供義務、特許などの条件を確認してください。

## Pull Requestを作成する

1. 変更内容が分かる単位でコミットする
2. 作業ブランチを自分のリポジトリへpushする
3. `main`ブランチ宛てにPull Requestを作成する
4. 変更理由、主な変更内容、実行したテストを本文に記載する
5. GitHub Actionsの完了を確認する

Pull Requestでは次のCIが実行されます。

- Rustfmt
- 全Rustテスト
- Clippy
- macOS arm64、macOS x64、Linux x64向け配布アーカイブの作成
- 同梱FFmpegによる実変換
- Linux実行ファイルの静的リンク確認とUbuntu 20.04での起動
- `curl`インストーラーの実行

## リリース手順

この節はリポジトリのリリース担当者向けです。

1. `Cargo.toml`の`version`を更新する
2. `Cargo.lock`を更新し、品質チェックと配布テストを実行する
3. バージョン変更を`main`へ反映する
4. `Cargo.toml`と一致する`v`付きタグをpushする

例：

```bash
git tag -a v0.1.1 -m "Release v0.1.1"
git push origin v0.1.1
```

タグのバージョンが`Cargo.toml`と一致しない場合、Release workflowは失敗します。

タグをpushするとGitHub Actionsが各ターゲットをビルド・テストし、GitHub Releaseへ次のファイルを公開します。

```text
gifmp4-<VERSION>-darwin-arm64.tar.gz
gifmp4-<VERSION>-darwin-x64.tar.gz
gifmp4-<VERSION>-linux-x64.tar.gz
SHA256SUMS
install.sh
```

Release本文はGitHubの自動生成リリースノートを利用します。
