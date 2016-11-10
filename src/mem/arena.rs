use std::ptr;
use util::memutil;

pub type CellId = usize;
pub type Word = u32;

pub const ArenaSize: usize = 1 << 20; // 1048576
pub const CellSize: usize = 16;

const MinArenaSize: usize = 1 << 20; // 1048576
const ArenaCellMask: usize = (ArenaSize-1); // 1048575
const ArenaMetadataSize: usize = (ArenaSize / 64); // 16384
const ArenaMaxObjMem: usize = (ArenaSize - ArenaMetadataSize); // 1032192
const MinCellId: usize = ArenaMetadataSize / CellSize; // 1024
const MaxCellId: usize = ArenaSize / CellSize; // 65536
const MaxUsableCellId: usize = MaxCellId-2; // 65534
const ArenaUsableCells: usize = MaxCellId - MinCellId; // 64512

const BlocksetBits: usize = 32;
const BlocksetMask: usize = BlocksetBits - 1; // 31
const UnusedBlockWords: usize = MinCellId / BlocksetBits; // 32
const MinBlockWord: usize = UnusedBlockWords; // 32
const MaxBlockWord: usize = ((ArenaMetadataSize/2) / (BlocksetBits /8)); // 2048
const MarkAreaOffset: usize = (ArenaMetadataSize/2); // 8192
const MaxBinSize: usize = 8;

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
    pub fn allocate() -> Result<Arena, memutil::MemoryError> {
        // TODO huge waste of memory, choose proper alignment
        let result = memutil::allocate_aligned(ArenaSize, ArenaSize);
        // TODO handle error
        let (actual_address, address) = result.unwrap();

        return Ok(Arena::new(actual_address, address));
    }

    fn new(actual_address: usize, address: usize) -> Arena {
        return Arena {
            actual_address: actual_address,
            address: address
        };
    }

    pub fn initialize(&self) {
        let word: Word = !0;
        for i in MinBlockWord..MaxBlockWord {
            self.set_mark_word(i, word);
        }
    }

    pub fn set_cell_state(&self, cell: CellId, state: BlockType) -> Option<()> {
        if cell < MinCellId {
            return None;
        }

        if cell > MaxCellId {
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
        if cell < MinCellId {
            return None;
        }

        if cell > MaxCellId {
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

    pub fn allocate_bump(&self, size: usize) -> Option<CellId> {
        // TODO implement
        return None;
    }

    pub fn allocate_fit(&self, size: usize) -> Option<CellId> {
        println!("");

        let cells = self.get_cells_needed_to_store(size);
        println!("allocate_fit - required cells: {}", cells);

        let mut i = 0;
        let mut start = 0;
        let mut free_count = 0;
        for i in MinCellId..(MaxCellId + 1) {
            let cell_state = self.get_cell_state(i).unwrap();
            println!("checking cell {} - {:?}", i, cell_state);
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
        println!("claiming cell {}", start);
        for i in 1..cells {
            println!("claiming cell {}", start + i);
            self.set_cell_state(start + i, BlockType::Extend);
        }

        return Some(start);
    }

    pub fn get_cell_id(&self, address: usize) -> CellId {
        return (address & ArenaCellMask) >> 4;
    }

    pub fn get_address(&self, cell: CellId) -> usize {
        return self.address + (CellSize * cell);
    }

    pub fn free(&self) {
        memutil::free(self.actual_address);
    }

    fn get_cells_needed_to_store(&self, size: usize) -> usize {
        if size % CellSize > 0 {
            return (size / CellSize) + 1;
        } else {
            return size / CellSize;
        }
    }

    /// returns the word containing the mark bit for the given cell
    fn get_mark_word(&self, cell: CellId) -> Word {
        let word_address = self.address + MarkAreaOffset + (cell - (cell & BlocksetMask));
        return unsafe { ptr::read(word_address as *const Word) };
    }

    fn set_mark_word(&self, cell: CellId, word: Word) {
        let word_address = self.address + MarkAreaOffset + (cell - (cell & BlocksetMask));
        unsafe { ptr::write(word_address as *mut Word, word) };
    }

    /// returns the word containing the block bit for the given cell
    fn get_block_word(&self, cell: CellId) -> Word {
        let word_address = self.address + (cell - (cell & BlocksetMask));
        return unsafe { ptr::read(word_address as *const Word) };
    }

    fn set_block_word(&self, cell: CellId, word: Word) {
        let word_address = self.address + (cell - (cell & BlocksetMask));
        unsafe { ptr::write(word_address as *mut Word, word) };
    }

    /// returns the bit index of cell data within a word
    fn get_bit_index(&self, cell: CellId) -> usize {
        return cell & BlocksetMask;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let arena = Arena::allocate().unwrap();
        arena.initialize();

        let addr = arena.get_address(6);
        let cell = arena.get_cell_id(addr);

        assert_eq!(6, cell);

        arena.free();
    }

    #[test]
    fn it_allocates_space() {
        let arena = Arena::allocate().unwrap();
        arena.initialize();

        let address = arena.allocate_fit(44).unwrap();
        let id = arena.get_cell_id(address);

        assert!(id > 0);

        arena.free();
    }
}
