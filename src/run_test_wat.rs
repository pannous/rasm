#![allow(unused)]
use crate::wasm_helpers::load_wat;

use crate::{gc_struct, print};
use crate::gc_traits::{GcObject, GcStruct, InstanceExt, Struct};
use wasmtime::Val;

// see gc_object_demo.rs for more details and nice ORM access by name
pub fn run_test_wat() -> anyhow::Result<()> {
    println!("=== Test WAT with dummy imports ===\n");
    let (instance, mut store) = load_wat("test.wat")?;

    // Call the wasp_main function
    let result: i64 = instance.call(&mut store, "wasp_main", &[])?;
    println!("✓ wasp_main returned: {}", result);
    Ok(())
}

#[test]
fn test_run_test_wat() -> anyhow::Result<()> {
    print("OK");
    run_test_wat()
}