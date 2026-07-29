#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <linux-x64-release-archive>" >&2
    exit 2
fi

archive_path=$(CDPATH= cd -- "$(dirname -- "$1")" && pwd)/$(basename -- "$1")
[ -f "$archive_path" ] || {
    echo "Release archive not found: $archive_path" >&2
    exit 1
}

command -v docker >/dev/null 2>&1 || {
    echo "docker is required for the old-Linux compatibility test." >&2
    exit 1
}
command -v readelf >/dev/null 2>&1 || {
    echo "readelf is required for the static-linking check." >&2
    exit 1
}

temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/gifmp4-linux-test.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM
tar -xzf "$archive_path" -C "$temporary_directory"

gifmp4_binary="$temporary_directory/gifmp4/bin/gifmp4"
ffmpeg_binary="$temporary_directory/gifmp4/libexec/ffmpeg"

for binary in "$gifmp4_binary" "$ffmpeg_binary"; do
    if readelf -l "$binary" | grep -q INTERP; then
        echo "Dynamic ELF interpreter found: $binary" >&2
        exit 1
    fi
done

docker run --rm \
    --platform linux/amd64 \
    --volume "$archive_path:/tmp/gifmp4.tar.gz:ro" \
    ubuntu:20.04 \
    sh -eu -c '
        mkdir -p /tmp/package /tmp/empty-path
        tar -xzf /tmp/gifmp4.tar.gz -C /tmp/package

        gifmp4=/tmp/package/gifmp4/bin/gifmp4
        ffmpeg=/tmp/package/gifmp4/libexec/ffmpeg

        test -x "$gifmp4"
        test -x "$ffmpeg"

        PATH=/tmp/empty-path "$gifmp4" doctor
    '

echo "Linux compatibility test passed on Ubuntu 20.04."
echo "Both gifmp4 and the bundled FFmpeg started without a system FFmpeg."
