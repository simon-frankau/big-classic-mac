//
// ROM patcher
//
// Applies a list of patches to a ROM.
//

use std::fs;

#[derive(Debug)]
struct Patch<'a> {
    addr: usize,
    before: &'a [u8],
    after: &'a [u8],
}

impl<'a> Patch<'a> {
    fn apply(&self, data: &mut [u8]) {
	let target = &mut data[self.addr..];
	assert_eq!(self.before, &target[..self.before.len()], "Patch 'before' doesn't match ROM");
	target[..self.after.len()].copy_from_slice(&self.after);
    }
}

const PATCHES: [Patch; 18] = [
    // Patch debug hooks from 0xf8XXXX to 0xfcXXXX, to avoid ROM
    // clash.
    Patch { addr: 0x000b8 + 5, before: &[0xf8], after: &[0xfc] },
    Patch { addr: 0x01bf0 + 3, before: &[0xf8], after: &[0xfc] },
    Patch { addr: 0x01bfa + 5, before: &[0xf8], after: &[0xfc] },
    // Patch references to absolute ROM addresses.
    Patch { addr: 0x00004 + 1, before: &[0x40], after: &[0xf8] },
    Patch { addr: 0x00136 + 3, before: &[0x41], after: &[0xf9] },
    Patch { addr: 0x00262 + 3, before: &[0x40], after: &[0xf8] },
    Patch { addr: 0x00636 + 3, before: &[0x41], after: &[0xf9] },
    Patch { addr: 0x00642 + 3, before: &[0x41], after: &[0xf9] },
    Patch { addr: 0x00c18 + 3, before: &[0x40], after: &[0xf8] },
    Patch { addr: 0x00c30 + 3, before: &[0x40], after: &[0xf8] },
    Patch { addr: 0x00c48 + 3, before: &[0x40], after: &[0xf8] },
    Patch { addr: 0x01482 + 3, before: &[0x40], after: &[0xf8] },
    // 0x019ec etc. dealt with below.
    Patch { addr: 0x01ca0 + 3, before: &[0x43], after: &[0xfb] },
    Patch { addr: 0x026cc + 3, before: &[0x40], after: &[0xf8] },
    Patch { addr: 0x0285a + 3, before: &[0x40], after: &[0xf8] },
    Patch { addr: 0x02860 + 3, before: &[0x40], after: &[0xf8] },
    Patch { addr: 0x0288a + 3, before: &[0x44], after: &[0xfc] },
    // TODO: Bunch at 436bc6.
    // TODO: 43d038 - ditto.
    Patch { addr: 0x3dd30 + 3, before: &[0x43], after: &[0xfb] },
    
];  

fn main() -> anyhow::Result<()> {
    let mut data = fs::read("../../ROM.sefdhd")?;

    for (idx, patch) in PATCHES.iter().enumerate() {
	println!("Applying patch #{}: {:?}", idx, patch);
	patch.apply(&mut data);
    }

    // Sequence of consecutive addresses to patch...
    for addr in 0x019ec..0x01ae8 {
	if addr % 4 == 1 {
	    Patch { addr, before: &[0x40], after: &[0xf8] }.apply(&mut data);
	}
    }
    
    fs::write("../../ROM.patched", data)?;
    Ok(())
}
