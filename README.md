# Refunge

#### A Rusty Befunge-98 Interpreter

This is an interpreter for [Funge-98](https://github.com/catseye/Funge-98/blob/master/doc/funge98.markdown), specifically the two-dimensional Befunge variant.
Almost all instructions have been implemented, including concurrency. For the full list of unimplemented instructions see the [todo](#todo) list.

Since Trefunge is not going to be implemented, Refunge uses the Trefunge-only `l`, `h`, and `m` instructions for custom instructions.
`l` corresponds to the "permute" instruction, which pops a value 'n' from the stack
and permutes the stack based on the nth [Lehmer Code](https://en.wikipedia.org/wiki/Lehmer_code).
`h` and `m` are currently unimplemented, but planned to have uses in future versions.

Refunge contains a TUI built with [ratatui](https://crates.io/crates/ratatui) which can be disabled with the -q arg.
There is also -s for script mode, which starts the instruction pointer at the first line that starts with a non-# character.
Additionally, there are some utility options for:

- starting the tui mode `p`aused
- `j`umping some ticks forward before starting the TUI
- `l`ogging the stack(s) after exiting
- setting a `m`aximum amount of ticks to run for

While in the TUI, the following keyboard shortcuts are available:

- ','/'.': slow down/speed up
- right arrow: tick while paused
- p: pause/unpause
- h/j/k/l: scroll grid display (vim style)
- i/o: scroll output text up/down
- r: restart interpretation
- q: exit after Refunge finished
- ctrl-c: quit immediately

### Todo

- add functionality to `h` and `m`
- add fingerprints (`()A-Z` commands)
- use spade for 2d environment instead of character vectors
