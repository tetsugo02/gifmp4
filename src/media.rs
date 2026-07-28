use anyhow::{Context, Result, bail};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionDirection {
    GifToMp4,
    Mp4ToGif,
}

pub fn detect_conversion_direction(
    input_path: &Path,
    output_path: Option<&Path>,
) -> Result<ConversionDirection> {
    let input_extension = normalize_path(input_path)
        .with_context(|| format!("Failed to normalize input path: {:?}", input_path))?;
    let output_extension = match output_path {
        Some(path) => normalize_path(path)
            .with_context(|| format!("Failed to normalize output path: {:?}", path))?,
        None => String::new(),
    };

    match (input_extension.as_str(), output_extension.as_str()) {
        ("gif", "mp4") => Ok(ConversionDirection::GifToMp4),
        ("mp4", "gif") => Ok(ConversionDirection::Mp4ToGif),
        ("gif", "") => Ok(ConversionDirection::GifToMp4),
        ("mp4", "") => Ok(ConversionDirection::Mp4ToGif),
        _ => bail!(
            "Unsupported conversion direction: input extension: {}, output extension: {}",
            input_extension,
            output_extension
        ),
    }
}

fn normalize_path(path: &Path) -> Result<String> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .context("There is no extension in the input file path")?;

    return Ok(extension.to_ascii_lowercase());
}
