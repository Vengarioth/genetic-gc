use std::ptr;
use util::memutil;

pub type CellId = usize;
pub type Word = u32;

pub const ARENA_SIZE: usize = 1 << 20; // 1048576
pub const CELL_SIZE: usize = 16;

const ARENA_CELL_MASK: usize = (ARENA_SIZE-1); // 1048575
const ARENA_METADATA_SIZE: usize = (ARENA_SIZE / 64); // 16384
const ARENA_MAX_OBJECT_MEMORY: usize = (ARENA_SIZE - ARENA_METADATA_SIZE); // 1032192
const MIN_CELL_ID: usize = ARENA_METADATA_SIZE / CELL_SIZE; // 1024
const MAX_CELL_ID: usize = ARENA_SIZE / CELL_SIZE; // 65536
// const MAX_USABLE_CELL_ID: usize = MAX_CELL_ID-2; // 65534
// const ARENA_USABLE_CELLS: usize = MAX_CELL_ID - MIN_CELL_ID; // 64512

const BLOCKSET_BITS: usize = 32;
const BLOCKSET_MASK: usize = BLOCKSET_BITS - 1; // 31
const UNUSED_BLOCK_WORDS: usize = MIN_CELL_ID / BLOCKSET_BITS; // 32
const MIN_BLOCK_WORD: usize = UNUSED_BLOCK_WORDS; // 32
const MAX_BLOCK_WORD: usize = ((ARENA_METADATA_SIZE/2) / (BLOCKSET_BITS /8)); // 2048
const MARK_AREA_OFFSET: usize = (ARENA_METADATA_SIZE/2); // 8192
// const MAX_BIN_SIZE: usize = 8;

#[derive(PartialEq, Debug)]
pub enum BlockType {
    Extend,
    Free,
    White,
    Black
}

#[derive(Debug)]
pub struct Arena {
    actual_address: usize,
    address: usize
}

impl Arena {
    pub fn new() -> Result<Arena, memutil::MemoryError> {
        // TODO huge waste of memory, choose proper alignment
        let result = memutil::allocate_aligned(ARENA_SIZE, ARENA_SIZE);
        // TODO handle error
        let (actual_address, address) = result.unwrap();

        return Ok(Arena {
            actual_address: actual_address,
            address: address
        });
    }

    pub fn get_arena_address_from_object_address(address: usize) -> usize {
        return address & !(ARENA_CELL_MASK);
    }

    pub fn get_arena_address(&self) -> usize {
        return self.address;
    }

    pub fn initialize(&self) {
        let block_word: Word = 0;
        let mark_word: Word = !0;
        for i in MIN_BLOCK_WORD..MAX_BLOCK_WORD {
            self.set_block_word(i, block_word);
            self.set_mark_word(i, mark_word);
        }
    }

    pub fn get_first_cell(&self) -> CellId {
        return MIN_CELL_ID;
    }

    pub fn get_last_cell(&self) -> CellId {
        return MAX_CELL_ID;
    }

    pub fn set_cell_state(&self, cell: CellId, state: BlockType) -> Option<()> {
        if cell < MIN_CELL_ID {
            return None;
        }

        if cell > MAX_CELL_ID {
            return None;
        }

        let position = self.get_bit_index(cell);
        let mut block_word = self.get_block_word(cell);
        let mut mark_word = self.get_mark_word(cell);

        match state {
            BlockType::Extend => {
                block_word &= !(1 << position);
                mark_word &= !(1 << position);
            }
            BlockType::Free => {
                block_word &= !(1 << position);
                mark_word |= 1 << position;
            }
            BlockType::White => {
                block_word |= 1 << position;
                mark_word &= !(1 << position);
            }
            BlockType::Black => {
                block_word |= 1 << position;
                mark_word |= 1 << position;
            }
        }

        self.set_block_word(cell, block_word);
        self.set_mark_word(cell, mark_word);

        return Some(());
    }

    pub fn get_cell_state(&self, cell: CellId) -> Option<BlockType> {
        if cell < MIN_CELL_ID {
            return None;
        }

        if cell > MAX_CELL_ID {
            return None;
        }

        let position = self.get_bit_index(cell);
        let block_word = self.get_block_word(cell);
        let mark_word = self.get_mark_word(cell);

        let block = (block_word & (1 << position)) >> position;
        let mark = (mark_word & (1 << position)) >> position;

        match (block << 1) + mark {
            0 => return Some(BlockType::Extend),
            1 => return Some(BlockType::Free),
            2 => return Some(BlockType::White),
            3 => return Some(BlockType::Black),
            _ => return None
        }
    }

    pub fn allocate_bump(&self, size: usize) -> Option<usize> {
        // TODO implement
        return None;
    }

    pub fn allocate_fit(&self, size: usize) -> Option<usize> {
        if size > ARENA_MAX_OBJECT_MEMORY {
            return None;
        }

        let cells = self.get_cells_needed_to_store(size);

        let mut start = 0;
        let mut free_count = 0;
        for i in MIN_CELL_ID..(MAX_CELL_ID + 1) {
            let cell_state = self.get_cell_state(i).unwrap();
            if cell_state == BlockType::Free {
                if start == 0 {
                    start = i;
                }
                free_count += 1;
            } else {
                start = 0;
                free_count = 0;
            }

            if free_count == cells {
                break;
            }
        }

        self.set_cell_state(start, BlockType::White);
        for i in 1..cells {
            self.set_cell_state(start + i, BlockType::Extend);
        }

        return Some(self.get_address(start));
    }

    pub fn get_cell_id(&self, address: usize) -> CellId {
        return (address & ARENA_CELL_MASK) >> 4;
    }

    pub fn get_address(&self, cell: CellId) -> usize {
        return self.address + (CELL_SIZE * cell);
    }

    pub fn free(&self) {
        memutil::free(self.actual_address);
    }

    fn get_cells_needed_to_store(&self, size: usize) -> usize {
        if size % CELL_SIZE > 0 {
            return (size / CELL_SIZE) + 1;
        } else {
            return size / CELL_SIZE;
        }
    }

    /// returns the word containing the mark bit for the given cell
    fn get_mark_word(&self, cell: CellId) -> Word {
        let word_address = self.address + MARK_AREA_OFFSET + (cell - (cell & BLOCKSET_MASK));
        return unsafe { ptr::read(word_address as *const Word) };
    }

    fn set_mark_word(&self, cell: CellId, word: Word) {
        let word_address = self.address + MARK_AREA_OFFSET + (cell - (cell & BLOCKSET_MASK));
        unsafe { ptr::write(word_address as *mut Word, word) };
    }

    /// returns the word containing the block bit for the given cell
    fn get_block_word(&self, cell: CellId) -> Word {
        let word_address = self.address + (cell - (cell & BLOCKSET_MASK));
        return unsafe { ptr::read(word_address as *const Word) };
    }

    fn set_block_word(&self, cell: CellId, word: Word) {
        let word_address = self.address + (cell - (cell & BLOCKSET_MASK));
        unsafe { ptr::write(word_address as *mut Word, word) };
    }

    /// returns the bit index of cell data within a word
    fn get_bit_index(&self, cell: CellId) -> usize {
        return cell & BLOCKSET_MASK;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let arena = Arena::new().unwrap();
        arena.initialize();

        let addr = arena.get_address(6);
        let cell = arena.get_cell_id(addr);

        assert_eq!(6, cell);

        arena.free();
    }

    #[test]
    fn it_allocates_space() {
        let arena = Arena::new().unwrap();
        arena.initialize();

        let address = arena.allocate_fit(44).unwrap();
        let id = arena.get_cell_id(address);

        assert!(id > 0);

        arena.free();
    }

    #[test]
    fn it_is_referencable_by_addresses() {
        let arena = Arena::new().unwrap();
        arena.initialize();

        let address = arena.allocate_fit(44).unwrap();

        assert_eq!(Arena::get_arena_address_from_object_address(address), arena.get_arena_address());

        arena.free();
    }
}
