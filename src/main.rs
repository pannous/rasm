#![allow(unused)]
// main.rs - small runner for demos
mod gc_traits;
mod wasm_name_resolver;
mod gc_object_demo;
mod gc_string_demo;
mod wasm_helpers;
mod gc_struct_demo;
mod run_test_wat;

use anyhow::Result;
use gc_object_demo::gc_object_demo;
use gc_struct_demo::gc_struct_demo;
use crate::run_test_wat::run_test_wat;

fn main() -> Result<()> {
    // Propagate errors from the demo to get proper backtraces / exit codes.
    // gc_struct_demo()?;
    run_test_wat()?;
    Ok(())
}

fn print<T: std::fmt::Display>(x: T) {
    println!("{}", x)
}
