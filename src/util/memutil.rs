use libc;

#[derive(PartialEq, Debug)]
pub enum MemoryError {
    OutOfMemory
}

pub fn allocate(size: usize) -> Result<usize, MemoryError> {
    let address = unsafe{ libc::malloc(size) } as usize;
    if address < 1 {
        return Err(MemoryError::OutOfMemory);
    }
    return Ok(address);
}

pub fn allocate_aligned(size: usize, alignment: usize) -> Result<(usize, usize), MemoryError> {
    let actual_address = unsafe{ libc::malloc(size + alignment) } as usize;
    if actual_address < 1 {
        return Err(MemoryError::OutOfMemory);
    }

    let address = (actual_address + alignment - 1) & ! (alignment - 1);

    return Ok((actual_address, address));
}

pub fn free(address: usize) {
    let mem_address = address as *mut libc::c_void;
    unsafe{ libc::free(mem_address) };
}
