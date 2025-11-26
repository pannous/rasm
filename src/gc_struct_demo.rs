#![allow(unused)]
use crate::wasm_helpers::load_wat;

use crate::{gc_struct, print};
use crate::gc_traits::{GcObject, GcStruct, InstanceExt, Struct};
use wasmtime::Val;

// see gc_object_demo.rs for more details and nice ORM access by name
pub fn gc_struct_demo() -> anyhow::Result<()> {
    println!("=== MINIMAL WebAssembly GC Type Struct Demo ===\n");
    let (instance, mut store) = load_wat("gc_struct.wat")?;
    // Call helper that returns an owned GcObject (no type annotation needed)
    // let ok = instance.call_obj(&mut store, "new_object", &[])?;
    let ok : Struct = instance.call(&mut store, "new_object", &[])?;
    assert_eq!(ok.get_int(0)?, 42); // get field 0 set in file gc_struct.wat
    println!("✓ Successfully called new_object and got value: {}", ok.get::<i32, _>(0)?);
    println!("✓ eqref from WebAssembly successfully converted to GcObject/Obj/Val");
    let num: i32 = ok.get("num")?;
    assert_eq!(num, 42); // get field 0 set in file gc_struct.wat
    println!("✓ Named property available whenever they are defined in WAT");
    // ok.num() // this magic needs
    Ok(())
}

#[test]
fn test_gc_struct_demo() -> anyhow::Result<()> {
    print("OK");
    gc_struct_demo()
}