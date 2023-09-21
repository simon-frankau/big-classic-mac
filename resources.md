# ROM resources

The ROM contains a set of resources from 0x41AF20 to 0x43CBB0.

| Res type | Res id | Name    | Attributes | ROM Offset | Symbol                         | Description                                           |
|----------|--------|---------|------------|------------|--------------------------------|-------------------------------------------------------|
| PACK     | 0x5    |         | 0x58       | 0x1B122    | `pack_5_Elems68K`              | Transcendental functions                              |
| PACK     | 0x4    |         | 0x58       | 0x1C188    | `pack_4_FP68K`                 | Floating-point arithmetic                             |
| PACK     | 0x7    |         | 0x58       | 0x1D348    | `pack_7_BCD`                   | Binary-decimal conversion                             |
| tcsl     | 0x0    |         | 0x58       | 0x1D896    | `tcsl`                         | Code to display `bbmc`                                |
| bbmc     | 0x0    |         | 0x58       | 0x1D924    | `bbmc`                         | Pictures of dev team                                  |
| SERD     | 0x0    |         | 0x58       | 0x31BAE    | `serd`                         | RAM serial driver                                     |
| DRVR     | 0xA    | .ATP    | 0x58       | 0x324AA    | `drvr_atp`                     | Appletalk Transaction Protocol driver                 |
| DRVR     | 0x9    | .MPP    | 0x58       | 0x3312A    | `drvr_mpp`                     | Low-level network driver                              |
| DRVR     | 0x4    | .Sony   | 0x58       | 0x34680    | `drvr_sony`                    | 3 1/2" Disk driver                                    |
| DRVR     | 0x3    | .Sound  | 0x58       | 0x36C90    | `drvr_sound`                   | Sound driver                                          |
| DRVR     | 0x28   | .XPP    | 0x58       | 0x37028    | `drvr_xpp`                     | Appletalk Filing Protocol & Zone Information Protocol |
| CDEF     | 0x0    |         | 0x58       | 0x37A30    | `cdef_0_button`                | Push button/check box/radio button                    |
| CDEF     | 0x1    |         | 0x58       | 0x37D5A    | `cdef_1_scrollbar`             | Scroll bar                                            |
| KCHR     | 0x0    |         | 0x58       | 0x38254    | `kchr`                         | Keyboard layout                                       |
| KMAP     | 0x0    |         | 0x58       | 0x387C6    | `kmap`                         | Keyboard mapping                                      |
| MBDF     | 0x0    |         | 0x58       | 0x38854    | `mbdf`                         | Draws the menu bar                                    |
| MDEF     | 0x0    |         | 0x58       | 0x38E02    | `mdef`                         | Draws menus                                           |
| WDEF     | 0x1    |         | 0x58       | 0x39212    | `wdef_1`                       | Rounded-corner window                                 |
| WDEF     | 0x0    |         | 0x58       | 0x395AC    | `wdef_0`                       | Regular window                                        |
| CURS     | 0x4    |         | 0x58       | 0x39C48    | `image_wristwatch_16x16`       | Wristwatch cursor                                     |
| CURS     | 0x1    |         | 0x58       | 0x39C94    | `image_ibar_16x16`             | Insertion point cursor                                |
| CURS     | 0x3    |         | 0x58       | 0x39CE0    | `image_spreadsheet_plus_16x16` | Spreadsheet cursor                                    |
| CURS     | 0x2    |         | 0x58       | 0x39D2C    | `image_drawing_plus_16x16`     | Crosshair curson                                      |
| FONT     | 0xC    |         | 0x58       | 0x39D78    | `font_c`                       |                                                       |
| FONT     | 0x0    | Chicago | 0x58       | 0x3AB62    | `font_0_chicago`               | Empty!                                                |
| FONT     | 0x189  |         | 0x58       | 0x3AB6A    | `font_189`                     |                                                       |
| FONT     | 0x18C  |         | 0x58       | 0x3B594    | `font_18c`                     |                                                       |
| FONT     | 0x209  |         | 0x58       | 0x3C204    | `font_209`                     |                                                       |

## Notes

 * A full 80kB of the 256kB ROM is taken up with pictures of the dev
   team!

## TODO

 * Work out the fonts.
