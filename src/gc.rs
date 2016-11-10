use std::collections::HashSet;
use mem::arena::Arena;

pub struct GC {
    root_references: HashSet<usize>,
    arenas: Vec<Arena>
}

impl GC {
    pub fn new() -> GC {
        return GC {
            root_references: HashSet::new(),
            arenas: Vec::new()
        };
    }

    pub fn allocate(&mut self, size: usize) -> usize {
        if self.arenas.len() < 1 {
            let arena = Arena::allocate().unwrap();
            self.arenas.push(arena);
        }

        for arena in &self.arenas {
            return arena.allocate_fit(size).unwrap();
        }

        return 0;
    }

    pub fn add_root(&mut self, address: usize) -> bool {
        return self.root_references.insert(address);
    }

    pub fn remove_root(&mut self, address: usize) -> bool {
        return self.root_references.remove(&address);
    }
}
