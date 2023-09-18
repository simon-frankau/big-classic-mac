# Apple SE FDHD ROM analysis

In order to build a Mac emulator that doesn't fully emulate the
hardware (which is possible because the ROM abstracts hardware away),
I need to understand the ROM in order to patch it. This is the
dissassembly of the ROM.

## Useful notes

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

TODO: ROM overlay in chapter 6.

OS dispatch table is at 0x200.
Toolbox dispatch table is at 0x600.

... Actually seems to be 0xe00 or 0x400 in the ROM!

SCSI vars seem to be at 0xc00, compared to what almanac says.

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

Then

1. The system startup code looks for an appropriate startup device. It
   first checks the internal 3.5-inch floppy drive. If a disk is
   found, it attempts to read it and looks for a System file. If it
   doesn’t find a disk or System file, it checks the default startup
   device specified by the user in the Startup Disk control panel. If
   no default device is specified or if the device specified is not
   connected, it checks for other devices connected to the SCSI port,
   beginning with the internal drive and proceeding successively from
   drive 6 through drive 1. If it doesn’t find a startup device, it
   displays the question-mark disk icon until a disk is inserted. If
   the startup device itself fails, the startup code displays the sad
   Macintosh icon until the computer is turned off.

2. After selecting a startup device, the system startup code reads
   system startup information from the startup device. The system
   startup information is located in the boot blocks, the logical
   blocks 0 and 1 on the startup disk. The boot blocks contain
   important information such as the name of the System file and the
   Finder. The boot blocks are described in detail in the next
   section.

3. The system startup code displays the happy Macintosh icon.

4. The system startup code reads the System file and uses that
   information to initialize the System Error Handler and the Font
   Manager.

5. The system startup code verifies that the necessary hardware is
   available to boot the system software and displays on the startup
   screen an alert box with the message “Welcome to Macintosh.”

6. The system startup code performs miscellaneous tasks: it verifies
   that enough RAM is available to boot the system software, it loads
   and turns on Virtual Memory if it is enabled in the Memory control
   panel, it loads the debugger, if present. (The system startup
   information contains the name of the debugger —usually MacsBug), it
   sets up the disk cache for the file system, and it loads and
   executes CPU-specific software patches. At this point, the system
   begins to trace mouse movement.

7. For any NuBus cards installed, the system startup code executes the
   secondary init code on the card’s declaration ROM.

8. The system startup code loads and initializes all script systems,
   including components for all keyboard input methods. It also
   executes the initialization resources in the System file.

9. The system startup code loads and executes system
   extensions. (System extensions can be located in the Extensions

10. The system startup code launches the Process Manager, which takes
    over at this point and launches the Finder. The Finder then
    displays the desktop and the menu bar. The desktop shows all
    mounted volumes; it also shows any windows that were open the last
    time the computer was shut down. The Memory Manager sets up a
    large, unsegmented application heap, which is divided into
    partitions as applications start up.

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
 * http://www.mac.linux-m68k.org/devel/macalmanac.php - Mac Almanac
   II, including low variables and trap numbers.
