#!/bin/sh
set -eu

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <darwin-arm64|darwin-x64|linux-x64> <output-directory>" >&2
    exit 2
fi

target=$1
output_directory=$2
script_directory=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
project_directory=$(CDPATH= cd -- "$script_directory/.." && pwd)
manifest="$project_directory/packaging/ffmpeg-artifacts.tsv"

binary_url=
binary_sha256=
license_url=
license_sha256=
archive_format=
ffmpeg_version=

while IFS='|' read -r manifest_target manifest_ffmpeg_version manifest_archive_format manifest_binary_url manifest_binary_sha256 manifest_license_url manifest_license_sha256; do
    case "$manifest_target" in
        ""|\#*) continue ;;
    esac

    if [ "$manifest_target" = "$target" ]; then
        ffmpeg_version=$manifest_ffmpeg_version
        archive_format=$manifest_archive_format
        binary_url=$manifest_binary_url
        binary_sha256=$manifest_binary_sha256
        license_url=$manifest_license_url
        license_sha256=$manifest_license_sha256
        break
    fi
done < "$manifest"

if [ -z "$binary_url" ]; then
    echo "Unsupported distribution target: $target" >&2
    exit 1
fi

verify_sha256() {
    file=$1
    expected=$2

    if command -v sha256sum >/dev/null 2>&1; then
        actual=$(sha256sum "$file")
        actual=${actual%% *}
    elif command -v shasum >/dev/null 2>&1; then
        actual=$(shasum -a 256 "$file")
        actual=${actual%% *}
    else
        echo "sha256sum or shasum is required." >&2
        exit 1
    fi

    if [ "$actual" != "$expected" ]; then
        echo "SHA-256 verification failed: $file" >&2
        echo "expected: $expected" >&2
        echo "actual:   $actual" >&2
        exit 1
    fi
}

temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/gifmp4-ffmpeg.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM

binary_archive="$temporary_directory/ffmpeg.$archive_format"
license_file="$temporary_directory/FFmpeg-LICENSE.txt"

echo "Downloading fixed FFmpeg $ffmpeg_version for $target..."
curl --fail --location --silent --show-error --output "$binary_archive" "$binary_url"
curl --fail --location --silent --show-error --output "$license_file" "$license_url"

verify_sha256 "$binary_archive" "$binary_sha256"
verify_sha256 "$license_file" "$license_sha256"

mkdir -p "$output_directory/licenses"
case "$archive_format" in
    gz)
        gzip -dc "$binary_archive" > "$output_directory/ffmpeg"
        ;;
    zip)
        unzip -p "$binary_archive" ffmpeg > "$output_directory/ffmpeg"
        ;;
    *)
        echo "Unsupported FFmpeg archive format: $archive_format" >&2
        exit 1
        ;;
esac
cp "$license_file" "$output_directory/licenses/FFmpeg-LICENSE.txt"
chmod 755 "$output_directory/ffmpeg"

echo "FFmpeg $ffmpeg_version downloaded and verified for $target."
