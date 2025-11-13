use wasmtime::Val;

fn nest_test(){

    // Create employee with nested person struct
    let create_employee = instance.get_func(&mut store, "create_employee")
        .expect("create_employee not found");
    let employee_id = instance.get_func(&mut store, "employee_id")
        .expect("employee_id not found");
    let employee_salary = instance.get_func(&mut store, "employee_salary")
        .expect("employee_salary not found");
    let employee_raise = instance.get_func(&mut store, "employee_raise")
        .expect("employee_raise not found");
    let employee_age = instance.get_func(&mut store, "employee_age")
        .expect("employee_age not found");

    create_employee.call(&mut store, &[person.clone(), Val::I32(12345), Val::F64(75000.0_f64.to_bits())], &mut results)?;
    let employee = results[0].clone();
    println!("\nCreated Employee");

    employee_id.call(&mut store, &[employee.clone()], &mut results)?;
    let id = results[0].unwrap_i32();
    println!("  ID: {}", id);

    employee_salary.call(&mut store, &[employee.clone()], &mut results)?;
    let salary = results[0].unwrap_f64();
    println!("  Salary: ${:.2}", salary);

    employee_age.call(&mut store, &[employee.clone()], &mut results)?;
    let emp_age = results[0].unwrap_i32();
    println!("  Age (from nested person): {}", emp_age);

    // Give a 10% raise
    employee_raise.call(&mut store, &[employee.clone(), Val::F64(0.10_f64.to_bits())], &mut [])?;
    employee_salary.call(&mut store, &[employee.clone()], &mut results)?;
    let new_salary = results[0].unwrap_f64();
    println!("  After 10% raise: ${:.2}", new_salary);

    println!("\n3. Working with GC Arrays");
    println!("-------------------------");

    let create_person_list = instance.get_func(&mut store, "create_person_list")
        .expect("create_person_list not found");
    let list_set = instance.get_func(&mut store, "list_set")
        .expect("list_set not found");
    let list_get = instance.get_func(&mut store, "list_get")
        .expect("list_get not found");
    let list_length = instance.get_func(&mut store, "list_length")
        .expect("list_length not found");

    // Create array of 3 people
    create_person_list.call(&mut store, &[Val::I32(3)], &mut results)?;
    let people_array = results[0].clone();
    println!("Created array for 3 people");

    // Add people to array
    create_person.call(&mut store, &[Val::I32(0), Val::I32(5), Val::I32(25)], &mut results)?;
    let alice = results[0].clone();
    list_set.call(&mut store, &[people_array.clone(), Val::I32(0), alice], &mut [])?;

    list_set.call(&mut store, &[people_array.clone(), Val::I32(1), bob], &mut [])?;

    list_length.call(&mut store, &[people_array.clone()], &mut results)?;
    let len = results[0].unwrap_i32();
    println!("  Array length: {}", len);

    // Iterate over array
    for i in 0..2 {
        list_get.call(&mut store, &[people_array.clone(), Val::I32(i)], &mut results)?;
        let person_ref = results[0].clone();

        if !person_ref.unwrap_anyref().is_none() {
            get_age.call(&mut store, &[person_ref], &mut results)?;
            let age = results[0].unwrap_i32();
            println!("  Person[{}] age: {}", i, age);
        }
    }

}