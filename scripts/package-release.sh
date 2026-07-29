#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <darwin-arm64|darwin-x64|linux-x64>" >&2
    exit 2
fi

target=$1
script_directory=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
project_directory=$(CDPATH= cd -- "$script_directory/.." && pwd)
version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$project_directory/Cargo.toml" | head -n 1)

if [ -z "$version" ]; then
    echo "Could not determine the package version from Cargo.toml." >&2
    exit 1
fi

distribution_name="gifmp4-$version-$target"
staging_directory="$project_directory/dist/$distribution_name"
package_directory="$staging_directory/gifmp4"
archive_path="$project_directory/dist/$distribution_name.tar.gz"

if [ -e "$staging_directory" ] || [ -e "$archive_path" ]; then
    echo "Distribution output already exists for $target." >&2
    exit 1
fi

GIFMP4_PACKAGE_TARGET="$target" "$script_directory/package-local.sh" "$package_directory"
tar -C "$staging_directory" -czf "$archive_path" gifmp4

echo "Release archive created: $archive_path"
