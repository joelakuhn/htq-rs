# HTQ

HTQ is a grep-like tool for extracting elements from html streams using CSS3 selectors. It uses the incomparable [libmodest](https://github.com/lexborisov/Modest) for html and css parsing and css selector filtering.

## Building

```shell
cargo build --release
```

## Usage

```
Usage: htq [OPTIONS] [files]...

Arguments:
  [files]...

Options:
  -c, --css <selector>    CSS selector
  -a, --attr <attribute>  Extract an attribute value
  -t, --text              Print text content only
  -h, --prefix            Print file name prefix
  -l, --list              Only print matching file names
  -C, --count             Print the number of matches
  -q, --quiet             Suppress output
  -0, --print0            Null-terminate lines
  -o, --output <output>   Output file
  -h, --help              Print help
  -V, --version           Print version
```

## Examples

```html
<!-- test.html -->
<!doctype html>
<html>
  <body>
    <h1>Test Page</h1>
    <blockquote>
      <p>Oh how I wish I could shut up like a telescope!</p>
      <p>I think I could, if I only knew how to begin.</p>
      <footer> — Lewis Carroll, Alice's Adventures in Wonderland</footer>
    </blockquote>
    <ul>
      <li><a href="#link-1">Link 1</a></li>
      <li><a href="#link-2">Link 2</a></li>
    </ul>
  </body>
</html>
```

```shell
$ htq p test.html
<p>Oh how I wish I could shut up like a telescope!</p>
<p>I think I could, if I only knew how to begin.</p>
$ htq p --text test.html
Oh how I wish I could shut up like a telescope!
I think I could, if I only knew how to begin.
$ htq a --attr href test.html
#link-1
#link-2
$ curl -s https://github.com/joelakuhn/htq | htq -c 'article h1' -c 'article h2' -t
HTQ
Building
Usage
Examples
```
