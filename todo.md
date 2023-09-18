# Todo
## General
- move over to Funge-98 terminology
- fix up the input system
- verbose/logging mode to display each action
- move more things away from states

## Instructions
### Concurrency
- `t`: Split a new instruction pointer
- `@`: destroy single ip (instead of stopping whole program)
- `q`: quit (stop whole program)
### Fingerprints
- `()`: load/unload semantics
- `A-Z`: fingerprint-defined functions
### Stack Stacks
- `{}`: create/destroy new stack
- `u`: move data between stacks
### Misc. Unimplemented Funge-98 Instructions
- `l`: trefunge only, used in Refunge as lehmer code rearrangement
- `hm`: trefunge only, unused in Refunge
