#!/bin/sh
set -eu

repository=${GIFMP4_REPOSITORY:-tetsugo02/gifmp4}
requested_version=${GIFMP4_VERSION:-latest}
install_base=${GIFMP4_INSTALL_BASE:-"$HOME/.local/share/gifmp4"}
bin_directory=${GIFMP4_BIN_DIR:-"$HOME/.local/bin"}

fail() {
    echo "gifmp4 installer: $*" >&2
    exit 1
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

sha256_file() {
    if command_exists sha256sum; then
        sha256sum "$1" | awk '{print $1}'
    elif command_exists shasum; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        fail "sha256sum or shasum is required"
    fi
}

detect_target() {
    operating_system=$(uname -s)
    architecture=$(uname -m)

    case "$operating_system-$architecture" in
        Darwin-arm64) echo "darwin-arm64" ;;
        Darwin-x86_64) echo "darwin-x64" ;;
        Linux-x86_64) echo "linux-x64" ;;
        *) fail "unsupported platform: $operating_system-$architecture" ;;
    esac
}

resolve_version() {
    if [ "$requested_version" != "latest" ]; then
        printf '%s\n' "${requested_version#v}"
        return
    fi

    latest_url=$(curl -fsSL -o /dev/null -w '%{url_effective}' \
        "https://github.com/$repository/releases/latest")
    latest_tag=${latest_url##*/}
    case "$latest_tag" in
        v*) printf '%s\n' "${latest_tag#v}" ;;
        *) fail "could not determine the latest release version" ;;
    esac
}

configure_path() {
    case ":$PATH:" in
        *":$bin_directory:"*) return ;;
    esac

    case "${SHELL:-}" in
        */zsh) shell_configuration="$HOME/.zshrc" ;;
        */bash) shell_configuration="$HOME/.bashrc" ;;
        *) shell_configuration="$HOME/.profile" ;;
    esac

    if [ "$bin_directory" = "$HOME/.local/bin" ]; then
        path_line='export PATH="$HOME/.local/bin:$PATH"'
    else
        escaped_bin_directory=$(printf '%s' "$bin_directory" |
            sed 's/\\/\\\\/g; s/"/\\"/g; s/\$/\\$/g; s/`/\\`/g')
        path_line="export PATH=\"$escaped_bin_directory:\$PATH\""
    fi

    if [ -f "$shell_configuration" ] &&
        grep -F "$path_line" "$shell_configuration" >/dev/null 2>&1; then
        return
    fi

    {
        printf '\n'
        printf '%s\n' '# Added by the gifmp4 installer'
        printf '%s\n' "$path_line"
    } >>"$shell_configuration"
    echo "Updated PATH in $shell_configuration"
}

command_exists curl || fail "curl is required"
command_exists tar || fail "tar is required"

case "$install_base" in
    ""|/) fail "unsafe install directory: $install_base" ;;
esac
case "$bin_directory" in
    ""|/) fail "unsafe bin directory: $bin_directory" ;;
esac

target=$(detect_target)
version=$(resolve_version)
case "$version" in
    ""|[!0-9]*|*[!0-9A-Za-z.+-]*) fail "invalid release version: $version" ;;
esac
tag="v$version"
archive_name="gifmp4-$version-$target.tar.gz"
release_url="https://github.com/$repository/releases/download/$tag"
archive_url=${GIFMP4_ARCHIVE_URL:-"$release_url/$archive_name"}
checksum_url=${GIFMP4_CHECKSUM_URL:-"$release_url/SHA256SUMS"}

temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/gifmp4-install.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM
archive_path="$temporary_directory/$archive_name"
checksum_path="$temporary_directory/SHA256SUMS"

echo "Downloading gifmp4 $version for $target..."
curl -fsSL "$archive_url" -o "$archive_path"

if [ -n "${GIFMP4_ARCHIVE_SHA256:-}" ]; then
    expected_checksum=$GIFMP4_ARCHIVE_SHA256
else
    curl -fsSL "$checksum_url" -o "$checksum_path"
    expected_checksum=$(awk -v archive="$archive_name" \
        '$2 == archive { print $1 }' "$checksum_path")
fi
[ -n "$expected_checksum" ] || fail "checksum not found for $archive_name"

actual_checksum=$(sha256_file "$archive_path")
[ "$actual_checksum" = "$expected_checksum" ] ||
    fail "checksum verification failed for $archive_name"

tar -tzf "$archive_path" | while IFS= read -r entry; do
    case "$entry" in
        /*|..|../*|*/..|*/../*) fail "archive contains an unsafe path: $entry" ;;
        gifmp4|gifmp4/|gifmp4/*) ;;
        *) fail "archive contains an unexpected path: $entry" ;;
    esac
done

tar -xzf "$archive_path" -C "$temporary_directory"
[ -x "$temporary_directory/gifmp4/bin/gifmp4" ] ||
    fail "archive does not contain bin/gifmp4"
[ -x "$temporary_directory/gifmp4/libexec/ffmpeg" ] ||
    fail "archive does not contain libexec/ffmpeg"

versions_directory="$install_base/versions"
version_directory="$versions_directory/$version"
mkdir -p "$versions_directory" "$bin_directory"

launcher="$bin_directory/gifmp4"
if [ -e "$launcher" ] && [ ! -L "$launcher" ]; then
    fail "$launcher already exists and is not a symbolic link"
fi

if [ -e "$version_directory" ]; then
    [ -x "$version_directory/bin/gifmp4" ] ||
        fail "existing installation is incomplete: $version_directory"
    [ -x "$version_directory/libexec/ffmpeg" ] ||
        fail "existing installation is incomplete: $version_directory"
    echo "gifmp4 $version is already installed; updating the launcher."
else
    mv "$temporary_directory/gifmp4" "$version_directory"
fi

ln -sfn "$version_directory/bin/gifmp4" "$launcher"

configure_path

echo "Installed gifmp4 $version to $version_directory"
echo "Executable: $launcher"
case ":$PATH:" in
    *":$bin_directory:"*) echo "Run: gifmp4 doctor" ;;
    *)
        echo "PATH will be available in new shells. For this shell, run:"
        echo "  export PATH=\"$bin_directory:\$PATH\""
        ;;
esac
