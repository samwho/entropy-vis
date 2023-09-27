use anyhow::Error;
use log::debug;
use std::cmp::max;
use std::fs::File;
use std::io::Read;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::result::Result;
use structopt::StructOpt;
use termion::color;
use termion::terminal_size;

#[derive(StructOpt)]
struct Cli {
    /// File to display as a grid
    file: PathBuf,

    #[structopt(short = "w", long = "width")]
    width: Option<u64>,

    #[structopt(short = "h", long = "height")]
    height: Option<u64>,
}

fn main() -> Result<(), Error> {
    human_panic::setup_panic!();
    env_logger::init();

    let args = Cli::from_args();
    let mut file = File::open(&args.file)?;
    let file_size = file.metadata()?.len();

    debug!("file: {}, size: {}", args.file.display(), file_size);

    let mut grid_width;
    let mut grid_height;

    match (args.width, args.height) {
        (Some(width), Some(height)) => {
            if width * height > file_size {
                return Err(Error::msg(format!(
                    "grid size ({}) is larger than file size ({})",
                    width * height,
                    file_size
                )));
            }

            grid_width = width;
            grid_height = height;
        }
        _ => {
            let (term_width, term_height) = terminal_size()?;
            grid_width = max(1, term_width as u64 / 2);
            grid_height = max(1, (term_height as f64 / 1.1).ceil() as u64);

            if let Some(width) = args.width {
                grid_width = width;
            }

            if let Some(height) = args.height {
                grid_height = height;
            }
        }
    }

    debug!("grid width: {}, height: {}", grid_width, grid_height);

    let chunk_size = max(1, file_size / (grid_width * grid_height));

    debug!(
        "chunk size: {} ({} / {})",
        chunk_size,
        file_size,
        grid_width * grid_height
    );

    let mut stdout = stdout().lock();

    let mut sum: u64 = 0;
    let mut count = 0;
    let mut total_chunks = 0;
    let mut buf = [0; 8096];

    loop {
        let bytes_read = file.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }

        for byte in buf.iter().take(bytes_read) {
            sum += *byte as u64;
            count += 1;

            if count == chunk_size {
                let avg = sum / count;
                let intensity = 255 - avg as u8;
                let c = color::Rgb(intensity, intensity, intensity);
                write!(stdout, "{}{}  ", color::Fg(c), color::Bg(c))?;
                write!(
                    stdout,
                    "{}{}",
                    color::Fg(color::Reset),
                    color::Bg(color::Reset)
                )?;
                sum = 0;
                count = 0;
                total_chunks += 1;

                if total_chunks % grid_width == 0 {
                    writeln!(stdout)?;
                }

                stdout.flush()?;
            }
        }
    }

    writeln!(stdout)?;
    Ok(())
}
