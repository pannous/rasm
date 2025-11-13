use wasmtime::{Store, Val};

// BLACK MAGIC: Generic function to inspect any GC struct
fn inspect_struct(store: &mut Store<()>, val: &Val, name: &str) -> anyhow::Result<()> {
    println!("\nInspecting '{}':", name);

    let anyref = val.unwrap_anyref().expect("not an anyref");
    let struct_ref = anyref.unwrap_struct(&*store)?;

    // Try to read fields (we don't know the count, so try until we get an error)
    let mut field_idx = 0;
    loop {
        match struct_ref.field(&mut *store, field_idx) {
            Ok(field_val) => {
                print!("  field[{}]: ", field_idx);
                match field_val {
                    Val::I32(v) => println!("i32 = {}", v),
                    Val::I64(v) => println!("i64 = {}", v),
                    Val::F32(bits) => println!("f32 = {}", f32::from_bits(bits as u32)),
                    Val::F64(bits) => println!("f64 = {}", f64::from_bits(bits)),
                    Val::AnyRef(ref r) => {
                        if r.is_none() {
                            println!("ref = null");
                        } else {
                            println!("ref = <object>");
                        }
                    }
                    _ => println!("<other>"),
                }
                field_idx += 1;
            }
            Err(_) => break, // No more fields
        }
    }

    Ok(())
}

fn test_inspect_struct(store: &mut Store<()>, val: &Val, name: &str) {

    println!("\nReading Person fields WITHOUT helper functions:");
    println!("------------------------------------------------");

    let person_anyref = person.unwrap_anyref().expect("not an anyref");
    let person_struct = person_anyref.unwrap_struct(&store)?;

    // Field 0: name ref, Field 1: age, Field 2: email ref
    let age_val = person_struct.field(&mut store, 1)?;
    let age = age_val.unwrap_i32();

    println!("  Direct field[1] (age): {}", age);

    // Check if email is null (option type)
    let email_val = person_struct.field(&mut store, 2)?;
    let has_email = !email_val.unwrap_anyref().is_none();
    println!("  Direct field[2] (email): {}", if has_email { "Some(...)" } else { "None" });

    println!("\nReading Employee nested struct WITHOUT helpers:");
    println!("------------------------------------------------");

    let emp_anyref = employee.unwrap_anyref().expect("not an anyref");
    let emp_struct = emp_anyref.unwrap_struct(&store)?;

    // Field 0: person ref, Field 1: id, Field 2: salary
    let id_val = emp_struct.field(&mut store, 1)?;
    let id = id_val.unwrap_i32();
    println!("  Direct field[1] (id): {}", id);

    let salary_val = emp_struct.field(&mut store, 2)?;
    let salary = salary_val.unwrap_f64();
    println!("  Direct field[2] (salary): ${:.2}", salary);

    // Access nested person through field 0
    let nested_person_val = emp_struct.field(&mut store, 0)?;
    let nested_person_anyref = nested_person_val.unwrap_anyref().expect("not an anyref");
    let nested_person_struct = nested_person_anyref.unwrap_struct(&store)?;

    let nested_age_val = nested_person_struct.field(&mut store, 1)?;
    let nested_age = nested_age_val.unwrap_i32();
    println!("  Direct field[0].field[1] (person.age): {}", nested_age);

    println!("\nGeneric struct inspector:");
    println!("-------------------------");
    inspect_struct(&mut store, &p1, "Point p1")?;
    inspect_struct(&mut store, &person, "Person")?;
    inspect_struct(&mut store, &employee, "Employee")?;

    println!("\n✨ BLACK MAGIC SUCCESS!");
    println!("No WASM helper functions needed - direct GC heap access from Rust!");

}