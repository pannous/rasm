use wasmtime::{Config, Engine, Instance, Module, Store};
use crate::gc_traits;
use anyhow::Result;

pub fn load_wat(path: &str) -> Result<(Instance, Store<()>)> {

    let mut config = Config::new();
    config.wasm_gc(true);
    config.wasm_function_references(true);

    let engine = Engine::new(&config)?;
    let mut store = Store::new(&engine, ());

    let wat_source = std::fs::read_to_string(path)
        .or_else(|_| std::fs::read_to_string("rasm/".to_string() + path))?;
    println!("Loading WAT file with GC types...\n");

    let wasm_bytes = wat::parse_str(&wat_source)?;
    gc_traits::register_gc_types_from_wasm(&wasm_bytes)?;
    let module = Module::new(&engine, wasm_bytes)?;
    let instance = Instance::new(&mut store, &module, &[])?;
    return Ok((instance, store));
}