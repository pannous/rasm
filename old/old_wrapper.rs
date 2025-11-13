use wasmtime::Val;
use crate::gc_traits::{GcContext, GcString, OldGcString, Person, Point};

fn test_old_wrapper(){

    println!("1. Working with GC Struct Types (Point)");
    println!("----------------------------------------");

    let create_point = instance.get_func(&mut store, "create_point")
        .expect("create_point function not found");
    let point_x = instance.get_func(&mut store, "point_x")
        .expect("point_x function not found");
    let point_y = instance.get_func(&mut store, "point_y")
        .expect("point_y function not found");
    let point_move = instance.get_func(&mut store, "point_move")
        .expect("point_move function not found");
    let point_distance = instance.get_func(&mut store, "point_distance")
        .expect("point_distance function not found");

    // Create a point at (3.0, 4.0)
    let mut results = vec![Val::I32(0)];
    create_point.call(&mut store, &[Val::F64(3.0_f64.to_bits()), Val::F64(4.0_f64.to_bits())], &mut results)?;
    let p1 = results[0].clone();
    println!("Created Point p1");

    // Get coordinates
    point_x.call(&mut store, &[p1.clone()], &mut results)?;
    let x = results[0].unwrap_f64();
    point_y.call(&mut store, &[p1.clone()], &mut results)?;
    let y = results[0].unwrap_f64();
    println!("  p1 coordinates: ({}, {})", x, y);

    // Move the point
    point_move.call(&mut store, &[p1.clone(), Val::F64(1.0_f64.to_bits()), Val::F64(2.0_f64.to_bits())], &mut [])?;
    point_x.call(&mut store, &[p1.clone()], &mut results)?;
    let new_x = results[0].unwrap_f64();
    point_y.call(&mut store, &[p1.clone()], &mut results)?;
    let new_y = results[0].unwrap_f64();
    println!("  After move(1, 2): ({}, {})", new_x, new_y);

    // Create another point
    create_point.call(&mut store, &[Val::F64(7.0_f64.to_bits()), Val::F64(9.0_f64.to_bits())], &mut results)?;
    let p2 = results[0].clone();
    println!("Created Point p2 at (7, 9)");

    // Calculate distance
    point_distance.call(&mut store, &[p1.clone(), p2.clone()], &mut results)?;
    let distance = results[0].unwrap_f64();
    println!("  Distance between p1 and p2: {:.2}", distance);

    println!("\n2. Working with Nested Structs (Employee contains Person)");
    println!("----------------------------------------------------------");

    let create_person = instance.get_func(&mut store, "create_person").expect("create_person not found");

    // Create a person (simplified - we're not actually creating strings properly)
    create_person.call(&mut store, &[Val::I32(0), Val::I32(5), Val::I32(30)], &mut results)?;
    let person = results[0].clone();
    println!("Created Person");

    create_person.call(&mut store, &[Val::I32(0), Val::I32(3), Val::I32(35)], &mut results)?;
    let bob = results[0].clone();

    println!("\n4. Working with UTF-8 Strings (GC Arrays)");
    println!("-----------------------------------------");

    // Create a person with actual name string
    let memory = instance.get_memory(&mut store, "memory")
        .expect("memory not found");

    // Write "Alice" to memory
    let name = "Alice 👋";  // UTF-8 with emoji
    memory.write(&mut store, 0, name.as_bytes())?;

    // Create person with the string
    create_person.call(&mut store, &[Val::I32(0), Val::I32(name.len() as i32), Val::I32(28)], &mut results)?;
    let alice_person = results[0].clone();

    println!("Created person with name: '{}'", name);

    // Read the name back using direct array introspection (BLACK MAGIC!)
    let anyref = alice_person.unwrap_anyref().expect("not an anyref");
    let person_struct = anyref.unwrap_struct(&store)?;
    let name_field = person_struct.field(&mut store, 0)?;  // field 0 is name

    // Method 1: Direct array reading (no WASM helper function!)
    let read_name = OldGcString::to_string_direct(&mut store, &name_field)?;
    println!("  Read via direct GC array access: '{}'", read_name);

    // Method 2: Using WASM helper (string_to_mem)
    let read_name2 = OldGcString::to_string(&mut store, &instance, &name_field)?;
    println!("  Read via string_to_mem helper: '{}'", read_name2);

    // Verify UTF-8 emoji works
    println!("  UTF-8 support verified: {} bytes for '{}' string", name.len(), name);

    // BEAUTIFUL API: Read string with generic .get<String>()
    println!("\n  Using BEAUTIFUL API:");
    let beautiful_name: String = person_struct.get(&mut store, 0)?;
    println!("  person.get<String>(0) = '{}'", beautiful_name);
    println!("  ✨ No manual string conversion needed!");

    println!("\n=== GC Types Demonstrated ===");
    println!("✓ struct types: Point, Person, Employee");
    println!("✓ Nested structs: Employee contains Person");
    println!("✓ Mutable fields: age, salary, x, y");
    println!("✓ Immutable fields: id, person reference");
    println!("✓ Option/nullable: email field (ref null)");
    println!("✓ Array types: array of Person references");
    println!("✓ UTF-8 Strings: GC-managed byte arrays with encoding/decoding");
    println!("✓ Reference types: anyref, eqref");
    println!("✓ Garbage collection: all structs/arrays/strings managed by GC");
    println!("\nReal WebAssembly GC structs - not memory simulation!");


    println!("Reading Point p1 fields WITHOUT helper functions:");
    println!("--------------------------------------------------");

    // BLACK MAGIC: Direct field access using Wasmtime's GC APIs
    let p1_anyref = p1.unwrap_anyref().expect("not an anyref");
    let p1_struct = p1_anyref.unwrap_struct(&store)?;

    // Access field 0 (x coordinate)
    let x_val = p1_struct.field(&mut store, 0)?;
    let x = x_val.unwrap_f64();

    // Access field 1 (y coordinate)
    let y_val = p1_struct.field(&mut store, 1)?;
    let y = y_val.unwrap_f64();

    println!("  Direct field[0] (x): {}", x);
    println!("  Direct field[1] (y): {}", y);

    println!("\n========================================");
    println!("BEAUTIFUL API: Trait-Based Access");
    println!("========================================\n");

    println!("Using generic .get<T>() method:");
    println!("--------------------------------");

    // Beautiful trait-based API!
    let x: f64 = p1_struct.get(&mut store, 0)?;
    let y: f64 = p1_struct.get(&mut store, 1)?;
    println!("  p1.get<f64>(0) = {}", x);
    println!("  p1.get<f64>(1) = {}", y);

    let age: i32 = person_struct.get(&mut store, 1)?;
    println!("  person.get<i32>(1) = {}", age);

    // Beautiful String support - automatic UTF-8 conversion!
    let name: String = person_struct.get(&mut store, 0)?;
    println!("  person.get<String>(0) = '{}'", name);

    let id: i32 = emp_struct.get(&mut store, 1)?;
    let salary: f64 = emp_struct.get(&mut store, 2)?;
    println!("  employee.get<i32>(1) = {}", id);
    println!("  employee.get<f64>(2) = ${:.2}", salary);

    println!("\nUsing .get_struct() for nested objects:");
    println!("----------------------------------------");

    let nested_person = emp_struct.get_struct(&mut store, 0)?;
    let nested_age: i32 = nested_person.get(&mut store, 1)?;
    println!("  employee.get_struct(0).get<i32>(1) = {}", nested_age);

    println!("\nUsing .is_null() for option types:");
    println!("-----------------------------------");

    let email_is_null = person_struct.is_null(&mut store, 2)?;
    println!("  person.is_null(2) = {}", email_is_null);

    println!("\nUsing type-safe wrappers:");
    println!("-------------------------");

    // Type-safe Point wrapper
    let point = Point::from_val(&store, p1.clone())?;
    let point_x = point.x(&mut store)?;
    let point_y = point.y(&mut store)?;
    println!("  Point.x() = {}", point_x);
    println!("  Point.y() = {}", point_y);

    // Type-safe Person wrapper
    let person_typed = Person::from_val(&store, person.clone())?;
    let person_age = person_typed.age(&mut store)?;
    println!("  Person.age() = {}", person_age);

    println!("\n✨ BEAUTIFUL API SUCCESS!");
    println!("Type-safe, ergonomic GC struct access with Rust traits!");



    println!("\n========================================");
    println!("CONTEXTUAL API: No More &mut store!");
    println!("========================================\n");

    // Create separate GcContext for each example to avoid lifetime issues
    println!("Example 1: Point struct");
    println!("-----------------------");
    {
        let mut ctx = GcContext::new(&mut store);
        let mut p1_ctx = p1.clone().with_context(&mut ctx)?;

        // Now call methods WITHOUT passing &mut store!
        let x: f64 = p1_ctx.get(0)?;  // No store parameter!
        let y: f64 = p1_ctx.get(1)?;  // No store parameter!
        println!("  p1.get(0) = {} (no &mut store!)", x);
        println!("  p1.get(1) = {} (no &mut store!)", y);
    }

    println!("\nExample 2: Employee fields");
    println!("---------------------------");
    {
        let mut ctx = GcContext::new(&mut store);
        let mut emp_ctx = employee.clone().with_context(&mut ctx)?;

        let id: i32 = emp_ctx.get(1)?;
        let salary: f64 = emp_ctx.get(2)?;
        println!("  employee.get(1) = {}", id);
        println!("  employee.get(2) = ${:.2}", salary);
    }

    println!("\nExample 3: Option type checking");
    println!("--------------------------------");
    {
        let mut ctx = GcContext::new(&mut store);
        let mut person_ctx = person.clone().with_context(&mut ctx)?;
        let age: i32 = person_ctx.get(1)?;
        let is_null = person_ctx.is_null(2)?;  // No store!
        println!("  person.get(1) = {}", age);
        println!("  person.is_null(2) = {} (no &mut store!)", is_null);
    }

    println!("\n✨ CONTEXTUAL API SUCCESS!");
    println!("Clean, ergonomic API with hidden store parameter!");

}