use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name="gifmp4", version, about="Convert GIF to MP4", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Convert GIF to MP4
    Convert {
        /// Input GIF file path
        input_path: PathBuf,
        output_path: Option<PathBuf>,

        #[arg(long = "fps", default_value_t = 30)]
        frame_per_second: u32,

        #[arg(long = "width")]
        output_width: Option<u32>,

        /// H.264 CRF used for GIF to MP4 conversion
        #[arg(long = "quality", default_value_t = 23)]
        constant_rate_factor: u8,

        /// Overwrite an existing output file
        #[arg(short = 'y', long)]
        overwrite: bool,
    },
    /// Check whether ffmpeg is installed and available in the system PATH
    Doctor,
}
