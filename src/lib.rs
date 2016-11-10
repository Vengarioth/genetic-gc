extern crate libc;

pub mod gc;
pub mod mem;
pub mod util;
pub use gc::GC;

pub trait GCTypeInformation {
    fn get_references(&self, address: usize) -> Vec<usize>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TypeInformation {}

    impl GCTypeInformation for TypeInformation {
        fn get_references(&self, address: usize) -> Vec<usize> {
            return vec![0, 0, 0];
        }
    }

    #[test]
    fn it_works() {
        let mut gc = GC::new();
        let obj_one = gc.allocate(30);
        let obj_two = gc.allocate(15);
        gc.add_root(obj_one);
        gc.add_root(obj_two);
        gc.remove_root(obj_two);
        // gc.mark();
        // gc.sweep();
    }
}
