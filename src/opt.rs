/*!
Parsing command-line options.
*/
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use clap::Parser;

use crate::FrErr;

#[derive(Clone, Copy, Debug)]
pub enum OutputMode {
    Replace,
    Extract,
}

#[derive(Clone, Copy, Debug)]
pub enum MatchMode {
    Regex,
    Verbatim,
}

#[derive(Parser)]
#[command(author, version, about)]
struct CliOpts {
    /// Pattern to find.
    pattern: String,

    /// Optional replacement.
    replace: Option<String>,

    /// Maximum number of replacements per line (default is all).
    #[arg(short, long, value_name = "N")]
    max: Option<usize>,

    /// Print only found pattern (default is print everything).
    #[arg(short = 'x', long = "extract")]
    extract: bool,

    /// Do simple verbatim string matching (default is regex matching).
    #[arg(short, long)]
    simple: bool,

    /// Delimiter to separate "lines".
    #[arg(short, long, value_name = "PATT",
        default_value_t = String::from(r#"\r?\n"#))]
    delimiter: String,

    /// Input file (default is stdin).
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output file (default is stdout).
    #[arg(short, long)]
    output: Option<PathBuf>,
}

pub struct Opts {
    pub pattern: String,
    pub replace: Option<String>,
    pub max: usize,
    pub output_mode: OutputMode,
    pub match_mode: MatchMode,
    pub delimiter: String,
    pub input: Box<dyn Read>,
    pub output: Box<dyn Write>,
}

impl Opts {
    pub fn new() -> Result<Self, FrErr> {
        let clio = CliOpts::parse();

        let max = clio.max.unwrap_or(usize::MAX);
        let output_mode = if clio.extract || clio.replace.is_none() {
            OutputMode::Extract
        } else {
            OutputMode::Replace
        };
        let match_mode = if clio.simple {
            MatchMode::Verbatim
        } else {
            MatchMode::Regex
        };

        let input: Box<dyn Read> = match clio.input {
            Some(pbuf) => Box::new(File::open(pbuf)?),
            None => Box::new(std::io::stdin().lock()),
        };
        let output: Box<dyn Write> = match clio.output {
            Some(pbuf) => Box::new(File::create(pbuf)?),
            None => Box::new(std::io::stdout().lock()),
        };

        Ok(Opts {
            pattern: clio.pattern,
            delimiter: clio.delimiter,
            replace: clio.replace,
            max, output_mode, match_mode,
            input, output,
        })
    }
}