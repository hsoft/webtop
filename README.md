# webtop: apachetop in Rust

Webtop is a [Rust][rust] console program that reads a log file from a HTTP server and output live
visit statistics (that is, visits having had at least a hit in the last 5 minutes). It repeatedly
reads the log file to keep stats fresh.

It's built on curses using [ncurses-rs][ncurses-rs]

## Status

Very early development. This project is mainly a sandbox for me to learn Rust in, but it's shaping
along well. Want to learn Rust with me? Contributions welcome.

## Features

* Live stats: repeadly polls the target log file
* Stats by Host, Path and Referer
* Can read from STDIN *continuously* ([goaccess][goaccess] doesn't do that)
* ncurses (console) interface

## Compiling

To compile this, you need Rust 1.0.0 beta with Cargo.

Then, it's only a matter of:

    $ cargo build

The resulting `webtop` binary will end up in the `target/` subfolder.

## Usage

It has very limited functionality, but the basics are that you call `webtop` with the target
log file you want to watch. Example: `webtop www.access.log`.

The program only reads the end of the target file. It works by repeatedly `stat`-ing the target
file and read the size difference from the last stat.

The program will then present you with a curses based interface showing you HTTP hits, grouped
by Host, ordered by hit count. There's also the Path mode and the Referer mode which group hits
differently.

### Piping STDIN

You can read `STDIN` by passing `-` as an argument to `webtop`. For example, if you are watching
a remote file, you could use `tail -f www.access.log | webtop -`.

Note that when you quit, because `STDIN` is still open, the process will not quit until you press
`CTRL-C`. I haven't managed to work around that limitation yet.

### Keybindings

* `q` - quit
* `h` - Host mode
* `p` - Path mode
* `r` - Referer mode
* `up/down` - Move selection up and down (selection doesn't do anything for now)

## Unsafe code

There's some usage of unsafe code in the program:

* TTY fiddling with `libc::isatty()`, `libc::fdopen()` and `libc::fopen()`.

## Alternatives

On the top of my head:

* apachetop ([many projects competing for the name](https://duckduckgo.com/?q=apachetop))
* [goaccess][goaccess]
* [wtop][wtop]

[rust]: http://rust-lang.org/
[ncurses-rs]: https://github.com/jeaye/ncurses-rs
[goaccess]: http://goaccess.io/
[wtop]: https://github.com/ClockworkNet/wtop

