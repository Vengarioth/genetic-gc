use std::collections::HashSet;
use std::collections::HashMap;
use mem::arena::BlockType;
use mem::arena::Arena;

pub trait GCTypeInformation {
    fn get_references(&self, address: usize) -> Vec<usize>;
    fn is_gray(&self, address: usize) -> bool;
    fn mark_gray(&self, address: usize);
    fn clear_gray(&self, address: usize);
}

pub struct GC<T> where T : GCTypeInformation {
    root_references: HashSet<usize>,
    arena_map: HashMap<usize, Arena>,
    type_information: T
}

impl<T> GC<T> where T : GCTypeInformation {
    pub fn new(type_information: T) -> GC<T> where T : GCTypeInformation {
        return GC {
            root_references: HashSet::new(),
            arena_map: HashMap::new(),
            type_information: type_information
        };
    }

    pub fn allocate(&mut self, size: usize) -> Option<usize> {
        if self.arena_map.len() < 1 {
            let arena = Arena::new().unwrap();
            arena.initialize();
            self.arena_map.insert(arena.get_arena_address(), arena);
        }

        for (_, arena) in &self.arena_map {
            let address = arena.allocate_fit(size).unwrap();
            return Some(address);
        }

        return None;
    }

    pub fn is_address_valid(&self, address: usize) -> bool {
        if address == 0 {
            return false;
        }

        let arena = self.get_containing_arena(address).unwrap();
        match arena.get_cell_state(arena.get_cell_id(address)).unwrap() {
            BlockType::White => return true,
            BlockType::Black => return true,
            _ => return false,
        }
    }

    pub fn add_root(&mut self, address: usize) -> bool {
        return self.root_references.insert(address);
    }

    pub fn remove_root(&mut self, address: usize) -> bool {
        return self.root_references.remove(&address);
    }

    pub fn collect(&self) {
        self.mark();
        self.sweep();
    }

    fn mark(&self) {
        let mut stack: Vec<usize> = Vec::new();
        for root_reference in &self.root_references {
            stack.push(root_reference.clone());
        }

        while let Some(address) = stack.pop() {
            for reference in self.type_information.get_references(address) {
                stack.push(reference);
            }

            let arena = self.get_containing_arena(address).unwrap();
            arena.set_cell_state(arena.get_cell_id(address), BlockType::Black).unwrap();
        }
    }

    fn sweep(&self) {
        for (_, arena) in &self.arena_map {
            for cellId in arena.get_first_cell()..arena.get_last_cell() {
                let mut clear = false;
                match arena.get_cell_state(cellId).unwrap() {
                    BlockType::Black => {
                        arena.set_cell_state(cellId, BlockType::White);
                        clear = false;
                    }
                    BlockType::White => {
                        arena.set_cell_state(cellId, BlockType::Free);
                        clear = true;
                    }
                    BlockType::Extend => {
                        if clear {
                            arena.set_cell_state(cellId, BlockType::Free);
                        }
                    }
                    BlockType::Free => {
                        clear = false;
                    }
                }
            }
        }
    }

    fn compact(&self) {

    }

    fn get_containing_arena(&self, address: usize) -> Option<&Arena> {
        let address = Arena::get_arena_address_from_object_address(address);
        return self.arena_map.get(&address);
    }
}
