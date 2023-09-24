# Tools for analysing the Mac ROM

I've built some custom tools to pull stuff apart/adjust it:

 * `extract_traps` decodes the compressed trap table in ROM and
   matches it against the known names of OS and toolbox functions to
   produce a Ghidra script that will label all the trap functions in
   the ROM.
 * `patch` applies patches to the SE FDHD ROM that replace the
   absolute addresses with adjusted absolute addresses, so that the
   ROM can live elsewhere in memory.
