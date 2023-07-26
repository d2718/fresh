# fresh

```text
A friendlier sed replacement.

Usage: fresh [OPTIONS] <PATTERN> [REPLACE]

Arguments:
  <PATTERN>  Pattern to find
  [REPLACE]  Optional replacement

Options:
  -m, --max <N>           Maximum number of replacements per line (default is all)
  -x, --extract           Print only found pattern (default is print everything)
  -s, --simple            Do simple verbatim string matching (default is regex matching)
  -d, --delimiter <PATT>  Delimiter to separate "lines" [default: \r?\n]
  -n, --newline [<NL>]    Print something other than a newline between chunks
  -i, --input <INPUT>     Input file (default is stdin)
  -o, --output <OUTPUT>   Output file (default is stdout)
  -h, --help              Print help
  -V, --version           Print version
```

## Installation

It should just `cargo build --release`.

## Use

By default, `fresh` reads from stdin, writes to stdout, and replaces occurrences
of its first argument with its second argument.

```text
$ echo "lorem ipsum dolor sit amet..." | fresh 'o' '0'
l0rem ipsum d0l0r sit amet...
````

By default, the first argument is interpreted as a regex.

```text
$ echo "lorem ipsum dolor sit amet..." | fresh '[aeiou]' '*'
l*r*m *ps*m d*l*r s*t *m*t...
```
The `${N}` notation in the second argument will substitute in the Nth
capture group from the first argument.

```text
$ echo "lorem ipsum dolor sit amet..." | fresh '([aeiou])([mt])' '${1}.${2}'
lore.m ipsu.m dolor si.t a.me.t...
```

Force simple verbatim string matching with `-s`.

```test
echo "lorem ipsum dolor sit amet..." | fresh -s '.' '?'
lorem ipsum dolor sit amet???
```

Limit the number of replacements with `-m`.

```text
$ echo "lorem ipsum dolor sit amet..." | fresh -m 3 '([aeiou])([mt])' '$1.$2'
lore.m ipsu.m dolor si.t amet...
```

To print only the matched text (or its replacement), use `-x`.

```text
echo "lorem ipsum dolor sit amet..." | fresh -x '[aeiou]' '$0'
oeiuooiae
```

Omitting the replacement argument makes `fresh` behave like either
`fresh -x PATTERN '$0'` or `fresh -s -x PATTERN PATTERN`.

```text
$ echo "lorem ipsum dolor sit amet..." | fresh '[aeiou]'
oeiuooiae
$ echo "lorem ipsum dolor sit amet..." | fresh -s 'o'
ooo
```
  
## &c.

`fresh` is still a work in progress. The goal is to be a friendlier
approximation of the venerable
[`sed`](https://www.gnu.org/software/sed/manual/sed.html) utility.
It is not intended to be a feature-complete copy; it is intended to make
performing a regex find/replace on a stream of text simpler. Any other
features are essentially incidental.  Features in pursuit of that goal
will generally reflect the features of Rust's
[`regex` crate](https://docs.rs/regex/latest/regex/).

No real thought has been given to performance; this will probably
change incrementally.
