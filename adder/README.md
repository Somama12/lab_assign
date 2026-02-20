# Adder Compiler

A simple compiler for the Adder language (supports 32-bit integers, add1, sub1, negate).

## How to Build and Run Tests

1. `make test/<name>.run`   (e.g., `make test/37.run`)
2. `./test/<name>.run`      to see the result

## Features Implemented
- Parsing S-expressions to AST
- Code generation to x86-64 assembly (leaves result in rax)
- Negate implemented via imul with -1
- Works on Apple Silicon via x86_64 target + Rosetta

Tested on macOS with 10 test cases.
See transcript.txt for demonstrations.
