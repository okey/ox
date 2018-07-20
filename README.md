ox
==

ox is an NWScript bytecode disassembler. It also supports reassembling its own output.
One day it might be a proper decompiler.

It is based on an old and incomplete description of NWN2-era NWScript opcodes, plus reverse-engineering some newer pieces added in DA:O &c.

There is no schedule for development.

### Building and running ox

Currently ox builds and runs with rust 1.27 stable. To run it on an NWScript file, you the appropriate definitions file. They are IP of the relevant companies so you must use your own copies, I can't provide any here.

### TODO

Executable:

- [ ] NWN definitions mode
- [ ] CLI interface needs better names
- [ ] documentation
- [ ] better I/O error handling than just panicking
- [ ] Stream input files in case they are large
- [ ] Additional input/output formatting configuration options
- [ ] Logging

Testing:

- [ ] add existing shell script tests to this repo or replace with purely Rust-based testing
- [ ] unit test suite
- [ ] circular mode for testing (disassemble a file then recompile it, and vice versa)

Opcodes:

- [ ] Improve output: start-of-code indices
- [ ] Improve output: calculate jump targets where possible (may require NWScript stack impl)
- [ ] Generate known opcode data to reduce hardcoding (maybe use a DSL)
- [ ] Warn or fail on duplicate opcode entries
- [ ] Options for engine type variations

Parsing:

- [ ] Test grammar
- [ ] Handle array types properly
- [ ] Do a preprocessor pass or incorporate preprocessor constants as literals into grammar (former is easy, latter might involve forking rust-peg)
- [ ] Handle resource strings (R"") better

Assembly:

- [ ] Report line number in errors
- [ ] Stricter parsing

Disassembly:

- [ ] Enforce opcode <-> type constraints
- [ ] Investigate what the compiler does to NWScript strings with control characters in them
- [ ] Support output formatting configuration
- [ ] Redesign payload structs
- [ ] Stricter type and size handling

Miscellaneous:

- [ ] Clean up unused code and includes once interal design is finalised
- [ ] Use byte count in read_exact macro
- [ ] Use nicer match statements per Rust 1.26 - this will reduce quite a bit of &/ref cruft