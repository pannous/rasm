// use color_eyre::eyre::Result;
use wasmtime::*;

mod gc_traits;
// mod wasm_inspect_struct;
// mod old_wrapper;
// mod nest_test;

use gc_traits::GcObject;

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
        name: 0 => String,
        age: 1 => mut i32,  // Mark as mutable to generate setter
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
    let person = GcObject::new(fresh_person, store)?;

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

    let bob = Person::new(person);

    // Generated accessor methods - IDE autocomplete works!
    let name2: String = bob.name()?;
    let age2: i32 = bob.age()?;

    println!("With gc_struct! macro:");
    println!("  let name: String = person.name()?;  // {}", name2);
    println!("  let age: i32 = person.age()?;       // {}", age2);
    println!("\n✨ Type-safe, autocomplete-friendly, zero boilerplate!");

    // Demo: Mutation - modifying fields from Rust!
    println!("\n");
    println!("========================================");
    println!("MUTATION: Modify GC Objects from Rust");
    println!("========================================\n");

    println!("Original age: {}", bob.age()?);

    // Mutate using the generated setter method
    bob.set_age(29)?;
    println!("After bob.set_age(29): {}", bob.age()?);

    // Can also use the underlying GcObject's set_field method
    bob.set_age(30)?;
    println!("After bob.set_age(30): {}", bob.age()?);

    bob.set_age(31)?;
    println!("After bob.set_age(31): {}", bob.age()?);

    println!("\n✨ Full mutation support - modify GC objects from Rust!");

    Ok(())
}
