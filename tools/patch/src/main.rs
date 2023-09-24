//
// ROM/disk patcher
//
// Applies a list of patches to a ROM, resource or disk image.
//

use std::fs;

use anyhow::bail;
use clap::{Parser, Subcommand};

////////////////////////////////////////////////////////////////////////
// Command line processing.
//

#[derive(Parser)]
#[command(name = "Patch")]
#[command(author = "Simon Frankau <sgf@arbitrary.name")]
#[command(version = "0.2")]
#[command(about = "Patches Apple 68k ROMs/disk images.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Patch a ROM
    Rom,
    Ptch34,
    Ptch630,
    Disk601,
}

////////////////////////////////////////////////////////////////////////
// Patching
//

#[derive(Debug)]
struct Patch<'a> {
    addr: usize,
    before: &'a [u8],
    after: &'a [u8],
}

impl<'a> Patch<'a> {
    fn apply(&self, data: &mut [u8]) {
        let target = &mut data[self.addr..];
        assert_eq!(
            self.before,
            &target[..self.before.len()],
            "Patch 'before' doesn't match ROM"
        );
        target[..self.after.len()].copy_from_slice(self.after);
    }
}

#[derive(Debug)]
struct PatternPatch<'a> {
    pattern: &'a [u8],
    replacement: &'a [u8],
}

impl<'a> PatternPatch<'a> {
    fn apply(&self, data: &mut [u8]) {
        assert_eq!(
            self.pattern.len(),
            self.replacement.len(),
            "Replacement length must match pattern"
        );
        for idx in 0..(data.len() - self.pattern.len()) {
            let curr = &mut data[idx..];
            if curr.starts_with(self.pattern) {
                curr[..self.replacement.len()].copy_from_slice(self.replacement);
                println!("Patched at 0x{:06x}", idx);
            }
        }
    }
}

fn find_resource(
    prefix: [u8; 4],
    res_type: [char; 4],
    res_id: i16,
    data: &[u8],
) -> anyhow::Result<usize> {
    let needle = [
        prefix[0],
        prefix[1],
        prefix[2],
        prefix[3],
        res_type[0] as u8,
        res_type[1] as u8,
        res_type[2] as u8,
        res_type[3] as u8,
        (res_id >> 8) as u8,
        (res_id & 0xff) as u8,
    ];
    let mut possibilities = Vec::new();
    for idx in 0..(data.len() - 10) {
        if data[idx..].starts_with(&needle) {
            possibilities.push(idx);
        }
    }

    match possibilities.len() {
        0 => bail!(
            "No match for resource {} {}",
            String::from_iter(res_type),
            res_id
        ),
        1 => Ok(possibilities[0]),
        _ => bail!(
            "Multiple matches for resource {} {}: {:?}",
            String::from_iter(res_type),
            res_id,
            possibilities
        ),
    }
}

///////////////////////////////////////////////////////////////////////
// Resource patching.
//

const PTCH_34_PATCHES: [Patch; 4] = [
    Patch {
        addr: 0x074e,
        before: &[0x00, 0x40],
        after: &[0x00, 0xf8],
    },
    Patch {
        addr: 0x0756,
        before: &[0x00, 0x40],
        after: &[0x00, 0xf8],
    },
    Patch {
        addr: 0x075e,
        before: &[0x00, 0x40],
        after: &[0x00, 0xf8],
    },
    Patch {
        addr: 0x0766,
        before: &[0x00, 0x40],
        after: &[0x00, 0xf8],
    },
];

fn patch_ptch_34() -> anyhow::Result<()> {
    let mut data = fs::read("../../system/6.0.1/ptch_34")?;
    patch_ptch_34_aux(&mut data);
    fs::write("../../system/6.0.1/ptch_34.patched", data)?;
    Ok(())
}

fn patch_ptch_34_aux(data: &mut [u8]) {
    for (idx, patch) in PTCH_34_PATCHES.iter().enumerate() {
        println!("Applying patch #{}: {:?}", idx, patch);
        patch.apply(data);
    }
}

const PTCH_630_PATCHES: [PatternPatch; 28] = [
    // JMP
    PatternPatch {
        pattern: &[0x4e, 0xf9, 0x00, 0x40],
        replacement: &[0x4e, 0xf9, 0x00, 0xf8],
    },
    PatternPatch {
        pattern: &[0x4e, 0xf9, 0x00, 0x41],
        replacement: &[0x4e, 0xf9, 0x00, 0xf9],
    },
    PatternPatch {
        pattern: &[0x4e, 0xf9, 0x00, 0x42],
        replacement: &[0x4e, 0xf9, 0x00, 0xfa],
    },
    PatternPatch {
        pattern: &[0x4e, 0xf9, 0x00, 0x43],
        replacement: &[0x4e, 0xf9, 0x00, 0xfb],
    },
    // JSR
    PatternPatch {
        pattern: &[0x4e, 0xb9, 0x00, 0x40],
        replacement: &[0x4e, 0xb9, 0x00, 0xf8],
    },
    PatternPatch {
        pattern: &[0x4e, 0xb9, 0x00, 0x41],
        replacement: &[0x4e, 0xb9, 0x00, 0xf9],
    },
    PatternPatch {
        pattern: &[0x4e, 0xb9, 0x00, 0x42],
        replacement: &[0x4e, 0xb9, 0x00, 0xfa],
    },
    PatternPatch {
        pattern: &[0x4e, 0xb9, 0x00, 0x43],
        replacement: &[0x4e, 0xb9, 0x00, 0xfb],
    },
    // LEA
    PatternPatch {
        pattern: &[0x41, 0xf9, 0x00, 0x40],
        replacement: &[0x41, 0xf9, 0x00, 0xf8],
    },
    PatternPatch {
        pattern: &[0x41, 0xf9, 0x00, 0x41],
        replacement: &[0x41, 0xf9, 0x00, 0xf9],
    },
    PatternPatch {
        pattern: &[0x41, 0xf9, 0x00, 0x42],
        replacement: &[0x41, 0xf9, 0x00, 0xfa],
    },
    PatternPatch {
        pattern: &[0x41, 0xf9, 0x00, 0x43],
        replacement: &[0x41, 0xf9, 0x00, 0xfb],
    },
    // CMP.I variant 1
    PatternPatch {
        pattern: &[0x0c, 0x97, 0x00, 0x40],
        replacement: &[0x0c, 0x97, 0x00, 0xf8],
    },
    PatternPatch {
        pattern: &[0x0c, 0x97, 0x00, 0x41],
        replacement: &[0x0c, 0x97, 0x00, 0xf9],
    },
    PatternPatch {
        pattern: &[0x0c, 0x97, 0x00, 0x42],
        replacement: &[0x0c, 0x97, 0x00, 0xfa],
    },
    PatternPatch {
        pattern: &[0x0c, 0x97, 0x00, 0x43],
        replacement: &[0x0c, 0x97, 0x00, 0xfb],
    },
    // CMP.I variant 2
    PatternPatch {
        pattern: &[0x0c, 0x96, 0x00, 0x40],
        replacement: &[0x0c, 0x96, 0x00, 0xf8],
    },
    PatternPatch {
        pattern: &[0x0c, 0x96, 0x00, 0x41],
        replacement: &[0x0c, 0x96, 0x00, 0xf9],
    },
    PatternPatch {
        pattern: &[0x0c, 0x96, 0x00, 0x42],
        replacement: &[0x0c, 0x96, 0x00, 0xfa],
    },
    PatternPatch {
        pattern: &[0x0c, 0x96, 0x00, 0x43],
        replacement: &[0x0c, 0x96, 0x00, 0xfb],
    },
    // CMP.I variant 3
    PatternPatch {
        pattern: &[0x0c, 0xaf, 0x00, 0x40],
        replacement: &[0x0c, 0xaf, 0x00, 0xf8],
    },
    PatternPatch {
        pattern: &[0x0c, 0xaf, 0x00, 0x41],
        replacement: &[0x0c, 0xaf, 0x00, 0xf9],
    },
    PatternPatch {
        pattern: &[0x0c, 0xaf, 0x00, 0x42],
        replacement: &[0x0c, 0xaf, 0x00, 0xfa],
    },
    PatternPatch {
        pattern: &[0x0c, 0xaf, 0x00, 0x43],
        replacement: &[0x0c, 0xaf, 0x00, 0xfb],
    },
    // PEA
    PatternPatch {
        pattern: &[0x48, 0x79, 0x00, 0x40],
        replacement: &[0x48, 0x79, 0x00, 0xf8],
    },
    PatternPatch {
        pattern: &[0x48, 0x79, 0x00, 0x41],
        replacement: &[0x48, 0x79, 0x00, 0xf9],
    },
    PatternPatch {
        pattern: &[0x48, 0x79, 0x00, 0x42],
        replacement: &[0x48, 0x79, 0x00, 0xfa],
    },
    PatternPatch {
        pattern: &[0x48, 0x79, 0x00, 0x43],
        replacement: &[0x48, 0x79, 0x00, 0xfb],
    },
];

fn patch_ptch_630() -> anyhow::Result<()> {
    let mut data = fs::read("../../system/6.0.1/PTCH_630")?;
    patch_ptch_630_aux(&mut data);
    fs::write("../../system/6.0.1/PTCH_630.patched", data)?;
    Ok(())
}

fn patch_ptch_630_aux(data: &mut [u8]) {
    for (idx, patch) in PTCH_630_PATCHES.iter().enumerate() {
        println!("Applying patch #{}: {:?}", idx, patch);
        patch.apply(data);
    }
}

fn patch_disk_601() -> anyhow::Result<()> {
    let mut data = fs::read("../../system/6.0.1/tools.dsk")?;

    let ptch_34_idx = find_resource([0x60, 0x00, 0x06, 0xe6], ['p', 't', 'c', 'h'], 34, &data)?;
    patch_ptch_34_aux(&mut data[ptch_34_idx..]);
    let ptch_630_idx = find_resource([0x60, 0x00, 0x03c, 0xce], ['P', 'T', 'C', 'H'], 630, &data)?;
    patch_ptch_630_aux(&mut data[ptch_630_idx..]);

    fs::write("../../system/6.0.1/tools.dsk.patched", data)?;
    Ok(())
}
///////////////////////////////////////////////////////////////////////
// ROM patching.
//

const ROM_PATCHES: [Patch; 18] = [
    // Patch debug hooks from 0xf8XXXX to 0xfcXXXX, to avoid ROM
    // clash.
    Patch {
        addr: 0x000b8 + 5,
        before: &[0xf8],
        after: &[0xfc],
    },
    Patch {
        addr: 0x01bf0 + 3,
        before: &[0xf8],
        after: &[0xfc],
    },
    Patch {
        addr: 0x01bfa + 5,
        before: &[0xf8],
        after: &[0xfc],
    },
    // Patch references to absolute ROM addresses.
    Patch {
        addr: 0x00004 + 1,
        before: &[0x40],
        after: &[0xf8],
    },
    Patch {
        addr: 0x00136 + 3,
        before: &[0x41],
        after: &[0xf9],
    },
    Patch {
        addr: 0x00262 + 3,
        before: &[0x40],
        after: &[0xf8],
    },
    Patch {
        addr: 0x00636 + 3,
        before: &[0x41],
        after: &[0xf9],
    },
    Patch {
        addr: 0x00642 + 3,
        before: &[0x41],
        after: &[0xf9],
    },
    Patch {
        addr: 0x00c18 + 3,
        before: &[0x40],
        after: &[0xf8],
    },
    Patch {
        addr: 0x00c30 + 3,
        before: &[0x40],
        after: &[0xf8],
    },
    Patch {
        addr: 0x00c48 + 3,
        before: &[0x40],
        after: &[0xf8],
    },
    Patch {
        addr: 0x01482 + 3,
        before: &[0x40],
        after: &[0xf8],
    },
    // 0x019ec etc. dealt with below.
    Patch {
        addr: 0x01ca0 + 3,
        before: &[0x43],
        after: &[0xfb],
    },
    Patch {
        addr: 0x026cc + 3,
        before: &[0x40],
        after: &[0xf8],
    },
    Patch {
        addr: 0x0285a + 3,
        before: &[0x40],
        after: &[0xf8],
    },
    Patch {
        addr: 0x02860 + 3,
        before: &[0x40],
        after: &[0xf8],
    },
    Patch {
        addr: 0x0288a + 3,
        before: &[0x44],
        after: &[0xfc],
    },
    // TODO: Bunch at 436bc6.
    // TODO: 43d038 - ditto.
    Patch {
        addr: 0x3dd30 + 3,
        before: &[0x43],
        after: &[0xfb],
    },
];

fn patch_rom() -> anyhow::Result<()> {
    let mut data = fs::read("../../ROM.sefdhd")?;

    for (idx, patch) in ROM_PATCHES.iter().enumerate() {
        println!("Applying patch #{}: {:?}", idx, patch);
        patch.apply(&mut data);
    }

    // Sequence of consecutive addresses to patch...
    print!("Patching consecutive addresses at 0x019ec: ");
    for addr in 0x019ec..0x01ae8 {
        if addr % 4 == 1 {
            Patch {
                addr,
                before: &[0x40],
                after: &[0xf8],
            }
            .apply(&mut data);
        }
        print!(".");
    }
    println!(" done!");

    fs::write("../../ROM.patched", data)?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////
// Main entry point.
//

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Rom => patch_rom()?,
        Commands::Ptch34 => patch_ptch_34()?,
        Commands::Ptch630 => patch_ptch_630()?,
        Commands::Disk601 => patch_disk_601()?,
    }

    Ok(())
}
