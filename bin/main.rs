mod arg;
mod fs;

use ::clap::Parser;
use ::pdf2::PdfBuilder;

use self::arg::Args;
use self::fs::append_pdf_files;
use self::fs::filter_pdf_files;

fn main() {
    let args = Args::parse();
    let mut files = filter_pdf_files(args.files);
    if let Some(dir) = args.directory {
        append_pdf_files(&mut files, &dir);
    }

    for file in files {
        // TODO Replace with log::info!.
        println!("INFO: Processing file: {:?}", file);
        let pdf_builder = match PdfBuilder::new(&file) {
            Ok(pdf_builder) => pdf_builder,
            Err(err) => {
                // TODO Replace with log::error!.
                eprintln!("ERROR: Failed to create PDF builder: {:?}", err);
                continue;
            }
        };
        let pdf = match pdf_builder.build() {
            Ok(pdf) => pdf,
            Err(err) => {
                // TODO Replace with log::error!.
                eprintln!("ERROR: Failed to build PDF: {}", err);
                continue;
            }
        };
        if let Err(err) = pdf.status() {
            // TODO Replace with log::error!.
            eprintln!("ERROR: PDF status: {}", err);
            continue;
        }
        // TODO Replace with log::info!.
        println!("INFO: PDF summary: {}", pdf.summary());
    }
}
