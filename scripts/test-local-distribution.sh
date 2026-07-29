#!/bin/sh
set -eu

script_directory=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/gifmp4-distribution-test.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM

package_directory="$temporary_directory/deployed/gifmp4"
empty_path_directory="$temporary_directory/empty-path"
fixture_directory="$temporary_directory/fixtures"
mkdir -p "$empty_path_directory" "$fixture_directory"

"$script_directory/package-local.sh" "$package_directory"

test -x "$package_directory/bin/gifmp4"
test -x "$package_directory/libexec/ffmpeg"
test -f "$package_directory/licenses/gifmp4-MIT.txt"
test -f "$package_directory/licenses/FFmpeg-LICENSE.txt"
test -f "$package_directory/licenses/FFmpeg-NOTICE.md"

doctor_output=$(PATH="$empty_path_directory" "$package_directory/bin/gifmp4" doctor 2>&1)
printf '%s\n' "$doctor_output"
printf '%s\n' "$doctor_output" | grep -F "$package_directory/libexec/ffmpeg" >/dev/null
printf '%s\n' "$doctor_output" | grep -F "FFmpeg is available." >/dev/null

"$package_directory/libexec/ffmpeg" \
    -loglevel error \
    -f lavfi \
    -i "color=c=red:s=16x16:d=0.1" \
    -frames:v 1 \
    -y \
    "$fixture_directory/input.gif"

PATH="$empty_path_directory" \
    "$package_directory/bin/gifmp4" \
    convert \
    "$fixture_directory/input.gif" \
    "$fixture_directory/output.mp4"

test -s "$fixture_directory/output.mp4"

echo "Local distribution test passed."
echo "The packaged gifmp4 used libexec/ffmpeg with no ffmpeg available in PATH."
