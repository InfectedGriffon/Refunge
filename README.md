# Refunge
#### A Rusty Befunge-93 Interpreter

The rules are the same as regular Befunge-93, except for these changes.
For a refresher on these rules, check out [this page](https://esolangs.org/wiki/Befunge#Language_overview).

The playfield takes on the size of the source code instead of being a fixed 80x25.
If the -e flag is not enabled, using `p` on areas outside the playfield will halt the program.

Multiple new instructions have been added:
- `q`: swaps the data tower into Queue (FIFO) mode.
- `s`: swaps the data tower into Stack (FILO) mode (default).
