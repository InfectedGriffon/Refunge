# Refunge
#### A Rusty Befunge-93 Interpreter

The rules are the same as regular Befunge-93, except for these changes.
For a refresher on these rules, check out [this page](https://esolangs.org/wiki/Befunge#Language_overview).

The playfield takes on the size of the source code instead of being a fixed 80x25.
If the -e flag is not enabled, using `p` on areas outside the playfield will halt the program.

Multiple new instructions have been added:
- `q`: swaps the data tower into Queue (FIFO) mode.
- `s`: swaps the data tower into Stack (FILO) mode (default).
- `l`: pops n, then rearranges the data tower based on that lehmer code

Borrowed from [Funge-98](https://github.com/catseye/Funge-98/blob/master/doc/funge98.markdown):
- `[`: rotate the Program Counter 90 degrees anti-clockwise
- `]`: rotate the PC 90 degrees clockwise
- `r`: rotate the PC 180 degrees, "reflect"
- `w`: pop m, n. if m > n turn left; if m < n turn right; otherwise continues straight
- `;`: pass over all instructions and do not execute until reaching another ";"
- `j`: pop n and jump over that many spaces (can be negative to move backwards)
- `a`-`f`: hex integer literals, push 10-15