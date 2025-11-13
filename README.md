# WebAssembly GC Types Demo

This project demonstrates **real WebAssembly GC (Garbage Collection) types** with actual `struct`, `array`, and reference types from the WebAssembly GC proposal.

## Run the Demo

```bash
cargo run --bin wasm_demo
```

## What's Demonstrated

The `gc_types.wat` file uses the WebAssembly GC proposal to create real typed structs and arrays, not memory simulations:

### 1. **Struct Types**

Real GC structs defined with the `struct` keyword:
```wat
(type $point (struct
  (field $x (mut f64))
  (field $y (mut f64))
))

(type $person (struct
  (field $name (ref $string))
  (field $age (mut i32))
  (field $email (ref null $string))
))
```

Features:
- Mutable fields: `(mut f64)`, `(mut i32)`
- Immutable fields: default (no `mut`)
- Created with `struct.new`
- Accessed with `struct.get` and `struct.set`

### 2. **Array Types**

GC-managed arrays:
```wat
(type $string (array (mut i8)))
(type $person_list (array (mut (ref null $person))))
```

Operations:
- `array.new_default` - Create array
- `array.get` / `array.set` - Access elements
- `array.len` - Get length

### 3. **Reference Types**

Actual typed references managed by GC:
- `(ref $type)` - Non-null reference
- `(ref null $type)` - Nullable reference (option type)
- `anyref` - Reference to any GC object
- `eqref` - Reference supporting equality

### 4. **Nested Structs**

Structs containing other struct references:
```wat
(type $employee (struct
  (field $person (ref $person))
  (field $id i32)
  (field $salary (mut f64))
))
```

Access nested fields through chaining:
```wat
(struct.get $person $age
  (struct.get $employee $person (local.get $emp)))
```

### 5. **Garbage Collection**

All structs and arrays are automatically garbage collected - no manual memory management needed!

## BLACK MAGIC: Direct GC Object Introspection

The demo includes **"black magic"** code that reads GC struct fields directly from Rust **without calling WASM helper functions!**

This uses Wasmtime's GC introspection APIs:

```rust
// Get a GC object reference from a Val
let anyref = val.unwrap_anyref().expect("not an anyref");
let struct_ref = anyref.unwrap_struct(&store)?;

// Read field by index - NO WASM HELPER NEEDED!
let field_val = struct_ref.field(&mut store, 0)?;
let x = field_val.unwrap_f64();
```

**The API chain:**
1. `Val::unwrap_anyref()` → `Option<Rooted<AnyRef>>`
2. `AnyRef::unwrap_struct(&store)` → `Result<Rooted<StructRef>>`
3. `StructRef::field(&mut store, index)` → `Result<Val>`

**Benefits:**
- No need to export getter functions from WASM
- Direct access to GC heap from host
- Works with any GC struct without prior knowledge
- Can iterate all fields generically

See the `inspect_struct()` function in `src/main.rs:262` for a generic inspector that works with **any** GC struct!

## BEAUTIFUL API: Trait-Based Access

We've wrapped the "black magic" in beautiful, ergonomic Rust traits!

### Generic `.get<T>()` Method

Type-safe field access with automatic conversion:

```rust
use gc_traits::StructExt;

// Generic get with type parameter
let x: f64 = point_struct.get(&mut store, 0)?;
let age: i32 = person_struct.get(&mut store, 1)?;

// Automatic UTF-8 string conversion!
let name: String = person_struct.get(&mut store, 0)?;

// Type inference works too!
let salary = employee_struct.get::<f64>(&mut store, 2)?;
```

**Supported Types:**
- Primitives: `i32`, `i64`, `f32`, `f64`, `bool`
- Strings: `String` (automatic UTF-8 conversion from GC byte arrays)
- Nested structs: `Rooted<StructRef>` (or use `.get_struct()` helper)

### Access Fields by Name!

The ultimate ergonomic API - access struct fields by name with the store completely hidden:

```rust
// Create GcObject - it owns the store
let person = GcObject::new(person_val, store)?;

// Access by field name - store completely hidden!
let name: String = person.get("name")?;
let age: i32 = person.get("age")?;
let is_null = person.is_null("email")?;

// Also works with numeric indices
let name: String = person.get(0)?;
let age: i32 = person.get(1)?;
```

**How it works:**
- `GcObject` wraps the store in `RefCell` for interior mutability
- `get()` method uses `FieldIndex` trait accepting both `usize` and `&str`
- Field names validated against struct metadata
- Store access completely transparent to caller

**Supported types:**
- `String` - Automatic UTF-8 conversion from GC byte arrays
- `i32`, `i64`, `f32`, `f64` - Direct extraction
- `bool` - Boolean conversion (0 = false, non-zero = true)
- `GcString` - For working with GC array strings directly

**Supported field names:**
- Person: `name`, `age`, `email`
- Point: `x`, `y`
- Employee: `person`, `id`, `salary`

_Note: Currently uses a hardcoded field name mapping. Future versions will parse the WASM name section for automatic support._

### Nested Struct Navigation

Access nested GC structs with `.get_struct()`:

```rust
// Get nested person from employee
let nested_person = employee.get_struct(&mut store, 0)?;
let nested_age: i32 = nested_person.get(&mut store, 1)?;
```

### Option Type Helpers

Check nullable fields with `.is_null()`:

```rust
if person.is_null(&mut store, 2)? {
    println!("No email");
}
```

### Type-Safe Wrappers

Define strongly-typed wrappers using the `gc_struct!` macro:

```rust
gc_struct! {
    Point {
        x: 0 => f64,
        y: 1 => f64,
    }
}

// Now use with named methods!
let point = Point::from_val(&store, val)?;
let x = point.x(&mut store)?;
let y = point.y(&mut store)?;
```

**Benefits:**
- ✨ Type-safe: `get<T>()` enforces correct types at compile time
- ✨ Ergonomic: Clean, readable syntax
- ✨ Composable: Chain method calls naturally
- ✨ Zero runtime overhead: All inline/monomorphized

See `src/gc_traits.rs` for the complete implementation!

## CONTEXTUAL API: Hide the Store Parameter

For the ultimate ergonomic experience, wrap your GC context to eliminate passing `&mut store` to every method:

```rust
use gc_traits::{GcContext, IntoContextual};

// Create a context once - it carries the store
let mut ctx = GcContext::new(&mut store);

// Wrap a GC struct with the context
let mut person_ctx = person.with_context(&mut ctx)?;

// Now call methods WITHOUT passing &mut store!
let age: i32 = person_ctx.get(1)?;
let is_null = person_ctx.is_null(2)?;
```

**Note:** Due to Rust lifetime constraints with mutable references, the contextual API works best for direct field access. For nested struct navigation, use the `StructExt` trait with explicit `&mut store` parameters.

**Benefits:**
- ✨ Clean syntax: No `&mut store` clutter
- ✨ Context reuse: Create once, use for multiple operations
- ✨ Type-safe: Still uses generic `.get<T>()` under the hood

## GcString: Type-Safe GC Array Wrapper

`GcString` wraps WebAssembly GC arrays (`array (mut i8)`) as a proper Rust type, similar to how `GcObject` wraps GC structs:

```rust
use gc_traits::GcString;

// Get a string field as GcString (using FromVal trait)
let gc_string: GcString = person.get(0)?;

// Convert to Rust String
let text = gc_string.to_string(&mut store)?;  // "Bob 🎉"

// Get length
let len = gc_string.len(&store)?;  // 8 bytes (UTF-8)

// Check if empty
let is_empty = gc_string.is_empty(&store)?;  // false
```

**API:**
- `GcString::from_val(store, val)` - Create from a Val
- `gc_string.to_string(&mut store)` - Convert to Rust String with UTF-8 validation
- `gc_string.len(&store)` - Get byte length
- `gc_string.is_empty(&store)` - Check if empty
- `gc_string.to_val()` - Convert back to Val for WASM calls

**Old API (OldGcString):**
The legacy static helper methods are still available as `OldGcString` for compatibility:
- `OldGcString::write_to_mem()` - Write string to linear memory
- `OldGcString::to_string()` - Read via WASM helper function
- `OldGcString::to_string_direct()` - Read via array introspection

**Benefits:**
- ✨ Type-safe: Wraps `Rooted<ArrayRef>` as a proper type
- ✨ Consistent: Follows same pattern as `GcObject` for structs
- ✨ Clean API: Methods on the type itself, not static helpers
- ✨ UTF-8 validated: Safe conversion to Rust String

## Demo Output

```
=== WebAssembly GC Types Demo ===

1. Working with GC Struct Types (Point)
  p1 coordinates: (3, 4)
  After move(1, 2): (4, 6)
  Distance between p1 and p2: 4.24

2. Working with Nested Structs (Employee contains Person)
  Age: 30
  After birthday: 31
  After 10% raise: $82500.00

3. Working with GC Arrays
  Array length: 3
  Person[0] age: 25
  Person[1] age: 35

BLACK MAGIC: Direct GC Object Access
=====================================

Reading Point p1 fields WITHOUT helper functions:
  Direct field[0] (x): 4
  Direct field[1] (y): 6

Generic struct inspector:
  Inspecting 'Point p1':
    field[0]: f64 = 4
    field[1]: f64 = 6

  Inspecting 'Employee':
    field[0]: ref = <object>
    field[1]: i32 = 12345
    field[2]: f64 = 82500

✨ No WASM helper functions needed - direct GC heap access from Rust!
```

## Other Examples

### Basic WAT Demo (`example.wat`)
Simple arithmetic functions (add, multiply, factorial)

### Interface Types Pattern Demo (`interface_types.wat`)
Shows how to implement high-level patterns using linear memory (pre-GC approach)

## Resources

- [WebAssembly GC Proposal](https://github.com/WebAssembly/gc)
- [WebAssembly Specification](https://webassembly.github.io/spec/)
- [Wasmtime GC Support](https://wasmtime.dev/)
- [WAT Format](https://developer.mozilla.org/en-US/docs/WebAssembly/Understanding_the_text_format)
