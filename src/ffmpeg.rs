use anyhow::{Context, Result, bail};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

pub fn find_ffmpeg_path() -> Result<PathBuf> {
    if let Some(configured_path) = std::env::var_os("GIFMP4_FFMPEG_PATH") {
        let configured_ffmpeg_path = PathBuf::from(configured_path);

        if !configured_ffmpeg_path.is_file() {
            bail!(
                "GIFMP4_FFMPEG_PATHで指定されたFFmpegが見つかりません: {}",
                configured_ffmpeg_path.display()
            );
        }

        return Ok(configured_ffmpeg_path);
    }

    let bundled_ffmpeg_path = bundled_ffmpeg_candidate()?;

    if bundled_ffmpeg_path.is_file() {
        return Ok(bundled_ffmpeg_path);
    }

    let ffmpeg_filename = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };

    Ok(PathBuf::from(ffmpeg_filename))
}

pub fn execute_ffmpeg(arguments: &[OsString]) -> Result<()> {
    let ffmpeg_path = find_ffmpeg_path()?;

    let exit_status = Command::new(&ffmpeg_path)
        .args(arguments)
        .status()
        .with_context(|| {
            format!(
                "FFmpegを起動できませんでした: {}\n\
                 FFmpegがインストールされ、PATHが設定されているか確認してください",
                ffmpeg_path.display()
            )
        })?;

    if !exit_status.success() {
        bail!(
            "FFmpegによる変換に失敗しました。終了コード: {:?}",
            exit_status.code()
        );
    }

    Ok(())
}

pub fn doctor() -> Result<()> {
    let ffmpeg_path = find_ffmpeg_path()?;

    println!("gifmp4 version: {}", env!("CARGO_PKG_VERSION"));
    println!("gifmp4 executable: {}", std::env::current_exe()?.display());
    println!("FFmpeg candidate: {}", ffmpeg_path.display());
    println!();

    let exit_status = Command::new(&ffmpeg_path)
        .arg("-version")
        .status()
        .with_context(|| format!("FFmpegを起動できませんでした: {}", ffmpeg_path.display()))?;

    if !exit_status.success() {
        bail!(
            "FFmpegの確認に失敗しました。終了コード: {:?}",
            exit_status.code()
        );
    }

    println!();
    println!("FFmpeg is available.");

    Ok(())
}

fn bundled_ffmpeg_candidate() -> Result<PathBuf> {
    let current_executable_path =
        std::env::current_exe().context("現在の実行ファイルのパスを取得できませんでした")?;
    let resolved_executable_path =
        std::fs::canonicalize(&current_executable_path).unwrap_or(current_executable_path);

    bundled_ffmpeg_candidate_for(&resolved_executable_path)
}

fn bundled_ffmpeg_candidate_for(current_executable_path: &std::path::Path) -> Result<PathBuf> {
    let executable_directory = current_executable_path
        .parent()
        .context("実行ファイルの親ディレクトリを取得できませんでした")?;

    let package_directory = executable_directory
        .parent()
        .context("パッケージディレクトリを取得できませんでした")?;

    let ffmpeg_filename = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };

    Ok(package_directory.join("libexec").join(ffmpeg_filename))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn finds_ffmpeg_next_to_the_package_bin_directory() {
        let executable = if cfg!(target_os = "windows") {
            Path::new("package/bin/gifmp4.exe")
        } else {
            Path::new("package/bin/gifmp4")
        };
        let expected = if cfg!(target_os = "windows") {
            PathBuf::from("package/libexec/ffmpeg.exe")
        } else {
            PathBuf::from("package/libexec/ffmpeg")
        };

        assert_eq!(bundled_ffmpeg_candidate_for(executable).unwrap(), expected);
    }
}
