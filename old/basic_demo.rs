use anyhow::Result;
use wasmtime::*;

pub fn run() -> Result<()> {
    println!("=== Basic WebAssembly Runtime Demo ===\n");

    let engine = Engine::default();
    let mut store = Store::new(&engine, ());

    let wat_source = std::fs::read_to_string("example.wat")?;
    println!("Loaded WAT file successfully\n");

    let wasm_bytes = wat::parse_str(&wat_source)?;
    println!("Compiled WAT to WASM successfully\n");

    let module = Module::new(&engine, wasm_bytes)?;
    let instance = Instance::new(&mut store, &module, &[])?;

    let add = instance.get_typed_func::<(i32, i32), i32>(&mut store, "add")?;
    let multiply = instance.get_typed_func::<(i32, i32), i32>(&mut store, "multiply")?;
    let factorial = instance.get_typed_func::<i32, i32>(&mut store, "factorial")?;

    println!("Testing exported functions:");
    println!("----------------------------");

    let a = 15;
    let b = 27;
    let result = add.call(&mut store, (a, b))?;
    println!("add({}, {}) = {}", a, b, result);

    let result = multiply.call(&mut store, (a, b))?;
    println!("multiply({}, {}) = {}", a, b, result);

    for n in [5, 7, 10] {
        let result = factorial.call(&mut store, n)?;
        println!("factorial({}) = {}", n, result);
    }

    println!("\n✓ All WebAssembly functions executed successfully!\n");

    Ok(())
}
