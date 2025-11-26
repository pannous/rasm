use wasmtime::{Config, Engine, Extern, Func, Instance, Linker, Module, Store, Val, ValType};
use crate::gc_traits;
use anyhow::Result;

pub fn load_wat(path: &str) -> Result<(Instance, Store<()>)> {
    load_wat_with_imports(path, &[])
}

pub fn load_wat_with_imports(path: &str, imports: &[(&str, &str, Extern)]) -> Result<(Instance, Store<()>)> {
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

    // Auto-detect and provide dummy imports if needed
    let mut linker = Linker::new(&engine);
    for import in module.imports() {
        let func_type = import.ty().unwrap_func().clone();
        let result_types: Vec<ValType> = func_type.results().collect();

        linker.func_new(
            import.module(),
            import.name(),
            func_type,
            move |_caller, _params, results| {
                // Return default values matching the expected result types
                for (result_ty, expected_ty) in results.iter_mut().zip(result_types.iter()) {
                    *result_ty = match expected_ty {
                        ValType::I32 => Val::I32(0),
                        ValType::I64 => Val::I64(0),
                        ValType::F32 => Val::F32(0u32),
                        ValType::F64 => Val::F64(0u64),
                        ValType::Ref(_) => Val::ExternRef(None),
                        _ => Val::I32(0),
                    };
                }
                Ok(())
            },
        )?;
    }

    let instance = linker.instantiate(&mut store, &module)?;
    return Ok((instance, store));
}