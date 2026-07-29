use clap::Parser;
use gifmp4::cli::{Cli, CliCommand};
use std::path::PathBuf;

#[test]
fn parses_convert_with_default_options() {
    let cli = Cli::try_parse_from(["gifmp4", "convert", "input.gif"]).unwrap();

    match cli.command {
        CliCommand::Convert {
            input_path,
            output_path,
            frames_per_second,
            output_width,
            constant_rate_factor,
        } => {
            assert_eq!(input_path, PathBuf::from("input.gif"));
            assert_eq!(output_path, None);
            assert_eq!(frames_per_second, 30);
            assert_eq!(output_width, None);
            assert_eq!(constant_rate_factor, 23);
        }
        CliCommand::Batch { .. } | CliCommand::Doctor => panic!("expected convert command"),
    }
}

#[test]
fn parses_convert_with_all_options() {
    let cli = Cli::try_parse_from([
        "gifmp4",
        "convert",
        "input.mp4",
        "output.gif",
        "--fps",
        "12",
        "--width",
        "640",
        "--quality",
        "18",
    ])
    .unwrap();

    match cli.command {
        CliCommand::Convert {
            input_path,
            output_path,
            frames_per_second,
            output_width,
            constant_rate_factor,
        } => {
            assert_eq!(input_path, PathBuf::from("input.mp4"));
            assert_eq!(output_path, Some(PathBuf::from("output.gif")));
            assert_eq!(frames_per_second, 12);
            assert_eq!(output_width, Some(640));
            assert_eq!(constant_rate_factor, 18);
        }
        CliCommand::Batch { .. } | CliCommand::Doctor => panic!("expected convert command"),
    }
}

#[test]
fn parses_doctor_command() {
    let cli = Cli::try_parse_from(["gifmp4", "doctor"]).unwrap();

    assert!(matches!(cli.command, CliCommand::Doctor));
}

#[test]
fn rejects_non_numeric_option_values() {
    let error =
        Cli::try_parse_from(["gifmp4", "convert", "input.gif", "--fps", "fast"]).unwrap_err();

    assert!(error.to_string().contains("invalid value"));
}

#[test]
fn parses_batch_with_output_directory_and_options() {
    let cli = Cli::try_parse_from([
        "gifmp4",
        "batch",
        "media/**/*.gif",
        "--output-dir",
        "converted",
        "--fps",
        "15",
        "--width",
        "480",
        "--quality",
        "20",
    ])
    .unwrap();

    match cli.command {
        CliCommand::Batch {
            pattern,
            output_dir,
            frames_per_second,
            output_width,
            constant_rate_factor,
        } => {
            assert_eq!(pattern, "media/**/*.gif");
            assert_eq!(output_dir, Some(PathBuf::from("converted")));
            assert_eq!(frames_per_second, 15);
            assert_eq!(output_width, Some(480));
            assert_eq!(constant_rate_factor, 20);
        }
        CliCommand::Convert { .. } | CliCommand::Doctor => panic!("expected batch command"),
    }
}
