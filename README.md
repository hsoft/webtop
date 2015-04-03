# webtop: apachetop in Rust

Move along, this is just me playing around with Rust. I was really *aching* to try it and I looked
for something, anything. I though "well, why not yet another apachetop?".

It's built on curses using [ncurses-rs][ncurses-rs]

## Compiling

To compile this, you need to be on Rust 0.13 nightly. You calso need Cargo.

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

**Note:** Due to a recent refactoring, Path and Referer modes don't do anything (it didn't even
work properly anyway). It will come back later.

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

## Unsafe code

There's some usage of unsafe code in the program:

* TTY fiddling with `libc::isatty()`, `libc::fdopen()` and `libc::fopen()`.

[ncurses-rs]: https://github.com/jeaye/ncurses-rs

