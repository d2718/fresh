mod err;

use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

use clap::Parser;
use regex::Regex;

use err::FrErr;

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

/**
Read input stream line-by-line, replacing occurrences of `patt` with `repl`,
according to the semantics of the
[`Regex::replace*`](https://docs.rs/regex/latest/regex/struct.Regex.html#method.replace)
family of functions.
*/
fn regex_replace<B, W>(
    patt: &str,
    repl: &str,
    mut instream: B,
    mut outstream: W,
    n_rep: Option<usize>,
) -> Result<(), FrErr>
where
    B: BufRead,
    W: Write,
{
    let re = Regex::new(patt)?;

    let mut buff = String::new();
    loop {
        let n = instream.read_line(&mut buff)?;
        if n == 0 {
            return Ok(());
        }

        let altered = match n_rep {
            Some(n) => re.replacen(&buff, n, repl),
            None => re.replace_all(&buff, repl),
        };

        outstream.write(altered.as_bytes())?;
        buff.clear();
    }
}

/**
Read the input stream line-by-line, replacing all instances of `patt` with
`repl`. This is straight string matching, unlike `regex_replace()`.
*/
fn static_replace<B, W>(
    patt: &str,
    repl: &str,
    mut instream: B,
    mut outstream: W,
    n_rep: Option<usize>,
) -> Result<(), FrErr>
where
    B: BufRead,
    W: Write,
{
    let mut buff = String::new();
    loop {
        let n = instream.read_line(&mut buff)?;
        if n == 0 {
            return Ok(());
        }

        match n_rep {
            Some(n) => {
                let mut splitter = buff.splitn(n, patt);
                if let Some(chunk) = splitter.next() {
                    outstream.write(chunk.as_bytes())?;
                }
                for chunk in splitter {
                    outstream.write(repl.as_bytes())?;
                    outstream.write(chunk.as_bytes())?;
                }
            }
            None => {
                let mut splitter = buff.split(patt);
                if let Some(chunk) = splitter.next() {
                    outstream.write(chunk.as_bytes())?;
                }
                for chunk in splitter {
                    outstream.write(repl.as_bytes())?;
                    outstream.write(chunk.as_bytes())?;
                }
            }
        }
        buff.clear();
    }
}

/**
Searches through the input stream line-by-line, printing _only_ occurrences
of the matcing pattern (possibly modified by the `repl`) argument, if not
`None`. Like `regex_replace()`, this modification is per the function of
`Regex::replace`.
*/
fn regex_extract<B, W>(
    patt: &str,
    repl: Option<&str>,
    mut instream: B,
    mut outstream: W,
    n_rep: Option<usize>,
) -> Result<(), FrErr>
where
    B: BufRead,
    W: Write,
{
    let re = Regex::new(patt)?;

    let mut buff = String::new();
    loop {
        let n = instream.read_line(&mut buff)?;
        if n == 0 {
            return Ok(());
        }

        let mut n = 0;
        let mut cap_idx = 0;
        let mut matched = false;
        while let Some(m) = re.find_at(&buff, cap_idx) {
            if let Some(n_rep) = n_rep {
                if n >= n_rep {
                    break;
                }
            }
            if let Some(repl) = repl {
                let altered = re.replace(&buff[m.start()..m.end()], repl);
                outstream.write(altered.as_bytes())?;
            } else {
                outstream.write(m.as_str().as_bytes())?;
            }
            matched = true;
            cap_idx = m.end();
            n += 1;
        }
        if matched {
            outstream.write("\n".as_bytes())?;
        }

        buff.clear();
    }
}

/**
Search through the input line-by-line, printing _only_ the occurrences of
`patt` (or, if `repl` is not `None`, prints `repl` for every occurrence
of `patt`). This is static string matching, not regex matching.
*/
fn static_extract<B, W>(
    patt: &str,
    repl: Option<&str>,
    mut instream: B,
    mut outstream: W,
    n_rep: Option<usize>,
) -> Result<(), FrErr>
where
    B: BufRead,
    W: Write,
{
    let mut buff = String::new();
    loop {
        let n = instream.read_line(&mut buff)?;
        if n == 0 {
            return Ok(());
        }

        let mut matched = false;
        for (n, _) in buff.matches(patt).enumerate() {
            if let Some(n_rep) = n_rep {
                if n >= n_rep {
                    break;
                }
            }
            let chunk = match repl {
                Some(repl) => repl,
                None => patt,
            };
            outstream.write(chunk.as_bytes())?;
            matched = true;
        }
        if matched {
            outstream.write("\n".as_bytes())?;
        }

        buff.clear();
    }
}

fn main() -> Result<(), FrErr> {
    let opts = Opts::parse();

    let mut input_stream: Box<dyn BufRead> = match &opts.input {
        Some(pbuf) => {
            let f = File::open(pbuf)?;
            Box::new(BufReader::new(f))
        }
        None => Box::new(std::io::stdin().lock()),
    };

    let mut output_stream: Box<dyn Write> = match &opts.output {
        Some(pbuf) => {
            let f = File::create(pbuf)?;
            Box::new(BufWriter::new(f))
        }
        None => Box::new(BufWriter::new(std::io::stdout().lock())),
    };

    if opts.replace.is_none() || opts.extract {
        if opts.simple {
            static_extract(
                &opts.pattern,
                opts.replace.as_deref(),
                &mut input_stream,
                &mut output_stream,
                opts.number,
            )?;
        } else {
            regex_extract(
                &opts.pattern,
                opts.replace.as_deref(),
                &mut input_stream,
                &mut output_stream,
                opts.number,
            )?;
        }
    } else {
        // Guaranteed by if clause to not be None.
        let repl = opts.replace.unwrap();
        if opts.simple {
            static_replace(
                &opts.pattern,
                &repl,
                &mut input_stream,
                &mut output_stream,
                opts.number,
            )?;
        } else {
            regex_replace(
                &opts.pattern,
                &repl,
                &mut input_stream,
                &mut output_stream,
                opts.number,
            )?;
        }
    }

    output_stream.flush()?;

    Ok(())
}
