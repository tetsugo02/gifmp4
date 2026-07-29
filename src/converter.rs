use crate::ffmpeg;
use crate::media::{ConversionDirection, detect_conversion_direction};
use anyhow::{Result, bail};
use std::ffi::OsString;
use std::path::Path;

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

    let resolved_output_path = execute_output_path(input_path, output_path)?;

    if resolved_output_path == input_path {
        bail!(
            "入力ファイルと出力ファイルに同じパスは指定できません: {}",
            input_path.display()
        );
    }

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
    let scale_filter = "scale=trunc(iw/2)*2:trunc(ih/2)*2";

    let arguments = vec![
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
    ];

    ffmpeg::execute_ffmpeg(&arguments)
}

fn convert_mp4_to_gif(
    input_path: &Path,
    output_path: &Path,
    options: &ConversionOptions,
) -> Result<()> {
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

    let arguments = vec![
        OsString::from("-i"),
        input_path.as_os_str().to_os_string(),
        OsString::from("-filter_complex"),
        OsString::from(filter_graph),
        OsString::from("-an"),
        OsString::from("-loop"),
        OsString::from("0"),
        output_path.as_os_str().to_os_string(),
    ];

    ffmpeg::execute_ffmpeg(&arguments)
}
