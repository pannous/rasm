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

### Even More Ergonomic: gc_struct! Macro!

For the ultimate developer experience with **zero boilerplate**, use the enhanced `gc_struct!` macro:

```rust
// Just define your struct - everything else is generated!
gc_struct! {
    Person {
        name: 0 => String,
        age: 1 => i32,
    }
}

// Usage - IDE autocomplete works!
let person = Person::new(GcObject::new(val, store)?);
let name: String = person.name()?;  // Generated method!
let age: i32 = person.age()?;       // Type-safe!

// Or create directly from Val and Store
let person = Person::from_val(val, store)?;
let name: String = person.name()?;
```

**What gets generated:**
```rust
pub struct Person {
    inner: GcObject<RefCell<Store<()>>>,
}

impl Person {
    pub fn new(obj: GcObject<RefCell<Store<()>>>) -> Self { ... }
    pub fn from_val(val: Val, store: Store<()>) -> Result<Self> { ... }
    pub fn name(&self) -> Result<String> { ... }
    pub fn age(&self) -> Result<i32> { ... }
}
```

**Benefits:**
- ✨ **IDE Autocomplete**: All methods show up in your IDE
- ✨ **Type Safety**: Field types checked at compile time
- ✨ **Clean Syntax**: Natural Rust method calls `person.name()?`
- ✨ **No String Literals**: Can't typo field names
- ✨ **Zero Boilerplate**: One macro invocation generates everything
- ✨ **Store Hidden**: No `&mut store` parameters

**Multiple structs:**
```rust
gc_struct! {
    Person {
        name: 0 => String,
        age: 1 => i32,
    }
}

gc_struct! {
    Point {
        x: 0 => f64,
        y: 1 => f64,
    }
}

// Each gets full type-safe API
let p = Point::from_val(val, store)?;
let x: f64 = p.x()?;
let y: f64 = p.y()?;
```

### Mutation: Modify GC Objects from Rust!

You can modify mutable WebAssembly GC fields from Rust! Mark fields as `mut` in the `gc_struct!` macro to generate setter methods:

```rust
gc_struct! {
    Person {
        name: 0 => mut String,  // Mutable string - setter takes &str!
        age: 1 => mut i32,      // Mutable - getter + setter generated
    }
}

// Read fields
let name: String = person.name()?;  // "Bob 🎉"
let age: i32 = person.age()?;       // 28

// Mutate primitive fields
person.set_age(29)?;
let age: i32 = person.age()?;  // 29

// Mutate string fields - completely transparent!
person.set_name("Alice")?;     // Just pass &str - GC string created automatically!
let name: String = person.name()?;  // "Alice"

// Can also use the generic set_field method
person.inner.set_field("age", 30)?;      // By name
person.inner.set_field(1, 31)?;          // By index
person.inner.set_field("name", "Bob")?;  // Strings too!
```

**How it works:**
- Only fields marked with `mut` get setter methods generated
- Setters use the `ToVal` trait to convert Rust types to WebAssembly values
- **String fields are special**: setters take `&str` and automatically create WebAssembly GC string arrays!
- Supported types: `i32`, `i64`, `f32`, `f64`, `bool`, `String`
- String mutation requires the `Instance` to be passed when creating `GcObject`

**Requirements:**
- Field must be declared as `(mut ...)` in the WebAssembly struct type
- Type must implement the `ToVal` trait
- For example, the WAT file must have: `(field $age (mut i32))`

**Benefits:**
- ✨ **Type-safe**: Compile-time type checking
- ✨ **Ergonomic**: Clean `person.set_age(30)?` syntax
- ✨ **IDE Support**: Setter methods show up in autocomplete
- ✨ **Generic fallback**: `set_field()` works with both names and indices

### Nested Struct Navigation

There are multiple ways to access nested GC structs, each with different tradeoffs:

**1. Most Ergonomic - `get_as<T>()` method:**
```rust
// Get nested struct as typed wrapper - CLEANEST syntax!
let ellis: Person = bob.get_as("friend")?;
let age = ellis.age()?;
ellis.set_age(26)?;  // Can be mutated!
```

**2. One-liner - `get_nested()` method:**
```rust
// Access nested field directly without creating wrapper
let friend_name: String = bob.get_nested("friend", "name")?;
let friend_age: i32 = bob.get_nested("friend", 1)?;
```

**3. Low-level - `get_struct()` method:**
```rust
// Get raw StructRef for manual access
let nested_person = employee.get_struct(&mut store, 0)?;
let nested_age: i32 = nested_person.get(&mut store, 1)?;
```

**Recommendation**: Use `get_as()` when you need the nested object as a proper typed wrapper, `get_nested()` for quick one-off field access, and `get_struct()` only for low-level manipulation.

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

## Creating WebAssembly GC Strings from Rust

### Invisible String Creation (Automatic)

When you use mutation APIs with string fields, GC string arrays are created automatically:

```rust
// Create GcObject with instance for string support
let person = GcObject::new(person_val, store, Some(instance))?;

// String automatically created as WebAssembly GC array!
person.set_field("name", "Alice")?;

// Or with generated setters
bob.set_name("Alice")?;  // &str → GC string array (invisible!)
```

**How it works under the hood:**
1. The `ToVal` trait is implemented for `&str`
2. When setting a string field, `GcString::create()` is called automatically
3. It writes the string to linear memory
4. Calls the WASM `new_string` function to create a GC array
5. Returns the GC array reference as a `Val`

**You never need to call `GcString::create()` manually!** It's handled transparently by the API.

### Manual String Creation (Advanced)

If you need direct control, you can create GC strings manually:

```rust
let val = GcString::create(&mut store, &instance, "Hello")?;
// val is now a Val containing a WebAssembly GC string array
```

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

## Direct Struct Creation from Rust (No WASM Helpers!)

You can create WebAssembly GC structs **entirely from Rust** without calling WASM helper functions!

### The Hybrid Approach (Recommended)

**Step 1: Bootstrap** - Create one instance using WASM functions to extract type information:
```rust
// Create first person using WASM helper (for type discovery)
let bob_val = create_person_wasm(...)?;
let bob_struct = bob_val.unwrap_anyref()?.unwrap_struct(&store)?;
```

**Step 2: Extract Types** - Create a `StructBuilder` from the existing instance:
```rust
use gc_traits::StructBuilder;

// Builder shares the store and handles all borrowing internally!
let builder = StructBuilder::from_existing_shared(
    bob.inner.clone_store(),
    bob.inner.as_struct_ref()
)?;
```

**Step 3: Direct Creation** - Create new instances directly from Rust (NO with_store needed!):
```rust
// Create Charlie WITHOUT calling WASM - look how clean this is!
let name = builder.create_string("Charlie")?;  // No &mut store!
let charlie_struct = builder.create(&[
    name,
    Val::I32(35),
    Val::null_any_ref(),  // no email
    Val::null_any_ref(),  // no friend
])?;

// Wrap in typed wrapper
let charlie = Person::new(GcObject::from_struct_shared(
    charlie_struct,
    bob.inner.clone_store(),
    bob.inner.clone_instance(),
));

// Use it!
println!("{} is {}", charlie.name()?, charlie.age()?);
```

### What You Get

**Before (calling WASM helpers):**
```rust
// Write to linear memory
memory.write(&mut store, 100, "Ellis".as_bytes())?;

// Call WASM function
create_person.call(&mut store, &[Val::I32(100), Val::I32(5), Val::I32(25)], &mut results)?;

// Extract result
let ellis = results[0].clone();
```

**After (direct creation):**
```rust
// Create directly from Rust - no store passing!
let name = builder.create_string("Ellis")?;
let ellis = builder.create(&[name, Val::I32(25), Val::null_any_ref(), Val::null_any_ref()])?;
```

### How It Works

1. **Type Extraction**: `StructBuilder::from_existing_shared()` gets struct and array types from an existing instance using `struct_ref.ty()` and `array_ref.ty()`

2. **Shared Store**: Builder owns `Rc<RefCell<Store<()>>>` and handles all borrowing internally - no more `with_store()` closures!

3. **Pre-allocators**: Creates reusable `StructRefPre` and `ArrayRefPre` for efficient repeated allocation

4. **Direct Creation**: Uses Wasmtime's `StructRef::new()` and `ArrayRef::new_fixed()` to create GC objects directly on the Rust side

5. **String Creation**: `create_string()` converts Rust `&str` to WebAssembly GC array of i8 bytes - borrows store internally

### Benefits

- ✨ **No WASM dependency**: Don't need to export helper functions from WASM
- ✨ **Performance**: Avoids linear memory writes and function calls
- ✨ **Type safety**: Extracted types match the actual WASM definitions
- ✨ **Reusable**: One builder can create many instances efficiently
- ✨ **Clean API**: No `with_store()` closures - builder handles borrowing internally
- ✨ **Hybrid flexibility**: Use WASM for first instance, direct creation after

### When to Use Each Approach

| Approach | Use When |
|----------|----------|
| **WASM helpers** | Initial prototyping, simple cases, or when types are complex |
| **Direct creation** | Performance-critical paths, bulk creation, or when you want full Rust control |
| **Hybrid (recommended)** | Production code - simple bootstrap, then efficient direct creation |

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
