#![allow(unused)]
use crate::gc_traits::GcObject;
use crate::wasm_helpers::load_wat;
use crate::{gc_struct, obj, print};
use wasmtime::Val;

// Example type-safe wrappers with ergonomic field access
gc_struct! {
    Person {
        name: 0 => mut String,  // Now mutable - generates set_name(&str)!
        age: 1 => mut i32,      // Mutable - generates set_age(i32)
    }
}

#[allow(unused)]
pub fn gc_object_demo() -> anyhow::Result<()> {
    // get backtraces of errors
    // unsafe { set_var("RUST_BACKTRACE", "short"); }
    println!("=== WebAssembly GC Types Demo ===\n");

    let (instance, mut store) = load_wat("gc_types.wat")?;
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
    let create_person = instance
        .get_func(&mut store, "create_person")
        .expect("create_person not found");

    let params = [Val::I32(0), Val::I32(bob_name.len() as i32), Val::I32(28)]; // name offset, name len, age
    create_person.call(&mut store, &params, &mut results)?;
    let fresh_person = results[0].clone();

    // Wrap the person in GcObject - it owns the store
    let person = GcObject::new(fresh_person, store, Some(instance))?;

    println!("GcObject wraps the struct:");
    println!("----------------------------------------------------");

    println!("Access fields by name - store completely hidden!");
    let name: String = person.get("name")?;
    let age: i32 = person.get("age")?;
    let has_email = person.has("email")?;

    println!("  let name: String = person.get(\"name\")?;  // {}", name);
    println!("  let age: i32 = person.get(\"age\")?;      // {}", age);
    println!("  person.has(\"email\")?; // {}", has_email);

    println!("\n");
    println!("EVEN MORE ERGONOMIC: gc_struct! Macro");
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
    let diana = Person::create(
        &person,
        obj! {
            name: "Diana 🚀",
            age: 29,
            email: "diana@example.com",
        },
    )?;

    // Make Bob and Diana friends - CLEAN API!
    println!("\nMaking Bob and Diana friends...");
    person.set_field("friend", diana)?; // ✨ NO .inner needed!
    println!("✨ bob.set_field(\"friend\", diana) - Done!");

    // Read Bob's friend's data - 2 ways to do it!

    // Way 1: Direct nested access (one-liner)
    let friend_name: String = person.get_nested("friend", "name")?;
    let friend_age: i32 = person.get_nested("friend", "age")?;
    println!("Way 1 (get_nested): {}, age {}", friend_name, friend_age);

    // Way 2: Get as proper Person object - CLEANEST syntax!
    let diana: Person = person.get_as("friend")?;
    println!("Way 2 (get_as): {}, age {}", diana.name()?, diana.age()?);

    print(person);

    Ok(())
}
