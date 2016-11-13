extern crate libc;

pub mod gc;
pub mod mem;
pub mod util;
pub use gc::GC;
pub use gc::GCTypeInformation;

#[cfg(test)]
mod tests {
    use std;
    use super::*;

    #[derive(Debug)]
    struct TestStruct {
        gray: bool,
        address_one: usize,
        address_two: usize
    }

    struct TypeInformation {}

    impl GCTypeInformation for TypeInformation {
        fn get_references(&self, address: usize) -> Vec<usize> {
            let reference = address as *mut TestStruct;
            let mut references = vec![];
            unsafe {
                let ref mut object = *reference;
                if object.address_one > 0 {
                    references.push(object.address_one);
                }
                if object.address_two > 0 {
                    references.push(object.address_two);
                }
            }
            return references;
        }

        fn is_gray(&self, address: usize) -> bool {
            let reference = address as *mut TestStruct;
            unsafe {
                let ref mut object = *reference;
                return object.gray;
            }
        }

        fn mark_gray(&self, address: usize) {
            let reference = address as *mut TestStruct;
            unsafe {
                let ref mut object = *reference;
                object.gray = true;
            }
        }

        fn clear_gray(&self, address: usize) {
            let reference = address as *mut TestStruct;
            unsafe {
                let ref mut object = *reference;
                object.gray = false;
            }
        }
    }

    #[test]
    fn it_doesnt_collect_roots() {
        let type_information = TypeInformation{};
        let mut gc = GC::new(type_information);
        let size = std::mem::size_of::<TestStruct>();
        let address = gc.allocate(size).unwrap();
        gc.add_root(address);
        gc.collect();
        assert!(gc.is_address_valid(address));
    }

    #[test]
    fn it_does_collect_unreferenced_objects() {
        let type_information = TypeInformation{};
        let mut gc = GC::new(type_information);
        let size = std::mem::size_of::<TestStruct>();
        let address = gc.allocate(size).unwrap();
        gc.collect();
        assert!(!gc.is_address_valid(address));
    }
}
