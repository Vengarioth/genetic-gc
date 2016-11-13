# genetic-gc
_a machine learning approach to garbage collection_

> To unleash machine learning onto a garbage collector, one has to write a garbage collector first.

## State of the GC
1. make it work __✓__
2. make it pretty
3. make it fast

## State of the model
1. make it work
2. make it pretty
3. make it fast

## API
```rust
  // TypeInformation and TestStruct omitted
let type_information = TypeInformation{};
let mut gc = GC::new(type_information);
let size = std::mem::size_of::<TestStruct>();
let address_one = gc.allocate(size).unwrap();
let address_two = gc.allocate(size).unwrap();

gc.add_root(address_one);
gc.collect();

// address_one will still be alive since it's a root reference
assert!(gc.is_address_valid(address_one));

// address_two will be gone since nothing refers to it
assert!(!gc.is_address_valid(address_two));
```

## Licence

__MIT__
