use gifmp4::media::{ConversionDirection, detect_conversion_direction};
use std::path::Path;

#[test]
fn detects_gif_to_mp4_from_explicit_extensions() {
    let direction =
        detect_conversion_direction(Path::new("animation.gif"), Some(Path::new("video.mp4")))
            .unwrap();

    assert_eq!(direction, ConversionDirection::GifToMp4);
}

#[test]
fn detects_mp4_to_gif_from_explicit_extensions() {
    let direction =
        detect_conversion_direction(Path::new("video.mp4"), Some(Path::new("animation.gif")))
            .unwrap();

    assert_eq!(direction, ConversionDirection::Mp4ToGif);
}

#[test]
fn infers_output_format_when_output_path_is_omitted() {
    assert_eq!(
        detect_conversion_direction(Path::new("animation.gif"), None).unwrap(),
        ConversionDirection::GifToMp4
    );
    assert_eq!(
        detect_conversion_direction(Path::new("video.mp4"), None).unwrap(),
        ConversionDirection::Mp4ToGif
    );
}

#[test]
fn extensions_are_case_insensitive() {
    assert_eq!(
        detect_conversion_direction(Path::new("animation.GIF"), Some(Path::new("video.MP4")))
            .unwrap(),
        ConversionDirection::GifToMp4
    );
}

#[test]
fn rejects_unsupported_or_matching_formats() {
    for (input, output) in [
        ("image.png", None),
        ("animation.gif", Some("copy.gif")),
        ("video.mp4", Some("copy.mp4")),
    ] {
        let error =
            detect_conversion_direction(Path::new(input), output.map(Path::new)).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Unsupported conversion direction"),
            "unexpected error for {input}: {error:#}"
        );
    }
}

#[test]
fn reports_paths_without_an_extension() {
    let error = detect_conversion_direction(Path::new("animation"), None).unwrap_err();

    assert!(
        error.to_string().contains("Failed to normalize input path"),
        "{error:#}"
    );
}
