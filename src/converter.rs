use crate::ffmpeg;
use crate::media::{ConversionDirection, detect_conversion_direction};
use anyhow::{Context, Result, bail};
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
pub struct ConversionOptions {
    pub frames_per_second: u32,
    pub output_width: Option<u32>,
    pub constant_rate_factor: u8,
}

pub fn convert(
    input_path: &Path,
    output_path: Option<&Path>,
    options: &ConversionOptions,
) -> Result<()> {
    validate_conversion_request(input_path, output_path, options)?;

    let conversion_direction = detect_conversion_direction(input_path, output_path)?;
    let resolved_output_path = execute_output_path(input_path, output_path)?;

    match conversion_direction {
        ConversionDirection::GifToMp4 => {
            convert_gif_to_mp4(input_path, &resolved_output_path, options)
        }
        ConversionDirection::Mp4ToGif => {
            convert_mp4_to_gif(input_path, &resolved_output_path, options)
        }
    }
}

pub fn convert_batch(
    pattern: &str,
    output_directory: Option<&Path>,
    options: &ConversionOptions,
) -> Result<()> {
    let pattern_base = glob_base(pattern);
    let entries = glob::glob(pattern).with_context(|| format!("無効なパターンです: {pattern}"))?;
    let mut matched_file_count = 0;
    let mut failures = Vec::new();

    for entry in entries {
        let input_path = match entry {
            Ok(path) if path.is_file() => path,
            Ok(_) => continue,
            Err(error) => {
                failures.push(error.to_string());
                continue;
            }
        };
        matched_file_count += 1;

        let output_path = match output_directory {
            Some(directory) => match batch_output_path(&input_path, &pattern_base, directory) {
                Ok(path) => Some(path),
                Err(error) => {
                    failures.push(format!("{}: {error:#}", input_path.display()));
                    continue;
                }
            },
            None => None,
        };

        if let Some(parent) = output_path.as_ref().and_then(|path| path.parent())
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            failures.push(format!(
                "{}: 出力先ディレクトリを作成できませんでした: {error}",
                input_path.display()
            ));
            continue;
        }

        if let Err(error) = convert(&input_path, output_path.as_deref(), options) {
            failures.push(format!("{}: {error:#}", input_path.display()));
        }
    }

    if matched_file_count == 0 {
        bail!("パターンに一致するファイルがありません: {pattern}");
    }

    if !failures.is_empty() {
        bail!(
            "{}件中{}件の変換に失敗しました:\n{}",
            matched_file_count,
            failures.len(),
            failures.join("\n")
        );
    }

    Ok(())
}

fn glob_base(pattern: &str) -> PathBuf {
    let pattern_path = Path::new(pattern);
    let mut base = PathBuf::new();

    for component in pattern_path.components() {
        let contains_metacharacter = match component {
            Component::Normal(value) => value
                .to_string_lossy()
                .chars()
                .any(|character| matches!(character, '*' | '?' | '[')),
            _ => false,
        };

        if contains_metacharacter {
            break;
        }
        base.push(component.as_os_str());
    }

    if base.as_os_str().is_empty() {
        PathBuf::from(".")
    } else if !pattern.ends_with(std::path::MAIN_SEPARATOR) && base == pattern_path {
        base.parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    } else {
        base
    }
}

fn batch_output_path(
    input_path: &Path,
    pattern_base: &Path,
    output_directory: &Path,
) -> Result<PathBuf> {
    let relative_input_path = if pattern_base == Path::new(".") {
        input_path
            .strip_prefix(".")
            .unwrap_or(input_path)
            .to_path_buf()
    } else {
        input_path
            .strip_prefix(pattern_base)
            .with_context(|| {
                format!(
                    "{}をパターンの基準ディレクトリ{}から相対化できません",
                    input_path.display(),
                    pattern_base.display()
                )
            })?
            .to_path_buf()
    };
    let conversion_direction = detect_conversion_direction(input_path, None)?;
    let output_extension = match conversion_direction {
        ConversionDirection::GifToMp4 => "mp4",
        ConversionDirection::Mp4ToGif => "gif",
    };

    Ok(output_directory
        .join(&relative_input_path)
        .with_extension(output_extension))
}

fn validate_conversion_request(
    input_path: &Path,
    output_path: Option<&Path>,
    options: &ConversionOptions,
) -> Result<()> {
    if !input_path.is_file() {
        bail!("入力ファイルが見つかりません: {}", input_path.display());
    }

    if options.frames_per_second == 0 {
        bail!("FPSには1以上の値を指定してください");
    }

    if matches!(options.output_width, Some(0)) {
        bail!("出力幅には1以上の値を指定してください");
    }

    if options.constant_rate_factor > 51 {
        bail!("qualityには0から51までの値を指定してください");
    }

    if matches!(output_path, Some(path) if path == input_path) {
        bail!(
            "入力ファイルと出力ファイルに同じパスは指定できません: {}",
            input_path.display()
        );
    }

    let resolved_output_path = execute_output_path(input_path, output_path)?;

    if let Some(output_parent_directory) = resolved_output_path.parent()
        && !output_parent_directory.as_os_str().is_empty()
        && !output_parent_directory.is_dir()
    {
        bail!(
            "出力先ディレクトリが存在しません: {}",
            output_parent_directory.display()
        );
    }

    Ok(())
}

fn execute_output_path(
    input_path: &Path,
    output_path: Option<&Path>,
) -> Result<std::path::PathBuf> {
    let conversion_direction = detect_conversion_direction(input_path, output_path)?;

    let resolved_output_path = match output_path {
        Some(specified_output_path) => specified_output_path.to_path_buf(),
        None => match conversion_direction {
            ConversionDirection::GifToMp4 => input_path.with_extension("mp4"),
            ConversionDirection::Mp4ToGif => input_path.with_extension("gif"),
        },
    };
    Ok(resolved_output_path)
}

fn convert_gif_to_mp4(
    input_path: &Path,
    output_path: &Path,
    options: &ConversionOptions,
) -> Result<()> {
    ffmpeg::execute_ffmpeg(&gif_to_mp4_arguments(input_path, output_path, options))
}

fn gif_to_mp4_arguments(
    input_path: &Path,
    output_path: &Path,
    options: &ConversionOptions,
) -> Vec<OsString> {
    let scale_filter = "scale=trunc(iw/2)*2:trunc(ih/2)*2";

    vec![
        OsString::from("-y"),
        OsString::from("-i"),
        input_path.as_os_str().to_os_string(),
        OsString::from("-an"),
        OsString::from("-vf"),
        OsString::from(scale_filter),
        OsString::from("-c:v"),
        OsString::from("libx264"),
        OsString::from("-crf"),
        OsString::from(options.constant_rate_factor.to_string()),
        OsString::from("-pix_fmt"),
        OsString::from("yuv420p"),
        OsString::from("-movflags"),
        OsString::from("+faststart"),
        output_path.as_os_str().to_os_string(),
    ]
}

fn convert_mp4_to_gif(
    input_path: &Path,
    output_path: &Path,
    options: &ConversionOptions,
) -> Result<()> {
    ffmpeg::execute_ffmpeg(&mp4_to_gif_arguments(input_path, output_path, options))
}

fn mp4_to_gif_arguments(
    input_path: &Path,
    output_path: &Path,
    options: &ConversionOptions,
) -> Vec<OsString> {
    let scale_filter = match options.output_width {
        Some(output_width) => {
            format!("scale={output_width}:-1:flags=lanczos")
        }
        None => "scale=iw:-1:flags=lanczos".to_string(),
    };

    let filter_graph = format!(
        "[0:v]fps={frames_per_second},{scale_filter},\
         split[video_for_palette][video_for_output];\
         [video_for_palette]palettegen=max_colors=256[palette];\
         [video_for_output][palette]paletteuse=dither=sierra2_4a",
        frames_per_second = options.frames_per_second
    );

    vec![
        OsString::from("-y"),
        OsString::from("-i"),
        input_path.as_os_str().to_os_string(),
        OsString::from("-filter_complex"),
        OsString::from(filter_graph),
        OsString::from("-an"),
        OsString::from("-loop"),
        OsString::from("0"),
        output_path.as_os_str().to_os_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMPORARY_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn options() -> ConversionOptions {
        ConversionOptions {
            frames_per_second: 30,
            output_width: None,
            constant_rate_factor: 23,
        }
    }

    fn temporary_directory() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "gifmp4-converter-test-{}-{unique}-{}",
            std::process::id(),
            TEMPORARY_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        path
    }

    fn create_input(directory: &Path, filename: &str) -> PathBuf {
        let path = directory.join(filename);
        fs::write(&path, b"test media").unwrap();
        path
    }

    #[test]
    fn resolves_default_output_extension_for_both_directions() {
        assert_eq!(
            execute_output_path(Path::new("clip.gif"), None).unwrap(),
            PathBuf::from("clip.mp4")
        );
        assert_eq!(
            execute_output_path(Path::new("clip.mp4"), None).unwrap(),
            PathBuf::from("clip.gif")
        );
    }

    #[test]
    fn preserves_an_explicit_output_path() {
        let output = Path::new("somewhere/result.mp4");

        assert_eq!(
            execute_output_path(Path::new("clip.gif"), Some(output)).unwrap(),
            output
        );
    }

    #[test]
    fn rejects_a_missing_input_file() {
        let error =
            validate_conversion_request(Path::new("missing.gif"), None, &options()).unwrap_err();

        assert!(error.to_string().contains("入力ファイルが見つかりません"));
    }

    #[test]
    fn rejects_invalid_numeric_options() {
        let directory = temporary_directory();
        let input = create_input(&directory, "input.mp4");

        let invalid_options = [
            ConversionOptions {
                frames_per_second: 0,
                ..options()
            },
            ConversionOptions {
                output_width: Some(0),
                ..options()
            },
            ConversionOptions {
                constant_rate_factor: 52,
                ..options()
            },
        ];

        for invalid in invalid_options {
            assert!(validate_conversion_request(&input, None, &invalid).is_err());
        }

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn rejects_same_input_and_output_path() {
        let directory = temporary_directory();
        let input = create_input(&directory, "input.gif");

        let error = validate_conversion_request(&input, Some(&input), &options()).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("入力ファイルと出力ファイルに同じパス")
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn rejects_output_in_a_missing_directory() {
        let directory = temporary_directory();
        let input = create_input(&directory, "input.gif");
        let output = directory.join("missing").join("output.mp4");

        let error = validate_conversion_request(&input, Some(&output), &options()).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("出力先ディレクトリが存在しません")
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn accepts_a_valid_request() {
        let directory = temporary_directory();
        let input = create_input(&directory, "input.gif");
        let output = directory.join("output.mp4");

        validate_conversion_request(&input, Some(&output), &options()).unwrap();

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn ffmpeg_arguments_enable_overwrite_for_both_directions() {
        let conversion_options = options();

        let gif_to_mp4 = gif_to_mp4_arguments(
            Path::new("input.gif"),
            Path::new("output.mp4"),
            &conversion_options,
        );
        let mp4_to_gif = mp4_to_gif_arguments(
            Path::new("input.mp4"),
            Path::new("output.gif"),
            &conversion_options,
        );

        assert_eq!(gif_to_mp4.first(), Some(&OsString::from("-y")));
        assert_eq!(mp4_to_gif.first(), Some(&OsString::from("-y")));
    }

    #[test]
    fn determines_the_non_wildcard_prefix_as_the_glob_base() {
        assert_eq!(glob_base("*.gif"), PathBuf::from("."));
        assert_eq!(glob_base("media/**/*.gif"), PathBuf::from("media"));
        assert_eq!(
            glob_base("media/nested/clip.gif"),
            PathBuf::from("media/nested")
        );
    }

    #[test]
    fn preserves_subdirectories_in_batch_output_path() {
        let output = batch_output_path(
            Path::new("media/animals/cat.gif"),
            Path::new("media"),
            Path::new("converted"),
        )
        .unwrap();

        assert_eq!(output, PathBuf::from("converted/animals/cat.mp4"));
    }

    #[test]
    fn resolves_batch_output_for_a_pattern_in_the_current_directory() {
        let output = batch_output_path(
            Path::new("animation.gif"),
            Path::new("."),
            Path::new("converted"),
        )
        .unwrap();

        assert_eq!(output, PathBuf::from("converted/animation.mp4"));
    }

    #[test]
    fn batch_reports_when_no_files_match() {
        let directory = temporary_directory();
        let pattern = directory.join("**").join("*.gif");

        let error = convert_batch(pattern.to_str().unwrap(), None, &options()).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("パターンに一致するファイルがありません")
        );
        fs::remove_dir_all(directory).unwrap();
    }
}
