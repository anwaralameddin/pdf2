// A module to parse command line arguments

use ::clap::Parser;
use ::std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    name = "pdf2",
    version = "0.1.0",
    about = "A package for inspecting and comparing PDF files (UNDER DEVELOPMENT)"
)]

pub struct Args {
    #[clap(short, long, help = "Enable verbose output")]
    pub verbose: bool,
    #[clap(short, long, help = "A space-separated list of PDF files")]
    pub files: Vec<PathBuf>,
    #[clap(short, long, help = "The directory containing the PDF files")]
    pub directory: Option<PathBuf>,
}
