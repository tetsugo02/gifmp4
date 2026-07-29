use anyhow::Result;
use clap::Parser;
use gifmp4::cli::{Cli, CliCommand};
use gifmp4::converter::{self, ConversionOptions};
use gifmp4::ffmpeg;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        CliCommand::Convert {
            input_path,
            output_path,
            frames_per_second,
            output_width,
            constant_rate_factor,
        } => {
            let conversion_options = ConversionOptions {
                frames_per_second,
                output_width,
                constant_rate_factor,
            };
            converter::convert(&input_path, output_path.as_deref(), &conversion_options)
        }
        CliCommand::Doctor => ffmpeg::doctor(),
    }
}
