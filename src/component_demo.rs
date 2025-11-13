use anyhow::Result;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};

wasmtime::component::bindgen!({
    path: "component.wit",
    world: "component-demo",
});

pub fn run() -> Result<()> {
    println!("=== Component Model Demo (WIT) ===\n");

    let mut config = Config::new();
    config.wasm_component_model(true);

    let engine = Engine::new(&config)?;
    let mut linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());

    let component_path = "component-guest/target/wasm32-wasip1/release/component_guest.wasm";

    if !std::path::Path::new(component_path).exists() {
        println!("Component not built yet. Building component...\n");
        let status = std::process::Command::new("cargo")
            .args(["build", "--release", "--target", "wasm32-wasip1"])
            .current_dir("component-guest")
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to build component");
        }
    }

    let component = Component::from_file(&engine, component_path)?;
    let (bindings, _) = ComponentDemo::instantiate(&mut store, &component, &linker)?;

    println!("1. Testing greet function");
    println!("-------------------------");
    let greeting = bindings.call_greet(&mut store, "World")?;
    println!("Result: {}\n", greeting);

    println!("2. Creating Person with records");
    println!("--------------------------------");
    let alice = bindings.call_create_person(&mut store, "Alice", 30)?;
    println!("Created: name='{}', age={}, email={:?}\n", alice.name, alice.age, alice.email);

    let charlie = bindings.call_create_person(&mut store, "Charlie", 15)?;
    println!("Created: name='{}', age={}, email={:?}\n", charlie.name, charlie.age, charlie.email);

    println!("3. Formatting person");
    println!("--------------------");
    let formatted = bindings.call_format_person(&mut store, &alice)?;
    println!("{}\n", formatted);

    println!("4. Testing option types");
    println!("-----------------------");
    let is_alice_adult = bindings.call_is_adult(&mut store, &alice)?;
    let is_charlie_adult = bindings.call_is_adult(&mut store, &charlie)?;
    println!("Is Alice adult? {}", is_alice_adult);
    println!("Is Charlie adult? {}\n", is_charlie_adult);

    println!("5. Getting list of people");
    println!("-------------------------");
    let people = bindings.call_get_people(&mut store)?;
    for person in &people {
        let formatted = bindings.call_format_person(&mut store, person)?;
        println!("  - {}", formatted);
    }
    println!();

    println!("6. Finding person (option return)");
    println!("----------------------------------");
    match bindings.call_find_person(&mut store, "Bob")? {
        Some(person) => {
            let formatted = bindings.call_format_person(&mut store, &person)?;
            println!("Found: {}", formatted);
        }
        None => println!("Not found"),
    }

    match bindings.call_find_person(&mut store, "David")? {
        Some(person) => {
            let formatted = bindings.call_format_person(&mut store, &person)?;
            println!("Found: {}", formatted);
        }
        None => println!("David not found (as expected)"),
    }
    println!();

    println!("7. Testing nested records and variants");
    println!("---------------------------------------");

    let employee = Employee {
        person: Person {
            name: "John Doe".to_string(),
            age: 35,
            email: Some("john@example.com".to_string()),
        },
        employee_id: 12345,
        department: "Engineering".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "San Francisco".to_string(),
            country: "USA".to_string(),
            postal_code: "94102".to_string(),
        },
    };

    let result = bindings.call_validate_employee(&mut store, &employee)?;
    match result {
        ResultCode::Success => println!("✓ Valid employee"),
        ResultCode::NotFound => println!("✗ Not found"),
        ResultCode::InvalidInput(msg) => println!("✗ Invalid: {}", msg),
    }

    let invalid_employee = Employee {
        person: Person {
            name: "".to_string(),
            age: 14,
            email: None,
        },
        employee_id: 99999,
        department: "".to_string(),
        address: Address {
            street: "".to_string(),
            city: "".to_string(),
            country: "USA".to_string(),
            postal_code: "".to_string(),
        },
    };

    let result = bindings.call_validate_employee(&mut store, &invalid_employee)?;
    match result {
        ResultCode::Success => println!("✓ Valid employee"),
        ResultCode::NotFound => println!("✗ Not found"),
        ResultCode::InvalidInput(msg) => println!("✗ Invalid: {}", msg),
    }

    println!("\n✓ All Component Model features tested successfully!");
    println!("\nComponent Model Features Demonstrated:");
    println!("  - Records (struct-like types)");
    println!("  - Strings (native UTF-8)");
    println!("  - Options (nullable types)");
    println!("  - Lists (dynamic arrays)");
    println!("  - Variants (enum/union types)");
    println!("  - Nested records");

    Ok(())
}
