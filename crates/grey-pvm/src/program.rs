//! PVM program loading and standard initialization (Appendix A.7).
//!
//! Standard program format: read-only data + read-write data + stack + arguments.

use crate::memory::{Memory, PageAccess};
use grey_types::constants::{PVM_PAGE_SIZE, PVM_ZONE_SIZE, PVM_INIT_INPUT_SIZE};

/// Parsed PVM program (eq A.37-A.38).
pub struct Program {
    /// c: The instruction bytecode (including jump table).
    pub code: Vec<u8>,
    /// Read-only data segment.
    pub ro_data: Vec<u8>,
    /// Read-write (heap) data segment.
    pub rw_data: Vec<u8>,
    /// Initial heap zero-pages.
    pub heap_pages: u32,
    /// Stack size.
    pub stack_size: u32,
}

/// Standard program initialization Y(p, a) (eq A.37).
///
/// Returns (code, registers, memory) or None if the program blob is invalid.
pub fn initialize_program(
    program_blob: &[u8],
    arguments: &[u8],
) -> Option<(Vec<u8>, [u64; 13], Memory)> {
    // Parse the program header (eq A.38):
    // E3(|o|) ⌢ E3(|w|) ⌢ E2(z) ⌢ E3(s) ⌢ o ⌢ w ⌢ E4(|c|) ⌢ c = p
    if program_blob.len() < 11 {
        return None;
    }

    let mut offset = 0;

    // Read |o| as 3-byte LE (read-only data size)
    let ro_size = read_le_u24(program_blob, &mut offset)? as u32;

    // Read |w| as 3-byte LE (read-write data size)
    let rw_size = read_le_u24(program_blob, &mut offset)? as u32;

    // Read z as 2-byte LE (additional heap pages)
    let heap_pages = read_le_u16(program_blob, &mut offset)? as u32;

    // Read s as 3-byte LE (stack size)
    let stack_size = read_le_u24(program_blob, &mut offset)? as u32;

    // Read read-only data
    if offset + ro_size as usize > program_blob.len() {
        return None;
    }
    let ro_data = &program_blob[offset..offset + ro_size as usize];
    offset += ro_size as usize;

    // Read read-write data
    if offset + rw_size as usize > program_blob.len() {
        return None;
    }
    let rw_data = &program_blob[offset..offset + rw_size as usize];
    offset += rw_size as usize;

    // Read |c| as 4-byte LE (code size)
    if offset + 4 > program_blob.len() {
        return None;
    }
    let code_size = u32::from_le_bytes([
        program_blob[offset],
        program_blob[offset + 1],
        program_blob[offset + 2],
        program_blob[offset + 3],
    ]);
    offset += 4;

    // Read code
    if offset + code_size as usize > program_blob.len() {
        return None;
    }
    let code = program_blob[offset..offset + code_size as usize].to_vec();

    // ZZ = 2^16 (zone size)
    let zz = PVM_ZONE_SIZE;
    let zi = PVM_INIT_INPUT_SIZE;

    // Helper: round up to next page boundary
    let page_round = |x: u32| -> u32 {
        let ps = PVM_PAGE_SIZE;
        ((x + ps - 1) / ps) * ps
    };

    // Helper: round up to next zone boundary
    let zone_round = |x: u32| -> u32 { ((x + zz - 1) / zz) * zz };

    // Check total memory fits in 32-bit address space (eq A.41)
    let ro_zone = zone_round(ro_size);
    let rw_zone = zone_round(rw_size + heap_pages * PVM_PAGE_SIZE);
    let stack_zone = zone_round(stack_size);

    let total = 5u64 * zz as u64 + ro_zone as u64 + rw_zone as u64 + stack_zone as u64 + zi as u64;
    if total > (1u64 << 32) {
        return None;
    }

    // Build memory (eq A.42)
    let mut memory = Memory::new();

    // Read-only data at ZZ
    let ro_base = zz;
    map_region_with_data(&mut memory, ro_base, &ro_data, page_round(ro_size), PageAccess::ReadOnly);

    // Read-write data at 2*ZZ + Z(|o|)
    let rw_base = 2 * zz + zone_round(ro_size);
    map_region_with_data(&mut memory, rw_base, &rw_data, page_round(rw_size + heap_pages * PVM_PAGE_SIZE), PageAccess::ReadWrite);

    // Stack at (2^32 - 2*ZZ - ZI - P(s)) .. (2^32 - 2*ZZ - ZI)
    let stack_top = (1u64 << 32) - 2 * zz as u64 - zi as u64;
    let stack_bottom = stack_top - page_round(stack_size) as u64;
    map_region(&mut memory, stack_bottom as u32, page_round(stack_size), PageAccess::ReadWrite);

    // Arguments at (2^32 - ZZ - ZI)
    let arg_base = (1u64 << 32) - zz as u64 - zi as u64;
    map_region_with_data(&mut memory, arg_base as u32, arguments, page_round(arguments.len() as u32), PageAccess::ReadOnly);

    // Initialize registers (eq A.43)
    let mut registers = [0u64; 13];
    registers[0] = (1u64 << 32) - (1u64 << 16);                     // SP initial
    registers[1] = (1u64 << 32) - 2 * zz as u64 - zi as u64;       // arg end
    registers[7] = (1u64 << 32) - zz as u64 - zi as u64;            // arg base
    registers[8] = arguments.len() as u64;                            // arg length

    Some((code, registers, memory))
}

/// Map a memory region with zero-filled pages.
fn map_region(memory: &mut Memory, base: u32, size: u32, access: PageAccess) {
    if size == 0 {
        return;
    }
    let start_page = base / PVM_PAGE_SIZE;
    let num_pages = (size + PVM_PAGE_SIZE - 1) / PVM_PAGE_SIZE;
    for i in 0..num_pages {
        memory.map_page(start_page + i, access);
    }
}

/// Map a memory region and copy data into it.
fn map_region_with_data(memory: &mut Memory, base: u32, data: &[u8], size: u32, access: PageAccess) {
    if size == 0 {
        return;
    }
    let start_page = base / PVM_PAGE_SIZE;
    let num_pages = (size + PVM_PAGE_SIZE - 1) / PVM_PAGE_SIZE;
    let page_size = PVM_PAGE_SIZE as usize;

    for i in 0..num_pages {
        let data_offset = i as usize * page_size;
        if data_offset < data.len() {
            let end = (data_offset + page_size).min(data.len());
            memory.map_page_with_data(start_page + i, access, &data[data_offset..end]);
        } else {
            memory.map_page(start_page + i, access);
        }
    }
}

fn read_le_u16(data: &[u8], offset: &mut usize) -> Option<u16> {
    if *offset + 2 > data.len() {
        return None;
    }
    let val = u16::from_le_bytes([data[*offset], data[*offset + 1]]);
    *offset += 2;
    Some(val)
}

fn read_le_u24(data: &[u8], offset: &mut usize) -> Option<u32> {
    if *offset + 3 > data.len() {
        return None;
    }
    let val = data[*offset] as u32
        | ((data[*offset + 1] as u32) << 8)
        | ((data[*offset + 2] as u32) << 16);
    *offset += 3;
    Some(val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_program_blob() {
        // Too short to parse header
        assert!(initialize_program(&[0; 5], &[]).is_none());
    }
}
