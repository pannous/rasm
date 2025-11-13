// use color_eyre::eyre::Result;
use wasmtime::*;

mod gc_traits;
// mod wasm_inspect_struct;
// mod old_wrapper;
// mod nest_test;

use gc_traits::{GcObject, StructExt};

// Beautiful object-literal macro for creating GC structs!
// Works with ANY struct type!
use gc_traits::ObjFieldValue;

macro_rules! obj {
    ( $($k:ident : $v:expr),* $(,)? ) => {{
        vec![
            $(
                (stringify!($k), $crate::gc_traits::ObjFieldValue::from($v)),
            )*
        ]
    }};
}

use std::panic;
use backtrace::Backtrace;

#[allow(unused)]
fn fix_fucking_rust_backtrace(){
    panic::set_hook(Box::new(|_info| {
        println!("=== Rust Panic ===");
        let bt = Backtrace::new();
        // let bt = Backtrace::force_capture();// RUST_LIB_BACKTRACE=1
        // let bt = Backtrace::capture();
        for frame in bt.frames() {
            for sym in frame.symbols() {
                if let Some(path) = sym.filename() {
                    let s = path.to_string_lossy();
                    if s.contains("/rasm/") {        // adjust to your crate folder
                        if let Some(line) = sym.lineno() {
                            eprintln!("             at {}:{}", s, line);
                        } else {
                            eprintln!("             at {}", s);
                        }
                    }
                }
            }
        }
    }));
}

fn main() {
    println!("{}",std::env::current_dir().unwrap().display());
    // Install color-eyre for pretty error reports and backtraces
    color_eyre::install().expect("failed to install color-eyre");
    // Ensure error backtraces are captured and shown for eyre::Report
    // Users can also set RUST_BACKTRACE=1 in the environment instead of doing this in code.
    // std::env::set_var("RUST_BACKTRACE", "1");
    fix_fucking_rust_backtrace();
    // std::fs::read("xxx").unwrap(); // works
    // real_main().unwrap();          // <- force panic on Err, but no backtrace!
    // Print the eyre::Report using Debug formatting so any captured backtrace is shown.
    // Use {:#?} for pretty, multi-line output with causes and backtrace.
    real_main().unwrap_or_else(|e| {
        eprintln!("Error: {:#?}", e);
        std::process::exit(1);
    });
}



// Example type-safe wrappers with ergonomic field access
gc_struct! {
    Person {
        name: 0 => mut String,  // Now mutable - generates set_name(&str)!
        age: 1 => mut i32,      // Mutable - generates set_age(i32)
    }
}

impl Person {
    /// Create a new Person using beautiful object-literal syntax!
    ///
    /// # Example
    /// ```
    /// let diana = Person::create(&bob, obj! {
    ///     name: "Diana 🚀",
    ///     age: 29,
    ///     email: "diana@example.com",
    /// })?;
    /// ```
    pub fn create<T: gc_traits::GcStructWrapper>(
        from: &T,
        fields: Vec<(&str, ObjFieldValue)>,
    ) -> Result<Self> {
        use gc_traits::StructBuilder;

        // Create builder from existing instance
        let builder = StructBuilder::from_existing_shared(
            from.get_inner().clone_store(),
            from.get_inner().as_struct_ref(),
        )?;

        // Map field names to indices (Person-specific)
        let field_name_to_index = |name: &str| -> Result<usize> {
            match name {
                "name" => Ok(0),
                "age" => Ok(1),
                "email" => Ok(2),
                "friend" => Ok(3),
                _ => Err(anyhow::anyhow!("Unknown field: {}", name)),
            }
        };

        // Build Val array in correct order
        let mut vals = vec![Val::null_any_ref(); 4];  // 4 fields in Person
        for (field_name, field_value) in fields {
            let index = field_name_to_index(field_name)?;
            vals[index] = builder.field_value_to_val(&field_value)?;
        }

        // Create the struct
        let struct_ref = builder.create(&vals)?;

        // Wrap in typed wrapper
        Ok(Person::new(GcObject::from_struct_shared(
            struct_ref,
            from.get_inner().clone_store(),
            from.get_inner().clone_instance(),
        )))
    }
}


fn real_main() -> Result<()> { // get backtraces of errors
    // unsafe { set_var("RUST_BACKTRACE", "short"); }
    println!("=== WebAssembly GC Types Demo ===\n");
    // Enable GC proposal
    let mut config = Config::new();
    config.wasm_gc(true);
    config.wasm_function_references(true);

    let engine = Engine::new(&config)?;
    let mut store = Store::new(&engine, ());

    let wat_source = std::fs::read_to_string("gc_types.wat")
        .or_else(|_| std::fs::read_to_string("rasm/gc_types.wat"))?;
    println!("Loading WAT file with GC types...\n");

    let wasm_bytes = wat::parse_str(&wat_source)?;
    let module = Module::new(&engine, wasm_bytes)?;
    let instance = Instance::new(&mut store, &module, &[])?;
    let memory = instance.get_memory(&mut store, "memory").expect("Failed to get memory");

    println!("\n========================================");
    println!("BLACK MAGIC: Direct GC Object Access");
    println!("========================================\n");

    // ⚠️ Don't copy past this, see below for ergonomic API!

    // Create a fresh person for this demo
    // Write "Bob 🎉" to memory
    let bob_name = "Bob 🎉";
    memory.write(&mut store, 0, bob_name.as_bytes())?;

    // Create person with name at offset 0
    // let create_person = instance.get_typed_func::<i32, i32>(&mut store, "create_person")?;
    let mut results = vec![Val::I32(0)];
    let create_person = instance.get_func(&mut store, "create_person").expect("create_person not found");

    let params = [Val::I32(0), Val::I32(bob_name.len() as i32), Val::I32(28)]; // name offset, name len, age
    create_person.call(&mut store, &params, &mut results)?;
    let fresh_person = results[0].clone();

    // Wrap the person in GcObject - it owns the store
    let person = GcObject::new(fresh_person, store, Some(instance))?;

    println!("The most ergonomic API - GcObject wraps the struct:");
    println!("----------------------------------------------------");

    // Access fields by name - store completely hidden!
    let name: String = person.get("name")?;
    let age: i32 = person.get("age")?;
    let has_email = person.has("email")?;

    println!("  let name: String = person.get(\"name\")?;  // {}", name);
    println!("  let age: i32 = person.get(\"age\")?;      // {}", age);
    println!("  person.has(\"email\")?; // {}", has_email);

    println!("\n✨ ULTIMATE API SUCCESS!");
    println!("Store completely hidden - cleanest possible API!");

    // Demo: Even more ergonomic with gc_struct! macro!
    println!("\n");
    println!("========================================");
    println!("EVEN MORE ERGONOMIC: gc_struct! Macro");
    println!("========================================\n");

    let person = Person::new(person);

    // Generated accessor methods - IDE autocomplete works!
    let name2: String = person.name()?;
    let age2: i32 = person.age()?;

    println!("With gc_struct! macro:");
    println!("  let name: String = person.name()?;  // {}", name2);
    println!("  let age: i32 = person.age()?;       // {}", age2);
    println!("\n✨ Type-safe, autocomplete-friendly, zero boilerplate!");

    // Demo: Mutation - modifying fields from Rust!
    println!("\n");
    println!("========================================");
    println!("MUTATION: Modify GC Objects from Rust");
    println!("========================================\n");

    println!("Original age: {}", person.age()?);

    // Mutate using the generated setter method
    person.set_age(42)?;
    // bob["age"]=29
    println!("After bob.set_age(29): {}", person.age()?);

    // Test string mutation!
    println!("Before mutation: name = {}", person.name()?);
    person.set_name("Alice")?;
    println!("After bob.set_name(\"Alice\"): {}", person.name()?);

    println!("\n✨ Full mutation support - modify GC objects from Rust!");

    // Note: String mutation also works if the field is marked as (mut ...) in WAT!
    // Example (if name was mutable):
    //   bob.set_name("Alice")?;  // Would create GC string automatically!
    //   let name: String = bob.name()?;  // "Alice"
    println!("========================================");
    println!("NESTED STRUCTURES: Bob & Diana Friends!");
    println!("========================================\n");


    // Create another one with BEAUTIFUL object-literal syntax!
    // Todo: do we really need the 'template' parameter bob?
    let diana = Person::create(&person, obj! {
        name: "Diana 🚀",
        age: 29,
        email: "diana@example.com",
    })?;

    // Make Bob and Diana friends - CLEAN API!
    println!("\nMaking Bob and Diana friends...");
    person.set_field("friend", diana)?;  // ✨ NO .inner needed!
    println!("✨ bob.set_field(\"friend\", diana) - Done!");

    // Read Bob's friend's data - 2 ways to do it!

    // Way 1: Direct nested access (one-liner)
    let friend_name: String = person.get_nested("friend", "name")?;
    let friend_age: i32 = person.get_nested("friend", "age")?;
    println!("Way 1 (get_nested): {}, age {}", friend_name, friend_age);

    // Way 2: Get as proper Person object - CLEANEST syntax!
    let diana: Person = person.get_as("friend")?;
    println!("Way 2 (get_as): {}, age {}", diana.name()?, diana.age()?);


    Ok(())
}
