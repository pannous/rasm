use anyhow::Result;
use wasmtime::*;

pub fn run_advanced_demo() -> Result<()> {
    println!("\n=== Advanced WebAssembly Demo ===");
    println!("Testing memory, structs, and strings\n");

    let engine = Engine::default();
    let mut store = Store::new(&engine, ());

    let wat_source = std::fs::read_to_string("advanced.wat")?;
    let wasm_bytes = wat::parse_str(&wat_source)?;
    let module = Module::new(&engine, wasm_bytes)?;
    let instance = Instance::new(&mut store, &module, &[])?;

    let memory = instance.get_memory(&mut store, "memory")
        .expect("Failed to get memory");

    let create_person = instance.get_typed_func::<i32, i32>(&mut store, "create_person")?;
    let get_age = instance.get_typed_func::<i32, i32>(&mut store, "get_age")?;
    let set_name = instance.get_typed_func::<(i32, i32, i32), ()>(&mut store, "set_name")?;
    let get_name_len = instance.get_typed_func::<i32, i32>(&mut store, "get_name_len")?;
    let get_name_ptr = instance.get_typed_func::<i32, i32>(&mut store, "get_name_ptr")?;
    let sum_ages = instance.get_typed_func::<(i32, i32), i32>(&mut store, "sum_ages")?;
    let birthday = instance.get_typed_func::<i32, ()>(&mut store, "birthday")?;
    let memory_copy = instance.get_typed_func::<(i32, i32, i32), ()>(&mut store, "memory_copy")?;

    println!("1. Creating Person structs in WASM memory");
    println!("------------------------------------------");

    let alice_offset = create_person.call(&mut store, 25)?;
    println!("Created Alice at offset: {}", alice_offset);

    let bob_offset = create_person.call(&mut store, 30)?;
    println!("Created Bob at offset: {}", bob_offset);

    println!("\n2. Reading struct fields");
    println!("------------------------");
    let alice_age = get_age.call(&mut store, alice_offset)?;
    let bob_age = get_age.call(&mut store, bob_offset)?;
    println!("Alice's age: {}", alice_age);
    println!("Bob's age: {}", bob_age);

    println!("\n3. Setting names (as repeating characters)");
    println!("-------------------------------------------");
    set_name.call(&mut store, (alice_offset, 'A' as i32, 5))?;
    set_name.call(&mut store, (bob_offset, 'B' as i32, 3))?;

    let alice_name_len = get_name_len.call(&mut store, alice_offset)?;
    let alice_name_ptr = get_name_ptr.call(&mut store, alice_offset)?;

    let bob_name_len = get_name_len.call(&mut store, bob_offset)?;
    let bob_name_ptr = get_name_ptr.call(&mut store, bob_offset)?;

    println!("Alice's name length: {}", alice_name_len);
    println!("Alice's name pointer: {}", alice_name_ptr);
    println!("Bob's name length: {}", bob_name_len);
    println!("Bob's name pointer: {}", bob_name_ptr);

    println!("\n4. Reading strings from WASM memory");
    println!("------------------------------------");

    let alice_name_bytes = read_memory(&memory, &mut store, alice_name_ptr as usize, alice_name_len as usize);
    let alice_name = String::from_utf8_lossy(&alice_name_bytes);
    println!("Alice's name from memory: '{}'", alice_name);

    let bob_name_bytes = read_memory(&memory, &mut store, bob_name_ptr as usize, bob_name_len as usize);
    let bob_name = String::from_utf8_lossy(&bob_name_bytes);
    println!("Bob's name from memory: '{}'", bob_name);

    println!("\n5. Writing strings from Rust to WASM memory");
    println!("--------------------------------------------");

    let carol_name = "Carol";
    let carol_offset = 500;
    write_memory(&memory, &mut store, carol_offset, carol_name.as_bytes());

    let read_back = read_memory(&memory, &mut store, carol_offset, carol_name.len());
    println!("Wrote '{}' to offset {}", String::from_utf8_lossy(&read_back), carol_offset);

    println!("\n6. Struct operations");
    println!("--------------------");

    let total_age = sum_ages.call(&mut store, (alice_offset, bob_offset))?;
    println!("Sum of Alice and Bob's ages: {}", total_age);

    println!("Celebrating Alice's birthday...");
    birthday.call(&mut store, alice_offset)?;
    let new_age = get_age.call(&mut store, alice_offset)?;
    println!("Alice's new age: {}", new_age);

    println!("\n7. Memory copy operation");
    println!("------------------------");

    let source_text = "Hello WASM!";
    let src_offset = 600;
    let dest_offset = 700;

    write_memory(&memory, &mut store, src_offset, source_text.as_bytes());
    memory_copy.call(&mut store, (dest_offset as i32, src_offset as i32, source_text.len() as i32))?;

    let copied = read_memory(&memory, &mut store, dest_offset, source_text.len());
    println!("Original at {}: '{}'", src_offset, source_text);
    println!("Copied to {}: '{}'", dest_offset, String::from_utf8_lossy(&copied));

    println!("\n8. Memory inspection");
    println!("--------------------");

    println!("Person struct layout (Alice at offset {}):", alice_offset);
    let struct_bytes = read_memory(&memory, &mut store, alice_offset as usize, 12);
    println!("  Age (4 bytes): {:?}", &struct_bytes[0..4]);
    println!("  Name ptr (4 bytes): {:?}", &struct_bytes[4..8]);
    println!("  Name len (4 bytes): {:?}", &struct_bytes[8..12]);

    let age_value = i32::from_le_bytes([struct_bytes[0], struct_bytes[1], struct_bytes[2], struct_bytes[3]]);
    let ptr_value = i32::from_le_bytes([struct_bytes[4], struct_bytes[5], struct_bytes[6], struct_bytes[7]]);
    let len_value = i32::from_le_bytes([struct_bytes[8], struct_bytes[9], struct_bytes[10], struct_bytes[11]]);

    println!("  Decoded age: {}", age_value);
    println!("  Decoded name ptr: {}", ptr_value);
    println!("  Decoded name len: {}", len_value);

    println!("\n✓ All advanced features tested successfully!");

    Ok(())
}

fn read_memory(memory: &Memory, store: &mut Store<()>, offset: usize, len: usize) -> Vec<u8> {
    let data = memory.data(store);
    data[offset..offset + len].to_vec()
}

fn write_memory(memory: &Memory, store: &mut Store<()>, offset: usize, data: &[u8]) {
    let mem_data = memory.data_mut(store);
    mem_data[offset..offset + data.len()].copy_from_slice(data);
}
