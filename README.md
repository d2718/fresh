# fresh
A friendlier sed-like utility.

```text
Usage: fresh [OPTIONS] <PATTERN> [REPLACE]

Arguments:
  <PATTERN>  Pattern to find
  [REPLACE]  Optional replacement

Options:
  -n, --number <N>       Maximum number of replacements per line (default is all)
  -x, --extract          Print only found pattern (default is print everything)
  -s, --simple           Do simple verbating string matching (default is regex matching)
  -i, --input <INPUT>    Input file (default is stdin)
  -o, --output <OUTPUT>  Output file (default is stdout)
  -h, --help             Print help
  -V, --version          Print version
```

`fresh` is still a work in progress. No thoughts have yet been given to
performance.

By default, `fresh` reads from stdin, writes to stdout, and replaces occurrences
of its first argument with its second argument.

```text
$ echo "lorem ipsum dolor sit amet..." | fresh 'o' '0'
l0rem ipsum d0l0r sit amet...
````

Again, by default, the first argument is interpreted as a regex.

```text
$ echo "lorem ipsum dolor sit amet..." | fresh '[aeiou]' '*'
l*r*m *ps*m d*l*r s*t *m*t...
```
The `${N}` notation in the second argument will substitute in the Nth
capture group from the first argument.

```text
$ echo "lorem ipsum dolor sit amet..." | fresh '([aeiou])([mt])' '$1.$2'
lore.m ipsu.m dolor si.t a.me.t...
```

Force simple verbatim string matching with `-s`.

```test
echo "lorem ipsum dolor sit amet..." | fresh -s '.' '?'
lorem ipsum dolor sit amet???
```

Limit the number of replacements with `-n`.

```text
$ echo "lorem ipsum dolor sit amet..." | fresh -n 3 '([aeiou])([mt])' '$1.$2'
lore.m ipsu.m dolor si.t amet...
```