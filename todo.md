# Todo
## General
- move over to Funge-98 terminology
- fix up the input system
- clean up/rearrange instructions
- add scrolling to grid display for big play fields
- verbose/logging mode to display each action

## Instructions
### Velocity/Vector/Delta System
- `x`: pop vector, set delta to vector
### Concurrency
- `t`: Split a new instruction pointer
- `@`: destroy single ip (instead of stopping whole program)
- `q`: quit (stop whole program)
### Fingerprints
- `()`: load/unload semantics
- `A-Z`: fingerprint-defined functions
### System
- `=`: pop 0"gnirt" and execute as system command
- `y`: push system info
### Stack Stacks
- `{}`: create/destroy new stack
- `u`: move data between stacks
### Misc. Unimplemented Funge-98 Instructions
- `l`: trefunge only, used in Refunge as lehmer code rearrangement
- `hm`: trefunge only, unused in Refunge
- `z`: Funge-98's explicit no-op, unused in Refunge
