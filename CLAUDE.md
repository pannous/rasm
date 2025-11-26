# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`rasm` is a Rust library for working with **WebAssembly GC (Garbage Collection) types** from the WebAssembly GC proposal. It provides ergonomic Rust wrappers around WASM GC structs, arrays, and strings, allowing type-safe interaction with WebAssembly's native garbage-collected types.

**Key concept**: This is NOT memory simulation - it uses real WebAssembly GC types (struct, array, ref) from the official GC proposal.

## Core Architecture

### Main Components

1. **`gc_traits.rs`** - Core traits and types:
   - `FromVal` / `ToVal` traits for converting between Rust and WASM values
   - `GcObject<T>` - Generic wrapper for WASM GC structs with field access by name/index
   - `GcString` - Type-safe wrapper for WASM GC string arrays (`array (mut i8)`)
   - `StructExt` trait for struct field operations
   - `InstanceExt` trait for calling WASM functions with typed returns

2. **`rasm-macros/`** - Procedural macros:
   - `gc_struct!` macro generates type-safe Rust wrappers for WASM structs
   - Generates field accessor methods (`.name()`, `.age()`, etc.)
   - Generates setters for mutable fields (`.set_age()`, `.set_name()`, etc.)
   - Implements `GcStructWrapper` trait automatically

3. **`wasm_name_resolver.rs`** - Parses WAT files to extract struct field names from exports

4. **Demo modules** show usage patterns:
   - `gc_struct_demo.rs` - Basic struct creation and field access
   - `gc_object_demo.rs` - Advanced ORM-like API with named fields
   - `gc_string_demo.rs` - String handling examples

5. **`.wat` files** - WebAssembly text format defining GC types:
   - `gc_struct.wat` - Basic struct definitions
   - `gc_strings.wat` - String array helpers
   - `gc_types.wat` - Complex type examples (Person, Point, etc.)
   - `test.wat` - Test cases for `run_test_wat.rs`

## Build and Run

```bash
# Run main demo (currently runs run_test_wat)
cargo run

# Run specific demo by changing main.rs to call:
# - gc_struct_demo()
# - gc_object_demo()
# - run_test_wat()

# Build only
cargo build

# Run tests
cargo test
```

## Development Workflow

### Working with GC Structs

1. **Define struct in WAT**:
```wat
(type $person (struct
  (field $name (ref $string))
  (field $age (mut i32))
))
```

2. **Generate Rust wrapper with macro**:
```rust
gc_struct! {
    Person {
        name: 0 => String,    // field index => Rust type
        age: 1 => i32,
    }
}
```

3. **Use type-safe API**:
```rust
let person = Person::from_val(val, store)?;
let name: String = person.name()?;  // Generated getter
person.set_age(30)?;                // Generated setter (if field is mut)
```

### Key Patterns

- **Field access**: Use generated methods (`.name()`) or generic API (`.get::<T>("name")`)
- **Mutation**: Only `mut` fields in WAT get setters; strings auto-create GC arrays
- **Nested structs**: Use `.get_as::<T>("field")` for typed navigation
- **String creation**: Automatic via `ToVal` trait - just pass `&str` to setters

### Important Constraints

- `GcObject` needs `Instance` parameter for string field mutation (to call `new_string`)
- Store must be mutable for all operations (WASM runtime requirement)
- Field indices in `gc_struct!` macro must match WAT struct field order

## Testing

Tests are embedded in demo modules using `#[test]` attributes:
```bash
cargo test                    # Run all tests
cargo test gc_struct_demo     # Run specific test
```

The `run_test_wat.rs` module provides a test runner for validating WAT files.

## Key Files to Understand

- Start with `README.md` for usage examples
- Read `gc_traits.rs` to understand the API surface
- Check `gc_object_demo.rs` for real-world usage patterns
- Examine `gc_struct.wat` for minimal WASM GC struct example
