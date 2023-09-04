# Refunge
#### A Rusty Befunge-93 Interpreter

The rules are the same as regular Befunge-93, except for these changes.
For a refresher on these rules, check out [this page](https://esolangs.org/wiki/Befunge#Language_overview).

The playfield takes on the size of the source code instead of being a fixed 80x25.
If the -e flag is not enabled, using `p` on areas outside the playfield will halt the program.

### Most instructions from [Funge-98](https://github.com/catseye/Funge-98/blob/master/doc/funge98.markdown) have been added:
- `[`: rotate the Program Counter 90 degrees anti-clockwise
- `]`: rotate the PC 90 degrees clockwise
- `r`: rotate the PC 180 degrees, "reflect"
- `w`: pop m, n. if m > n turn left; if m < n turn right; otherwise continues straight
- `;`: pass over all instructions and do not execute until reaching another ";"
- `j`: pop n and jump over that many spaces (can be negative to move backwards)
- `a`-`f`: hex integer literals, push 10-15
- `'`: single-tick string mode, push the next character
### In Progress:
- `k`: pop n and repeat the next instruction n times
- `i`: pop a filename string and read onto grid
- `o`: pop a filename and two vectors, read from grid into file
- `s`: pop c and print as char on next space
### Refunge Only:
- `l`: pop n and rearrange data tower based on nth lehmer code