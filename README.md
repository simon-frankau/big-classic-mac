# Apple SE FDHD ROM analysis

In order to build a Mac emulator that doesn't fully emulate the
hardware (which is possible because the ROM abstracts hardware away),
I need to understand the ROM in order to patch it. This is the
dissassembly of the ROM.

## Useful notes

### Misc

Might be useful to compare with Macintosh Portable, that has a
different memory map and screen size.

Need to reed up on their interrupts.

VBL interrupt doesn't need to sync with screen! Just needs to be
60.15Hz. Need a separate real VBL queue, though (Vertical Retrace
Manager).

Video: 0 is white, 1 is black. Top left is high-order bit at start of
screen buffer. Main and alternate screen buffer controlled by a bit in
the VIA. Video buffers are ScrnBase and ScrnBase - 0x8000. Not all
models have alternate screen buffer!

TODO: ROM overlay in chapter 6.

OS dispatch table is at 0x200.
Toolbox dispatch table is at 0x600.

### Boot sequence

The system initialization sequence is subject to change; the
information in this section is provided for informational purposes
only.

1. Hardware is initialized. The initialization code performs a set of
   diagnostic tests to verify functionality of some vital hardware
   components. If the diagnostics succeed, the initialization code
   initializes these hardware components. If diagnostics fail, the
   initialization code issues diagnostic tones to indicate the type of
   hardware failure. The initialization code determines how much RAM
   is available and tests it, then validates the parameter RAM
   (PRAM). Parameter RAM contains a user’s preferences for settings of
   various control panel settings and port configurations.

   The initialization code determines the global timing variables,
   TimeDBRA, TimeSCCDB, andTimeSCSIDB. (See “Global Timing Variables”
   on page 9-9 for more information) and initializes the Resource
   Manager, Notification Manager, Time Manager, and Deferred Task
   Manager.

2. On machines with expansion slots, the initialization code
   initializes the Slot Manager. The Slot Manager then initializes any
   installed cards by executing the primary initialization code in
   each card’s declaration ROM. Video expansion cards, including
   built-in video, initialize themselves by determining the type of
   connected monitor, and then set the display to 1 bit per pixel, and
   display a gray screen (alternating black and white dots).

3. The initialization code initializes the Vertical Retrace Manager
   and Gestalt Manager. ROM drivers for all built-in functionality are
   installed in the unit table and initialized. The initialization
   code initializes the Apple Desktop Bus (ADB) Manager that then
   initializes each ADB device. The initialization code initializes
   the Sound Manager and SCSI Manager.

4. The initialization code loads drivers from all on-line SCSI
   devices.

5. The initialization code chooses the boot device, and calls the boot
   blocks to begin initialization of the System Software.

### Memory map

 * 0x000000-0x400000 RAM
 * 0x400000-0x440000 ROM
 * 0x580000-0x600000 SCSI
 * 0x900000-0xA00000 SCC read
 * 0xA00000-0xB00000 Reserved
 * 0xB00000-0xC00000 SCC write
 * 0xD00000-0xE00000 IWM
 * 0xE80000-0xF00000 VIA

## References

 * Apple Guide to the Macintosh Family Hardware 2e (PDF)
