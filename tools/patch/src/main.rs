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
    // Patch an individual resource.
    Resource {
        res_type: String,
        res_id: i16,
    },
    // Patch a disk containing resources.
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

#[derive(Debug)]
struct ArrayPatch<'a> {
    start_addr: usize,
    // End address is inclusive.
    end_addr: usize,
    step: usize,
    before: &'a [u8],
    after: &'a [u8],
}

impl<'a> ArrayPatch<'a> {
    fn apply(&self, data: &mut [u8]) {
        let mut addr = self.start_addr;
        while addr <= self.end_addr {
            print!(" 0x{:06x}", addr);
            Patch {
                addr,
                before: self.before,
                after: self.after,
            }
            .apply(data);
            addr += self.step;
        }
        println!();
    }
}

fn find_resource(prefix: &[u8], res_type: &str, res_id: i16, data: &[u8]) -> anyhow::Result<usize> {
    assert_eq!(prefix.len(), 4);
    assert_eq!(res_type.len(), 4);

    let mut res_type_bytes = res_type.as_bytes();
    let mut res_id = res_id;
    if res_type == "boot" {
        // Ugh, special case. Horrible hack.
        res_type_bytes = &[0x00, 0x86, 0x00, 0x17];
        res_id = 0;
    }

    let needle = [
        prefix[0],
        prefix[1],
        prefix[2],
        prefix[3],
        res_type_bytes[0],
        res_type_bytes[1],
        res_type_bytes[2],
        res_type_bytes[3],
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
        0 => bail!("No match for resource {} {}", res_type, res_id),
        1 => Ok(possibilities[0]),
        _ => bail!(
            "Multiple matches for resource {} {}: {:?}",
            res_type,
            res_id,
            possibilities
        ),
    }
}

////////////////////////////////////////////////////////////////////////
// Generic immediate instruction patching.
//

const OP_PREFIXES: [&[u8]; 24] = [
    &[0x04, 0x82], // SUB
    &[0x0c, 0x80], // CMP
    &[0x0c, 0x81], // CMP
    &[0x0c, 0x91], // CMP
    &[0x0c, 0x96], // CMP
    &[0x0c, 0x97], // CMP
    &[0x0c, 0xa1], // CMP
    &[0x0c, 0xa8], // CMP
    &[0x0c, 0xae], // CMP
    &[0x0c, 0xaf], // CMP
    &[0x0c, 0xb8], // CMP
    &[0x20, 0x7c], // MOVEA
    &[0x22, 0x7c], // MOVEA
    &[0x22, 0xbc], // MOVE
    &[0x23, 0x3c], // MOVE
    &[0x2c, 0xbc], // MOVE
    &[0x2e, 0xbc], // MOVE
    &[0x2f, 0x3c], // MOVE
    &[0x2f, 0x7c], // MOVE
    &[0x41, 0xf9], // LEA
    &[0x48, 0x79], // PEA
    &[0x49, 0xf9], // LEA
    &[0x4e, 0xb9], // JSR
    &[0x4e, 0xf9], // JMP
];

const ADDR_SUFFIXES: [(&[u8], &[u8]); 4] = [
    (&[0x00, 0x40], &[0x00, 0xf8]),
    (&[0x00, 0x41], &[0x00, 0xf9]),
    (&[0x00, 0x42], &[0x00, 0xfa]),
    (&[0x00, 0x43], &[0x00, 0xfb]),
];

// Tedious experiments in working around Rust's lifetime stuff.
struct OwnedPatternPatch {
    pattern: Vec<u8>,
    replacement: Vec<u8>,
}

impl OwnedPatternPatch {
    fn to_pattern_patch(&self) -> PatternPatch {
        PatternPatch {
            pattern: &self.pattern,
            replacement: &self.replacement,
        }
    }

    fn apply(&self, data: &mut [u8]) {
        self.to_pattern_patch().apply(data);
    }
}

// Build a set of patches that represent immediate ops on absolute ROM
// addresses.
fn build_op_patches(prefixes: &[&[u8]], suffixes: &[(&[u8], &[u8])]) -> Vec<OwnedPatternPatch> {
    let mut patterns = Vec::new();

    for prefix in prefixes.iter() {
        for (suffix_l, suffix_r) in suffixes.iter() {
            let mut pattern = Vec::from(*prefix);
            pattern.extend_from_slice(suffix_l);

            let mut replacement = Vec::from(*prefix);
            replacement.extend_from_slice(suffix_r);

            patterns.push(OwnedPatternPatch {
                pattern,
                replacement,
            });
        }
    }

    patterns
}

///////////////////////////////////////////////////////////////////////
// Resource patching.
//

const BOOT_1_PATCHES: [Patch; 1] = [Patch {
    addr: 0x2ea,
    before: &[0x00, 0x40],
    after: &[0x00, 0xf8],
}];

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

const CACH_1_PATCHES: [Patch; 2] = [
    Patch {
        addr: 0x58,
        before: &[0x00, 0x40],
        after: &[0x00, 0xf8],
    },
    Patch {
        addr: 0x2b6,
        before: &[0x00, 0x40],
        after: &[0x00, 0xf8],
    },
];

const PTCH_3_PATCHES: [Patch; 2] = [
    Patch {
        addr: 0x19d6,
        before: &[0x00, 0x40],
        after: &[0x00, 0xf8],
    },
    Patch {
        addr: 0x19e4,
        before: &[0x00, 0x40],
        after: &[0x00, 0xf8],
    },
];

struct ResourcePatch {
    res_type: &'static str,
    res_id: i16,
    // If none, apply immediate operand patches.
    patches: Option<&'static [Patch<'static>]>,
    length: usize,
    // Used for finding the ROM on-disk.
    prefix: &'static [u8],
}

impl ResourcePatch {
    fn patch_file(&self) -> anyhow::Result<()> {
        let name = format!("../../system/6.0.1/{}_{}", self.res_type, self.res_id);
        let mut data = fs::read(&name)?;
        self.patch_data(&mut data);
        fs::write(format!("{}.patched", &name), data)?;
        Ok(())
    }

    fn patch_data(&self, data: &mut [u8]) {
        let patches = build_op_patches(&OP_PREFIXES, &ADDR_SUFFIXES);

        if let Some(patches) = self.patches {
            // Specfic patches
            for (idx, patch) in patches.iter().enumerate() {
                println!("Applying patch #{}: {:?}", idx, patch);
                patch.apply(data);
            }
        } else {
            // Generic immediate operand patches.
            for (idx, patch) in patches.iter().enumerate() {
                println!("Applying patch #{}: {:?}", idx, patch.to_pattern_patch());
                patch.apply(data);
            }
        }
    }
}

const RESOURCE_PATCHES: [ResourcePatch; 6] = [
    ResourcePatch {
        res_type: "boot",
        res_id: 1,
        patches: Some(&BOOT_1_PATCHES),
        length: 0x404,
        prefix: &[0x4c, 0x4b, 0x60, 0x00],
    },
    ResourcePatch {
        res_type: "ptch",
        res_id: 34,
        patches: Some(&PTCH_34_PATCHES),
        length: 0x772,
        prefix: &[0x60, 0x00, 0x06, 0xe6],
    },
    ResourcePatch {
        res_type: "PTCH",
        res_id: 117,
        patches: None,
        length: 0x4a48,
        prefix: &[0x60, 0x00, 0x44, 0xA2],
    },
    ResourcePatch {
        res_type: "PTCH",
        res_id: 630,
        patches: None,
        length: 0x41e0,
        prefix: &[0x60, 0x00, 0x03c, 0xce],
    },
    ResourcePatch {
        res_type: "CACH",
        res_id: 1,
        patches: Some(&CACH_1_PATCHES),
        length: 0xb86,
        prefix: &[0x60, 0x00, 0x07, 0xA4],
    },
    ResourcePatch {
        res_type: "ptch",
        res_id: 3,
        patches: Some(&PTCH_3_PATCHES),
        length: 0x1ab8,
        prefix: &[0x60, 0x00, 0x1A, 0xA4],
    },
];

fn patch_resource(res_type: &str, res_id: i16) -> anyhow::Result<()> {
    for res in RESOURCE_PATCHES.iter() {
        if res.res_type == res_type && res.res_id == res_id {
            res.patch_file()?;
            return Ok(());
        }
    }

    bail!("Couldn't find resource {} {}", res_type, res_id);
}

////////////////////////////////////////////////////////////////////////
// Disk patching.
//

fn patch_disk_601() -> anyhow::Result<()> {
    let mut data = fs::read("../../system/6.0.1/tools.dsk")?;

    // The "boot" resource contents also occurs at the start of the
    // disk (I guess it makes sense for boot resources to be placed in
    // the boot block!), so we skip that to avoid multiple matches.
    let patchable_data = &mut data[0x100..];

    for patch in RESOURCE_PATCHES.iter() {
        println!("Patching {} {}", patch.res_type, patch.res_id);
        let idx = find_resource(patch.prefix, patch.res_type, patch.res_id, &patchable_data)?;
        patch.patch_data(&mut patchable_data[idx..][..patch.length]);
    }

    // And let's patch the boot sector while we're at it.
    {
	let boot_data = &mut data[..0x10000];
	let boot_patch = &RESOURCE_PATCHES[0];
	assert_eq!(boot_patch.res_type, "boot");
	assert_eq!(boot_patch.res_id, 1);
        let idx = find_resource(boot_patch.prefix, boot_patch.res_type, boot_patch.res_id, &boot_data)?;
        boot_patch.patch_data(&mut boot_data[idx..][..boot_patch.length]);
    }

    fs::write("../../system/6.0.1/tools.dsk.patched", data)?;
    Ok(())
}

///////////////////////////////////////////////////////////////////////
// ROM patching.
//

const ROM_PATCHES: [Patch; 24] = [
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
    Patch {
        addr: 0x3dd30 + 3,
        before: &[0x43],
        after: &[0xfb],
    },
    // Patch location of SCSI HW.
    Patch {
	addr: 0x004b4 + 3,
	before: &[0x5f, 0xf0],
	after: &[0xfc, 0x10],
    },
    Patch {
	addr: 0x01c74 + 3,
	before: &[0x5f, 0xf0],
	after: &[0xfc, 0x10],
    },
    Patch {
	addr: 0x004bc + 3,
	before: &[0x5f, 0xf2],
	after: &[0xfc, 0x12],
    },
    Patch {
	addr: 0x004c4 + 3,
	before: &[0x5f, 0xf2],
	after: &[0xfc, 0x12],
    },
    Patch {
	addr: 0x004ce + 3,
	before: &[0x5f, 0xf0],
	after: &[0xfc, 0x10],
    },
    // Patch for maximum amount of memory that can be installed in the
    // machine.
    Patch {
	addr: 0x0267e,
	before: &[0x40],
	after: &[0x80],
    },
];

const ROM_ARRAY_PATCHES: [ArrayPatch; 3] = [
    ArrayPatch {
        start_addr: 0x019ec + 1,
        end_addr: 0x01ae4 + 1,
        step: 4,
        before: &[0x40],
        after: &[0xf8],
    },
    ArrayPatch {
        start_addr: 0x36bc6 + 3,
        end_addr: 0x36c0e + 3,
        step: 6,
        before: &[0x43],
        after: &[0xfb],
    },
    ArrayPatch {
        start_addr: 0x3d038 + 3,
        end_addr: 0x3d08c + 3,
        step: 6,
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

    for (idx, patch) in ROM_ARRAY_PATCHES.iter().enumerate() {
        println!("Applying array patch #{}: {:?}", idx, patch);
        patch.apply(&mut data);
    }

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
        Commands::Resource { res_type, res_id } => patch_resource(&res_type, res_id)?,
        Commands::Disk601 => patch_disk_601()?,
    }

    Ok(())
}
