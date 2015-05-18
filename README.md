# webtop: apachetop in Rust

Webtop is a [Rust][rust] console program that reads a log file from a HTTP server and output live
visit statistics (that is, visits having had at least a hit in the last 5 minutes). It repeatedly
reads the log file to keep stats fresh.

It's built on curses using [ncurses-rs][ncurses-rs]

## Features

* Live stats: repeadly polls the target log file
* Stats by Host, Path and Referer
* Drill down single visit stats
* Can read from STDIN *continuously* ([goaccess][goaccess] doesn't do that) so you can `tail -f`
  from a Docker container and pipe this in `webtop`.
* ncurses (console) interface

## Status & Roadmap

Early development. This project is mainly a sandbox for me to learn Rust in, but it's shaping
along well. Because of this, I think I'll aim for shaping this into an actual program.

Want to learn Rust with me? Contributions welcome.

Things I want to do before 1.0 are:

### Multiple log files support

When using multiple log files, we would get merged visits stats, so a host visiting `site1` would
be the same as the host visiting `site2`, but we'd also have specific stats subsites. For example,
`/some/path` in `site1z wouldn't be considered as the same as `/some/path` in `site2`.

This is something that other similar apps like goaccess lack but that I'd find useful.

### Full drill-down

Make each base table "drill-down"-able, with navigable subtables which can also be drilled down,
up to our atomic structure, the visit.

### More log formats

For now, there's support for only one log format. We need more.

### Better stats

* Bandwidth stats
* Resource types filtering (ignore image, CSS and JS hits)
* Better usage of screen estate for tables
* Alternative sort criteria

### Documentation

Developer and user documentation.

## Compiling

To compile this, you need Rust 1.0 with Cargo.

Then, it's only a matter of:

    $ cargo build

The resulting `webtop` binary will end up in the `target/` subfolder.

## Usage

It has very limited functionality, but the basics are that you call `webtop` with the target
log file you want to watch. Example: `webtop www.access.log`.

The program only reads the end of the target file. It works by repeatedly `stat`-ing the target
file and read the size difference from the last stat.

### Display

The program will present you with a curses based interface showing you HTTP hits, grouped
by Host, ordered by hit count. 

A line will start with a `!` if the visit has something "special". For now, "special" means at
least one 4xx or 5xx hit.

There's also the Path mode and the Referer mode which group hits differently.

### Details

When you press `d`, it summons the Details panel, which shows more details about the currently
selected item. For now, this only works in Host mode

Because of the moving nature of the display, the details panel doesn't follow selection at each
refresh. To update the panel, you have to press `d` again.

`q` closes the panel.

### Piping STDIN

You can read `STDIN` by passing `-` as an argument to `webtop`. For example, if you are watching
a remote file, you could use `tail -f www.access.log | webtop -`.

Note that when you quit, because `STDIN` is still open, the process will not quit until you press
`CTRL-C`. I haven't managed to work around that limitation yet.

### Keybindings

You can press `?` to get an in-program list of all available keybindings.

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

