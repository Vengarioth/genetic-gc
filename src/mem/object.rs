pub struct Object {
    address: usize,
    arena_size: usize
}

impl Object {
    pub fn new(address: usize, arena_size: usize) -> Object {
        return Object {
            address: address,
            arena_size: arena_size
        };
    }

    pub fn get_arena_address(&self) -> usize {
        return 0;
    }
}
