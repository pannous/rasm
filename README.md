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

```rust
// Just define your struct - everything else is generated!
gc_struct! {
    Person {
        name: 0 => String,
        age: 1 => i32,
    }
}


// Or create directly from Val and Store
let person = Person::from_val(val, store)?;
let name: String = person.name()?;  // Generated method!
let age: i32 = person.age()?;       // Type-safe!
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

### Mutate primitive fields
```
person.set_age(29)?;
person.set_name("Alice")?;     // Just pass &str - GC string created automatically!

// Can also use the generic set_field method
person.inner.set_field("age", 30)?;      // By name
person.inner.set_field(1, 31)?;          // By index
```


## **How it works:**
- Only fields marked with `mut` get setter methods generated
- Setters use the `ToVal` trait to convert Rust types to WebAssembly values
- **String fields are special**: setters take `&str` and automatically create WebAssembly GC string arrays!
- Supported types: `i32`, `i64`, `f32`, `f64`, `bool`, `String`
- String mutation requires the `Instance` to be passed when creating `GcObject`

### Nested Struct Navigation

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


### Type-Safe Wrappers
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
```rust
// Todo: do we really need the 'template' parameter person?
    let diana = Person::create(&person, obj! {
        name: "Diana 🚀",
        age: 29,
        email: "diana@example.com",
    })?;
    // diana is now a GcObject<Person> with fields set and can be passed as normal wasm ref
```

## Resources

- [WebAssembly GC Proposal](https://github.com/WebAssembly/gc)
- [WebAssembly Specification](https://webassembly.github.io/spec/)
- [Wasmtime GC Support](https://wasmtime.dev/)
- [WAT Format](https://developer.mozilla.org/en-US/docs/WebAssembly/Understanding_the_text_format)
