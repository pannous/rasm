(module
  ;; Simple function that adds two numbers
  (func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )

  ;; Function that multiplies two numbers
  (func $multiply (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.mul
  )

  ;; Function that calculates factorial (recursive)
  (func $factorial (param $n i32) (result i32)
    (if (result i32)
      (i32.le_s (local.get $n) (i32.const 1))
      (then (i32.const 1))
      (else
        (i32.mul
          (local.get $n)
          (call $factorial
            (i32.sub (local.get $n) (i32.const 1))
          )
        )
      )
    )
  )

  ;; Export functions to make them callable from host
  (export "add" (func $add))
  (export "multiply" (func $multiply))
  (export "factorial" (func $factorial))
)
