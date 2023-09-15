# Refunge
#### A Rusty Befunge-93 Interpreter

The rules are the same as regular Befunge-93, except for these changes.
For a refresher on these rules, check out [this page](https://esolangs.org/wiki/Befunge#Language_overview).

The playfield takes on the size of the source code instead of being a fixed 80x25.
If the -e flag is not enabled, using `p` on areas outside the playfield will halt the program.
Most instructions from [Funge-98](https://github.com/catseye/Funge-98/blob/master/doc/funge98.markdown) have been added, excluding the list seen in [todo.md](todo.md).
There is also the Refunge-only `l` instruction, which pops a number n and rearranges the stack based on the nth lehmer code.
This instruction will eventually be moved into a fingerprint to keep consistent with Funge-98 once those are implemented. 