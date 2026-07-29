#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <release-archive>" >&2
    exit 2
fi

script_directory=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
project_directory=$(CDPATH= cd -- "$script_directory/.." && pwd)
archive_path=$(CDPATH= cd -- "$(dirname -- "$1")" && pwd)/$(basename -- "$1")
archive_name=$(basename -- "$archive_path")
version=$(printf '%s\n' "$archive_name" |
    sed -n 's/^gifmp4-\([0-9][^-]*\)-.*\.tar\.gz$/\1/p')

[ -f "$archive_path" ] || {
    echo "Release archive not found: $archive_path" >&2
    exit 1
}
[ -n "$version" ] || {
    echo "Could not determine version from: $archive_name" >&2
    exit 1
}

if command -v sha256sum >/dev/null 2>&1; then
    archive_checksum=$(sha256sum "$archive_path" | awk '{print $1}')
else
    archive_checksum=$(shasum -a 256 "$archive_path" | awk '{print $1}')
fi

temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/gifmp4-installer-test.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM
test_home="$temporary_directory/home"
install_base="$test_home/.local/share/gifmp4"
bin_directory="$test_home/.local/bin"
checksum_path="$temporary_directory/SHA256SUMS"
mkdir -p "$test_home"
printf '%s  %s\n' "$archive_checksum" "$archive_name" >"$checksum_path"

run_installer() {
    HOME="$test_home" \
        SHELL=/bin/zsh \
        GIFMP4_VERSION="$version" \
        GIFMP4_INSTALL_BASE="$install_base" \
        GIFMP4_BIN_DIR="$bin_directory" \
        GIFMP4_ARCHIVE_URL="file://$archive_path" \
        GIFMP4_CHECKSUM_URL="file://$checksum_path" \
        sh "$project_directory/install.sh"
}

run_installer
run_installer

test -L "$bin_directory/gifmp4"
test -x "$install_base/versions/$version/bin/gifmp4"
test -x "$install_base/versions/$version/libexec/ffmpeg"
test "$(grep -Fc 'export PATH="$HOME/.local/bin:$PATH"' "$test_home/.zshrc")" -eq 1

doctor_output=$(PATH="$bin_directory" "$bin_directory/gifmp4" doctor 2>&1)
printf '%s\n' "$doctor_output"
resolved_version_directory=$(CDPATH= cd -- "$install_base/versions/$version" && pwd -P)
printf '%s\n' "$doctor_output" |
    grep -F "$resolved_version_directory/libexec/ffmpeg" >/dev/null

echo "Installer test passed."
