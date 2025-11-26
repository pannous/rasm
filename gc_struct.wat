(module
  (type $obj (struct (field $num i32)))

  (func $new_object (export "new_object") (result eqref)
    (struct.new $obj (i32.const 42)))

;; see gc_types.wat for more complex structs:
;;  (type $person (struct
;;    (field $name (mut (ref $string)))
;;    (field $age (mut i32))
;;    (field $friend (mut (ref null $person)))
;;  ))

)
