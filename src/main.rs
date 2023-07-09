use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use clap::Parser;
use regex::Regex;

#[derive(Parser)]
#[command(author, version, about)]
struct Opts {
    /// Pattern to find.
    pattern: String,

    /// Optional replacement.
    replace: Option<String>,

    /// Maximum number of replacements per line (default is all).
    #[arg(short, long, value_name = "N")]
    number: Option<usize>,

    /// Print only found pattern (default is print everything).
    #[arg(short = 'x', long = "extract")]
    extract: bool,

    /// Do simple verbating string matching (default is regex matching).
    #[arg(short, long)]
    simple: bool,

    /// Input file (default is stdin).
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output file (default is stdout).
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn regex_replace<B, W>(
    patt: &str,
    repl: &str,
    mut instream: B,
    mut outstream: W,
    n_rep: Option<usize>,
) -> Result<(), String>
where
    B: BufRead,
    W: Write,
{
    let re = Regex::new(patt).map_err(|e| format!("invalid regex pattern \"{}\": {}", patt, &e))?;

    let mut buff = String::new();
    loop {
        let n = instream
            .read_line(&mut buff)
            .map_err(|e| format!("error reading from input stream: {}", &e))?;
        if n == 0 {
            return Ok(());
        }

        let altered = match n_rep {
            Some(n) => re.replacen(&buff, n, repl),
            None => re.replace_all(&buff, repl),
        };

        outstream
            .write(&altered.as_bytes())
            .map_err(|e| format!("error writing to output stream: {}", &e))?;
        buff.clear();
    }
}

fn static_replace<B, W>(
    patt: &str,
    repl: &str,
    mut instream: B,
    mut outstream: W,
    n_rep: Option<usize>,
) -> Result<(), String>
where
    B: BufRead,
    W: Write,
{
    let mut buff = String::new();
    loop {
        let n = instream
            .read_line(&mut buff)
            .map_err(|e| format!("error reading from input stream: {}", &e))?;
        if n == 0 {
            return Ok(());
        }

        match n_rep {
            Some(n) => {
                let mut splitter = buff.splitn(n, patt);
                if let Some(chunk) = splitter.next() {
                    outstream
                        .write(chunk.as_bytes())
                        .map_err(|e| format!("error writing to output stream: {}", &e))?;
                }
                while let Some(chunk) = splitter.next() {
                    outstream
                        .write(repl.as_bytes())
                        .map_err(|e| format!("error writing to output stream: {}", &e))?;
                    outstream
                        .write(chunk.as_bytes())
                        .map_err(|e| format!("error writing to output stream: {}", &e))?;
                }
            }
            None => {
                let mut splitter = buff.split(patt);
                if let Some(chunk) = splitter.next() {
                    outstream
                        .write(chunk.as_bytes())
                        .map_err(|e| format!("error writing to output stream: {}", &e))?;
                }
                while let Some(chunk) = splitter.next() {
                    outstream
                        .write(repl.as_bytes())
                        .map_err(|e| format!("error writing to output stream: {}", &e))?;
                    outstream
                        .write(chunk.as_bytes())
                        .map_err(|e| format!("error writing to output stream: {}", &e))?;
                }
            }
        }
    }
}

fn main() -> Result<(), String> {
    let opts = Opts::parse();

    if opts.extract {
        return Err("non-replacement not yet supported".into());
    }

    let mut input_stream: Box<dyn BufRead> = match &opts.input {
        Some(pbuf) => {
            let f = File::open(pbuf)
                .map_err(|e| format!("unable to open input file {}: {}", pbuf.display(), &e))?;
            Box::new(BufReader::new(f))
        }
        None => Box::new(std::io::stdin().lock()),
    };

    let mut output_stream: Box<dyn Write> = match &opts.output {
        Some(pbuf) => {
            let f = File::create(pbuf)
                .map_err(|e| format!("unable to open output file {}: {}", pbuf.display(), &e))?;
            Box::new(f)
        }
        None => Box::new(std::io::stdout().lock()),
    };

    let repl: &str = &opts
        .replace
        .ok_or_else(|| format!("non-replacement not yet supported"))?;

    if opts.simple {
        static_replace(
            &opts.pattern,
            repl,
            &mut input_stream,
            &mut output_stream,
            opts.number,
        )
    } else {
        regex_replace(
            &opts.pattern,
            repl,
            &mut input_stream,
            &mut output_stream,
            opts.number,
        )
    }
}
