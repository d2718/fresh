mod err;

use std::{
    borrow::Cow,
    fs::File,
    io::{BufWriter, Read, Write},
    path::PathBuf,
};

use clap::Parser;
use regex::bytes::Regex;
use regex_chunker::ByteChunker;

use err::FrErr;

#[cfg(not(windows))]
static NEWLINE: &[u8] = b"\n";
#[cfg(windows)]
static NEWLINE: &[u8] = b"\r\n";

#[derive(Parser)]
#[command(author, version, about)]
struct Opts {
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

    /// Do simple verbating string matching (default is regex matching).
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

fn find_subslice<T>(haystack: &[T], needle: &[T]) -> Option<usize>
where
    T: PartialEq
{
    if needle.len() > haystack.len() {
        return None;
    }

    for (n, w) in haystack.windows(needle.len()).enumerate() {
        if w == needle {
            return Some(n);
        }
    }

    None
}

/**
Read input stream line-by-line, replacing occurrences of `patt` with `repl`,
according to the semantics of the
[`Regex::replace*`](https://docs.rs/regex/latest/regex/struct.Regex.html#method.replace)
family of functions.
*/
fn regex_replace<R, W>(
    patt: &str,
    repl: &str,
    delim: &str,
    instream: R,
    mut outstream: W,
    n_rep: Option<usize>,
) -> Result<(), FrErr>
where
    R: Read,
    W: Write,
{
    let re = Regex::new(patt)?;
    let chunker = ByteChunker::new(instream, delim)?;
    let repl = repl.as_bytes();

    for chunk in chunker {
        let chunk = chunk?;
        let altered = match n_rep {
            Some(n) => re.replacen(&chunk, n, repl),
            None => re.replace_all(&chunk, repl),
        };

        match altered {
            Cow::Owned(mut v) => {
                v.extend_from_slice(NEWLINE);
                outstream.write(&v)?;
            },
            Cow::Borrowed(b) => {
                outstream.write(b)?;
                outstream.write(NEWLINE)?;
            }
        }
    }

    Ok(())
}

/**
Read the input stream line-by-line, replacing all instances of `patt` with
`repl`. This is straight string matching, unlike `regex_replace()`.
*/
fn static_replace<R, W>(
    patt: &str,
    repl: &str,
    delim: &str,
    instream: R,
    mut outstream: W,
    n_rep: Option<usize>,
) -> Result<(), FrErr>
where
    R: Read,
    W: Write,
{
    let patt = patt.as_bytes();
    let repl = repl.as_bytes();
    let chunker = ByteChunker::new(instream, delim)?;
    let n_rep = n_rep.unwrap_or(usize::MAX);

    for chunk in chunker {
        let chunk = chunk?;
        let mut subslice = &chunk[..];
        let mut n_replaced: usize = 0;

        while n_replaced < n_rep {
            if let Some(n) = find_subslice(subslice, patt) {
                outstream.write_all(&subslice[..n])?;
                outstream.write_all(repl)?;
                n_replaced += 1;
                let offs = n + patt.len();
                subslice = &subslice[offs..];
            } else {
                break;
            }
        }

        if !subslice.is_empty() {
            outstream.write_all(subslice)?;
            outstream.write_all(NEWLINE)?;
        }
    }

    Ok(())
}

/**
Searches through the input stream line-by-line, printing _only_ occurrences
of the matcing pattern (possibly modified by the `repl`) argument, if not
`None`. Like `regex_replace()`, this modification is per the function of
`Regex::replace`.
*/
fn regex_extract<R, W>(
    patt: &str,
    repl: Option<&str>,
    delim: &str,
    instream: R,
    mut outstream: W,
    n_rep: Option<usize>,
) -> Result<(), FrErr>
where
    R: Read,
    W: Write,
{
    let re = Regex::new(patt)?;
    let chunker = ByteChunker::new(instream, delim)?;
    let n_rep = n_rep.unwrap_or(usize::MAX);

    let mut buff = Vec::new();
    for chunk in chunker {
        let chunk = chunk?;

        if let Some(repl) = repl {
            for cap in re.captures_iter(&chunk).take(n_rep) {
                cap.expand(repl.as_bytes(), &mut buff);
            }
        } else {
            for m in re.find_iter(&chunk).take(n_rep) {
                buff.extend_from_slice(&chunk[m.range()]);
            }
        }

        if !buff.is_empty() {
            buff.extend_from_slice(NEWLINE);
            outstream.write_all(&buff)?;
            buff.clear();
        }
    }

    Ok(())
}

/**
Search through the input line-by-line, printing _only_ the occurrences of
`patt` (or, if `repl` is not `None`, prints `repl` for every occurrence
of `patt`). This is static string matching, not regex matching.
*/
fn static_extract<R, W>(
    patt: &str,
    repl: Option<&str>,
    delim: &str,
    instream: R,
    mut outstream: W,
    n_rep: Option<usize>,
) -> Result<(), FrErr>
where
    R: Read,
    W: Write,
{
    let patt = patt.as_bytes();
    let repl = repl.map(|x| x.as_bytes()).unwrap_or(patt);
    let chunker = ByteChunker::new(instream, delim)?;
    let n_rep = n_rep.unwrap_or(usize::MAX);
    let mut buff: Vec<u8> = Vec::new();

    for chunk in chunker {
        let chunk = chunk?;
        let mut subslice = &chunk[..];
        let mut n_replaced: usize = 0;

        while n_replaced < n_rep {
            if let Some(n) = find_subslice(subslice, patt) {
                buff.extend_from_slice(repl);
                n_replaced += 1;
                let offs = n + repl.len();
                subslice = &subslice[offs..];
            } else {
                break;
            }
        }

        if !buff.is_empty() {
            buff.extend_from_slice(NEWLINE);
            outstream.write_all(&buff)?;
            buff.clear();
        }
    }

    Ok(())
}

fn main() -> Result<(), FrErr> {
    let opts = Opts::parse();

    let mut input_stream: Box<dyn Read> = match &opts.input {
        Some(pbuf) => Box::new(File::open(pbuf)?),
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
                &opts.delimiter,
                &mut input_stream,
                &mut output_stream,
                opts.max,
            )?;
        } else {
            regex_extract(
                &opts.pattern,
                opts.replace.as_deref(),
                &opts.delimiter,
                &mut input_stream,
                &mut output_stream,
                opts.max,
            )?;
        }
    } else {
        // Guaranteed by if clause to not be None.
        let repl = opts.replace.unwrap();
        if opts.simple {
            static_replace(
                &opts.pattern,
                &repl,
                &opts.delimiter,
                &mut input_stream,
                &mut output_stream,
                opts.max,
            )?;
        } else {
            regex_replace(
                &opts.pattern,
                &repl,
                &opts.delimiter,
                &mut input_stream,
                &mut output_stream,
                opts.max,
            )?;
        }
    }

    output_stream.flush()?;

    Ok(())
}
