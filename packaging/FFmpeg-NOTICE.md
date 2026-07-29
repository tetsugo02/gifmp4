# FFmpeg distribution notice

The `libexec/ffmpeg` executable in this package is a fixed FFmpeg build.
The macOS packages use FFmpeg 8.1.2 provided by Martin Riedl's FFmpeg Build
Server:

https://ffmpeg.martin-riedl.de/

The Linux x64 package uses the statically linked FFmpeg 7.0.2 binary from the
`eugeneware/ffmpeg-static` b6.1.1 GitHub Release:

https://github.com/eugeneware/ffmpeg-static/releases/tag/b6.1.1

FFmpeg itself is a separate project and is available from:

https://ffmpeg.org/

This build enables GPL and version 3 components, including libx264. The
accompanying `FFmpeg-LICENSE.txt` contains the GNU General Public License,
version 3. The build also statically links third-party libraries whose
applicable terms and source-code requirements must be reviewed by the package
distributor.

The exact URLs and SHA-256 checksums used to produce this package are recorded
in `packaging/ffmpeg-artifacts.tsv` in the gifmp4 source repository.
