# Apple SE FDHD ROM analysis

In order to build a Mac clone that doesn't fully emulate the hardware
(which is possible because the ROM abstracts hardware away), I need to
understand the ROM in order to patch it. This is the dissassembly of
the ROM and relevant other resources that make absoluate references to
the ROM, and tools to patch the ROM/resources.

## Current state

The `patch` tool will patch the SE FDHD ROM and System 6.0.1 resources
such that it'll boot to Finder under a hacked-up copy of Basilisk II
(I've put a branch up at
https://github.com/simon-frankau/macemu/tree/rom_move that builds on
an M1 Mac. This stuff is not terribly portable, so good luck making it
work on your OS.)

The ROM is moved from 0x400000 to 0xf80000, and max RAM has been
increased to 5MB rather than 4MB for any other classic Mac.

The references to debug tooling, around 0xf80000, were moved to
0xfc0000, to make room for the ROM.

### TODO

While the longer term plans are to not just move the ROM but also redo
all hardware access, for now I want to support ROM relocation
solidly. That entails:

 * Check System 6.0.1 resources beyond those in "System".
 * Get System 7 booting with a relocated ROM.

## Ghidra hacking

A-line traps are an integral part of the Mac ROM code, but Ghidra
doesn't recognise them, it views them as invalid instructions. To
handle this, **I added the A-line traps as an instruction type in
Ghidra's 68000 definition. You may also need it to load the
disassembly.**

To do this, I added the following line to `68000.sinc` (after
`addx.l`, to keep alphabetical order):

```
:aline "#"^op015                is op=10 & op015                { __m68k_trap(10:1); }
```

## The plan

My planned approach looks something like this:

 * Goal 0: Establish base camp ☑️
   * Trace the reset vector a little to get the lay of the land. ☑️
   * Look up the memory map, start identifying some hardware access. ☑️
   * Import known symbols and trap names etc. ☑️
 * Goal 1: Identify all code. ☑️
   * Find big chunks of embedded data - pictures, unused areas, etc. ☑️
   * Identify the rest of the resource file ☑️
   * Try to identify missed code, looking for stray 4e75 etc. ☑️
 * Goal 2: Find how it's referenced.
   * Identify all functions that are currently unreferenced.
   * Work out how they're referenced!
 * Goal 3: Identify hack points
   * Find all absolute references to ROM, allowing it to be relocated. ☑️
   * Find all references to HW, allowing it to be replaced.

The plan has been somewhat adjusted by the fact that there are a bunch
of absolute ROM references in the System software, so I'm building
patching tooling for the System files, too. Relevant files live in the
'system' directory.

### Other TODOs

 * Name the other known low memory variables.
 * Look for remaing 4 char codes treated as longs.
 * Investigate structure of FONT resources.

## Notes for reversing

See also NOTES.md.

 * The ROM contains a set of resources, documented in
   [resources.md](./resources.md).
 * Despite what various docs say, the trap tables are at 0x400 and
   0xE00 on the Mac SE FDHD.
 * SCSI variables appear to be at 0xC00.
 * In my reversing of the ROM, trap function names start with an
   underscore.
 * Access to 0xf8XXXX marked with "High memory reference"
 * Access to 0x4XXXXX marked with "Absolute ROM reference"
 * Access to 0x3fXXXX marked with "High RAM reference"
   * It looks like these references are before we know how much RAM
     there is, any it relies on mirroring if there's <4MB to hit the
     top end of whatever RAM is available.
 * Assumption that 8MB is the largest value you'll ever want to deal
   with in RAM is marked with "8MB allocation limit".

### Remapped memory

 * 0xf80000-0xfbffff ROM
 * 0xfc0000-0xfc0007 HW debug stuff
 * 0xfc1000-0xfc127f SCSI
 * 0xfc2000-0xfc2fff SCC read
 * 0xfc3000-0xfc3fff SCC write
 * 0xfc4000-0xfc5fff IWM
 * 0xfc6000-0xfc7fff VIA

The assumption that the largest possible allocation is 0x00800000
(8MB) is replaced with 0x00FC0000 (15.75MB), which is larger than the
maximum amount of RAM I intend to support.

### Memory map

 * 0x000000-0x400000 RAM
 * 0x400000-0x440000 ROM
   * References to absoulute ROM addresses from both ROM and
     System. Stored in 0xc00, 0xc04, 0xc08. Later two only referenced
     by System patches?
 * 0x580000-0x600000 SCSI
   * Used for HD. Referenced via base of 0x5f0000. Uses A4-A6. ROM
     references absolute address.
 * 0x900000-0xA00000 SCC read
   * Z8530 for serial ports. Referenced via 0x9f0000. ROM and System
     references.
 * 0xA00000-0xB00000 Reserved
   * I'm going to pretend this doesn't exist!
 * 0xB00000-0xC00000 SCC write
   * Other part of Z8530
 * 0xD00000-0xE00000 IWM
   * Floppy disk drive, ROM references absolute address
 * 0xE80000-0xF00000 VIA
   * Rockwell or VTI 6522 / 6523, used for ADB & RTC. ROM references
     absolute address.

Other hardware devices include the video display, sound, and interrupt
switch.

### Misc

Might be useful to compare with Macintosh Portable, that has a
different memory map and screen size.

Need to read up on their interrupts.

VBL interrupt doesn't need to sync with screen! Just needs to be
60.15Hz. Need a separate real VBL queue, though (Vertical Retrace
Manager).

Video: 0 is white, 1 is black. Top left is high-order bit at start of
screen buffer. Main and alternate screen buffer controlled by a bit in
the VIA. Video buffers are ScrnBase and ScrnBase - 0x8000. Not all
models have alternate screen buffer!

TODO: ROM overlay in chapter 6 of the HW guide.

Could potentially borrow the pinout from the Mac's backplane
assignment for my hardware implementation.

## References

 * Apple Guide to the Macintosh Family Hardware 2e (PDF)
 * All the Inside Macintosh series
 * http://www.mac.linux-m68k.org/devel/macalmanac.php - Mac Almanac
   II, including low variables and trap numbers.
 * Easter Egg:
   * https://eeggs.com/items/2258.html
   * https://www.nycresistor.com/2012/08/21/ghosts-in-the-rom/
 * https://github.com/unsound/hfsexplorer - for extracting files and
   resources from HFS disks.
