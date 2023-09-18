//
// Trap extractor
//
// Decodes the trap table in the ROM.
//

use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

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
	// DIY signed arithmetic as I'm lazy.
        if offset & 0x4000 != 0x0000 {
	    self.pointer -= 0x10000;
        }
        Some(self.pointer)
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
where
    P: AsRef<Path>,
{
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn trap_to_idx(trap_num: u32) -> usize {
    (if trap_num & 0x0800 != 0 {
        // Toolbox
        trap_num & 0x1ff
    } else {
        // OS
        (trap_num & 0xff) + 0x200
    }) as usize
}

fn idx_to_trap(idx: usize) -> u32 {
    (if idx >= 0x200 {
        0xa000 + idx - 0x200
    } else {
        0xa800 + idx
    }) as u32
}

fn read_traps<P>(filename: P) -> anyhow::Result<HashMap<usize, String>>
where
    P: AsRef<Path>,
{
    let lines = read_lines(filename)?;
    let mut map = HashMap::new();
    for line in lines {
        let line_unwrapped = line?;
        let mut bits = line_unwrapped.split(',');
        let addr_str = bits.next().unwrap();
        let fn_str = bits.next().unwrap();
        assert_eq!(bits.next(), None);
        if let Some(existing) = map.insert(
            trap_to_idx(u32::from_str_radix(addr_str, 16)?),
            fn_str.to_string(),
        ) {
            // "A12F,_PPostEvent" was taken out of "trap_names.txt" as
            // it triggered this, and it's clearly the same trap under
            // a different name (vs. "_PostEvent"), with different
            // return conventions.
            //
            // Ditto "A87D,_CloseCPort" with ClosePort.
            //
            // "FrameRoundRect" should be A8B0, the Almanac is
            // incorrect; this check found something real!
            //
            // Synonyms "A9EB,_FP68K" and "A9EC,_Elems68K" also
            // removed.
            panic!(
                "Multiple entries for 0x{}: {} vs {}",
                addr_str, fn_str, existing
            );
        }
    }
    Ok(map)
}

fn main() -> anyhow::Result<()> {
    let traps = read_traps("trap_names.txt")?;
    let data = fs::read("../../ROM.sefdhd")?;

    let mut d = Decoder::new(&data);

    for (idx, addr) in (&mut d).enumerate() {
	let opt_name = traps.get(&idx);

	if addr == UNIMPL && opt_name.is_none() {
	    // No name found and unimplemented function?
	    // No label needed!
	    continue;
	}
	
        let name = if let Some(name) = opt_name {
            name.clone()
        } else {
            format!("_Unk_{:04X}", idx_to_trap(idx))
        };
	
        println!("createLabel(currentProgram.parseAddress(\"0x{:06X}\")[0], \"{}\", True)", addr, name);
    }

    eprintln!("Final table pointer: 0x{:06X}", d.table);

    Ok(())
}
