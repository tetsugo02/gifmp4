#!/bin/sh
set -eu

script_directory=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
project_directory=$(CDPATH= cd -- "$script_directory/.." && pwd)
output_directory=${1:-"$project_directory/dist/gifmp4"}

case "$output_directory" in
    /|""|"$project_directory")
        echo "Refusing unsafe output directory: $output_directory" >&2
        exit 1
        ;;
esac

if [ -e "$output_directory" ]; then
    echo "Output already exists: $output_directory" >&2
    echo "Remove it or specify a different output directory." >&2
    exit 1
fi

detect_target() {
    operating_system=$(uname -s)
    architecture=$(uname -m)

    case "$operating_system-$architecture" in
        Darwin-arm64) echo "darwin-arm64" ;;
        Darwin-x86_64) echo "darwin-x64" ;;
        Linux-x86_64) echo "linux-x64" ;;
        *)
            echo "Unsupported host: $operating_system-$architecture" >&2
            exit 1
            ;;
    esac
}

host_target=$(detect_target)
package_target=${GIFMP4_PACKAGE_TARGET:-"$host_target"}

if [ "$package_target" != "$host_target" ]; then
    echo "Cross packaging is not supported: host=$host_target target=$package_target" >&2
    exit 1
fi

temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/gifmp4-package.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM
package_directory="$temporary_directory/gifmp4"
download_directory="$temporary_directory/download"

"$script_directory/fetch-ffmpeg.sh" "$package_target" "$download_directory"

mkdir -p "$package_directory/bin" "$package_directory/libexec" "$package_directory/licenses"

case "$package_target" in
    linux-x64)
        rust_target=x86_64-unknown-linux-musl
        cargo build \
            --release \
            --locked \
            --target "$rust_target" \
            --manifest-path "$project_directory/Cargo.toml"
        gifmp4_binary="$project_directory/target/$rust_target/release/gifmp4"
        ;;
    darwin-arm64|darwin-x64)
        cargo build --release --locked --manifest-path "$project_directory/Cargo.toml"
        gifmp4_binary="$project_directory/target/release/gifmp4"
        ;;
    *)
        echo "Unsupported package target: $package_target" >&2
        exit 1
        ;;
esac

cp "$gifmp4_binary" "$package_directory/bin/gifmp4"
cp "$download_directory/ffmpeg" "$package_directory/libexec/ffmpeg"
cp "$project_directory/LICENSE" "$package_directory/licenses/gifmp4-MIT.txt"
cp "$project_directory/packaging/FFmpeg-NOTICE.md" "$package_directory/licenses/FFmpeg-NOTICE.md"
cp "$download_directory/licenses/FFmpeg-LICENSE.txt" "$package_directory/licenses/FFmpeg-LICENSE.txt"

chmod 755 "$package_directory/bin/gifmp4" "$package_directory/libexec/ffmpeg"
mkdir -p "$(dirname -- "$output_directory")"
mv "$package_directory" "$output_directory"

trap - EXIT HUP INT TERM
rm -rf "$temporary_directory"

echo "Local distribution created: $output_directory"
echo "Target: $package_target"
