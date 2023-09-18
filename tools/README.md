# Tools for analysing the Mac ROM

I've built some custom tools to pull stuff apart:

 * `extract_traps` decodes the compressed trap table in ROM and
   matches it against the known names of OS and toolbox functions to
   produce a Ghidra script that will label all the trap functions in
   the ROM.
