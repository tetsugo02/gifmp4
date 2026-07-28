use crate::media::{ConversionDirection, detect_conversion_direction};
use anyhow::{Result, bail};
use std::path::Path;

#[derive(Debug)]
pub struct ConversionOptions {
    pub frames_per_second: u32,
    pub output_width: Option<u32>,
    pub constant_rate_factor: u8,
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

    let conversion_direction = detect_conversion_direction(input_path, output_path)?;

    let resolved_output_path = match output_path {
        Some(specified_output_path) => specified_output_path.to_path_buf(),
        None => match conversion_direction {
            ConversionDirection::GifToMp4 => input_path.with_extension("mp4"),
            ConversionDirection::Mp4ToGif => input_path.with_extension("gif"),
        },
    };

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
