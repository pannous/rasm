(module $wasp_module
  (type (;0;) (func (param i32) (result externref)))
  (type (;1;) (func (param externref) (result i32)))
  (type (;2;) (func (param externref) (result i32)))
  (type (;3;) (func (param externref) (result i64)))
  (type (;4;) (func (param externref) (result f64)))
  (type (;5;) (func (result i64)))
  (import "env" "getElementById" (func $getElementById (type 0)))
  (import "env" "toNode" (func $toNode (type 1)))
  (import "env" "toString" (func $toString (type 2)))
  (import "env" "toLong" (func $toLong (type 3)))
  (import "env" "toReal" (func $toReal (type 4)))
  (func $wasp_main (type 5) (result i64)
    (local $x i32) (local $result i64)
    i32.const 65560
    call $getElementById
    nop
    nop
    nop
    nop
    call $toLong
    nop
    nop
    nop
    nop
    i32.wrap_i64
    nop
    local.tee $x
    i64.extend_i32_s
    nop
    return)
  (memory (;0;) 10240)
  (export "memory" (memory 0))
  (export "wasp_main" (func $wasp_main))
  (export "_start" (func $wasp_main))
  (data $wasp_data (i32.const 65536) "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00")
  (data $getElementById (i32.const 65552) "\18\00\01\00\03\00\00\00bla\00")
  (@custom "name" "\00\0c\0bwasp_module\01>\06\00\0egetElementById\01\06toNode\02\08toString\03\06toLong\04\06toReal\05\09wasp_main\02\0e\01\05\02\00\01x\01\06result\04\01\00\07\01\00\09\1c\02\00\09wasp_data\01\0egetElementById\0a\01\00")
  (@custom "producers" "\01\0cwasp 0.1.195")
  (@custom "sourceMappingURL" "\0dmain.wasm.map"))
