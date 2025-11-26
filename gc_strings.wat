(module

  ;; Define array type first
  (type $string (array (mut i8)))

  ;; Linear memory for string data TODO why? we have type $string ^^
  (memory (export "memory") 1)

  (func $reverse_string (export "reverse_string") (param $s (ref $string)) (result (ref $string))
    (local $len i32)
    (local $i i32)
    (local $new_str (ref $string))
    (local.set $len (array.len (local.get $s)))
    (local.set $new_str (array.new_default $string (local.get $len)))
    (local.set $i (i32.const 0))
    (block $break
      (loop $continue
        (br_if $break (i32.ge_u (local.get $i) (local.get $len)))
        (array.set $string
          (local.get $new_str)
          (i32.sub (i32.sub (local.get $len) (i32.const 1)) (local.get $i))
          (array.get_u $string (local.get $s) (local.get $i))
        )
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $continue)
      )
    )
    (local.get $new_str)
  )
)
