mod err;
mod opt;

use std::{borrow::Cow, io::Write};

use regex::bytes::Regex;
use regex_chunker::ByteChunker;

use err::FrErr;
use opt::{MatchMode, Opts, OutputMode};

fn find_subslice<T>(haystack: &[T], needle: &[T]) -> Option<usize>
where
    T: PartialEq,
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
Read input stream line-by-line, either replacing or extracting (and
possibly replacing) text according to the semantics of the
[`Regex::replace*`](https://docs.rs/regex/latest/regex/struct.Regex.html#method.replace)
family of functions.
*/
fn regex_match(mut opts: Opts) -> Result<(), FrErr> {
    let re = Regex::new(&opts.pattern)?;
    let chunker = ByteChunker::new(opts.input, &opts.delimiter)?;

    match opts.output_mode {
        OutputMode::Replace(repl) => {
            let repl = repl.as_bytes();
            for chunk in chunker {
                let chunk = chunk?;
                let altered = re.replacen(&chunk, opts.max, repl);

                match altered {
                    Cow::Owned(mut v) => {
                        if let Some(ref nl_bytes) = opts.newline {
                            v.extend_from_slice(nl_bytes);
                        }
                        opts.output.write_all(&v)?;
                    }
                    Cow::Borrowed(b) => {
                        opts.output.write_all(b)?;
                        if let Some(ref nl_bytes) = opts.newline {
                            opts.output.write_all(nl_bytes)?;
                        }
                    }
                }
            }
        }
        OutputMode::Extract(repl) => {
            let repl = repl.as_bytes();
            let mut buff: Vec<u8> = Vec::new();
            for chunk in chunker {
                let chunk = chunk?;

                for cap in re.captures_iter(&chunk) {
                    cap.expand(repl, &mut buff);
                }

                if !buff.is_empty() {
                    if let Some(ref nl_bytes) = opts.newline {
                        buff.extend_from_slice(nl_bytes);
                    }
                    opts.output.write_all(&buff)?;
                    buff.clear();
                }
            }
        }
    }

    opts.output.flush()?;
    Ok(())
}

/**
Read the input stream line-by line, either (as governed by
`opts.output_mode`) replacing or extracting (and maybe also
replacing) text.
*/
fn static_match(mut opts: Opts) -> Result<(), FrErr> {
    let patt = opts.pattern.as_bytes();
    let chunker = ByteChunker::new(opts.input, &opts.delimiter)?;
    let mut buff: Vec<u8> = Vec::new();

    match opts.output_mode {
        OutputMode::Replace(repl) => {
            let repl = repl.as_bytes();
            for chunk in chunker {
                let chunk = chunk?;
                let mut subslice = &chunk[..];
                let mut n_replaced: usize = 0;

                while n_replaced < opts.max {
                    if let Some(n) = find_subslice(subslice, patt) {
                        buff.extend_from_slice(&subslice[..n]);
                        buff.extend_from_slice(repl);
                        n_replaced += 1;
                        let offs = n + patt.len();
                        subslice = &subslice[offs..];
                    } else {
                        break;
                    }
                }

                if !subslice.is_empty() {
                    buff.extend_from_slice(subslice)
                }
                if let Some(ref nl_bytes) = opts.newline {
                    buff.extend_from_slice(nl_bytes);
                }
                opts.output.write_all(&buff)?;
                buff.clear();
            }
        }
        OutputMode::Extract(repl) => {
            let repl = repl.as_bytes();
            for chunk in chunker {
                let chunk = chunk?;
                let mut subslice = &chunk[..];
                let mut n_replaced: usize = 0;

                while n_replaced < opts.max {
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
                    if let Some(ref nl_bytes) = opts.newline {
                        buff.extend_from_slice(nl_bytes);
                    }
                    opts.output.write_all(&buff)?;
                    buff.clear();
                }
            }
        }
    }

    opts.output.flush()?;
    Ok(())
}

fn main() -> Result<(), FrErr> {
    let opts = Opts::new()?;

    let mode = opts.match_mode;

    match mode {
        MatchMode::Regex => regex_match(opts)?,
        MatchMode::Verbatim => static_match(opts)?,
    }

    Ok(())
}
