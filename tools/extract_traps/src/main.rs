//
// Trap extractor
//
// Decodes the trap table in the ROM.
//

use std::fs;

// Offset from start of ROM where the offset for the table is.
const TABLE_OFFSET: usize = 0x22;

// Base address of the ROM.
const ROM_BASE: u32 = 0x400000;

// Address of "unimplemented" function.
const UNIMPL: u32 = 0x400768;

fn read_long(mem: &[u8], addr: usize) -> u32 {
    ((mem[addr] as u32) << 24)
        | ((mem[addr + 1] as u32) << 16)
        | ((mem[addr + 2] as u32) << 8)
        | (mem[addr + 3] as u32)
}

fn get_table_start(mem: &[u8]) -> usize {
    read_long(mem, TABLE_OFFSET) as usize
}

#[derive(Debug)]
struct Decoder<'a> {
    mem: &'a [u8],
    table: usize,
    pointer: u32,
}

impl<'a> Decoder<'a> {
    fn new(mem: &'a [u8]) -> Decoder<'a> {
        Decoder {
            mem,
            table: get_table_start(mem),
            pointer: ROM_BASE,
        }
    }
}

impl<'a> Iterator for Decoder<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let b = self.mem[self.table];

        if b == 0x80 {
            self.table += 1;
            return Some(UNIMPL);
        }

        if b == 0xff {
            self.pointer = read_long(self.mem, self.table + 1) + ROM_BASE;
            self.table += 5;
            return Some(self.pointer);
        }

        let offset;
        if b & 0x80 != 0x00 {
            offset = (b & 0x7f) as u32;
            self.table += 1;
        } else {
            offset = ((b as u32) << 8) | (self.mem[self.table + 1] as u32);
            self.table += 2;

            if offset == 0 {
                return None;
            }
        }

        self.pointer += offset * 2;
        if offset & 0x8000 != 0x0000 {
            panic!("Reverse!");
        }
        Some(self.pointer)
    }
}

fn main() -> anyhow::Result<()> {
    let data = fs::read("../../ROM.sefdhd")?;

    let d = Decoder::new(&data);

    for addr in d {
        println!("0x{:6x}", addr);
    }

    Ok(())
}
