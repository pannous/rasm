(module
  ;; Define array type first
  (type $string (array (mut i8)))

  ;; Define struct types
  (type $person (struct
    (field $name (mut ref $string))
    (field $age (mut i32))
    (field $email (ref null $string))
    (field $friend (mut (ref null $person)))
  ))
  (type $person_list (array (mut (ref null $person))))

  ;; Array type for list of people

  ;; Linear memory for string data
  (memory (export "memory") 1)

  ;; Create a new string from memory
  (func $new_string (param $ptr i32) (param $len i32) (result (ref $string))
    (local $i i32)
    (local $arr (ref $string))

    ;; Create array with the specified length
    (local.set $arr
      (array.new_default $string (local.get $len))
    )

    ;; Copy bytes from linear memory to GC array
    (local.set $i (i32.const 0))
    (block $break
      (loop $continue
        ;; Check if we've copied all bytes
        (br_if $break (i32.ge_u (local.get $i) (local.get $len)))

        ;; Copy one byte: arr[i] = memory[ptr + i]
        (array.set $string
          (local.get $arr)
          (local.get $i)
          (i32.load8_u (i32.add (local.get $ptr) (local.get $i)))
        )

        ;; i++
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $continue)
      )
    )

    (local.get $arr)
  )

  ;; Get string length
  (func $string_len (export "string_len") (param $s (ref $string)) (result i32)
    (array.len (local.get $s))
  )

  ;; Get byte at index
  (func $string_get (export "string_get") (param $s (ref $string)) (param $i i32) (result i32)
    (array.get_u $string (local.get $s) (local.get $i))
  )

  ;; Copy string to linear memory
  (func $string_to_mem (export "string_to_mem")
    (param $s (ref $string)) (param $ptr i32) (result i32)
    (local $i i32)
    (local $len i32)

    (local.set $len (array.len (local.get $s)))
    (local.set $i (i32.const 0))

    (block $break
      (loop $continue
        (br_if $break (i32.ge_u (local.get $i) (local.get $len)))

        ;; memory[ptr + i] = arr[i]
        (i32.store8
          (i32.add (local.get $ptr) (local.get $i))
          (array.get_u $string (local.get $s) (local.get $i))
        )

        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $continue)
      )
    )

    ;; Return length
    (local.get $len)
  )

  ;; Create a person
  (func $create_person (export "create_person")
    (param $name_ptr i32) (param $name_len i32) (param $age i32)
    (result (ref $person))

    (struct.new $person
      (call $new_string (local.get $name_ptr) (local.get $name_len))
      (local.get $age)
      (ref.null $string)
      (ref.null $person)  ;; friend starts as null
    )
  )

  ;; Create person with email
  (func $create_person_with_email (export "create_person_with_email")
    (param $name_ptr i32) (param $name_len i32)
    (param $age i32)
    (param $email_ptr i32) (param $email_len i32)
    (result (ref $person))

    (struct.new $person
      (call $new_string (local.get $name_ptr) (local.get $name_len))
      (local.get $age)
      (call $new_string (local.get $email_ptr) (local.get $email_len))
      (ref.null $person)  ;; friend starts as null
    )
  )

  ;; Get person age
  (func $get_age (export "get_age") (param $p (ref $person)) (result i32)
    (struct.get $person $age (local.get $p))
  )

  ;; Set person age
  (func $set_age (export "set_age") (param $p (ref $person)) (param $new_age i32)
    (struct.set $person $age (local.get $p) (local.get $new_age))
  )

  ;; Check if person has email (option type)
  (func $has_email (export "has_email") (param $p (ref $person)) (result i32)
    (if (result i32)
      (ref.is_null (struct.get $person $email (local.get $p)))
      (then (i32.const 0))
      (else (i32.const 1))
    )
  )

  ;; Create a point
  (func $create_point (export "create_point") (param $x f64) (param $y f64) (result (ref $point))
    (struct.new $point
      (local.get $x)
      (local.get $y)
    )
  )

  ;; Get point x coordinate
  (func $point_x (export "point_x") (param $p (ref $point)) (result f64)
    (struct.get $point $x (local.get $p))
  )

  ;; Get point y coordinate
  (func $point_y (export "point_y") (param $p (ref $point)) (result f64)
    (struct.get $point $y (local.get $p))
  )

  ;; Move point
  (func $point_move (export "point_move") (param $p (ref $point)) (param $dx f64) (param $dy f64)
    (struct.set $point $x
      (local.get $p)
      (f64.add (struct.get $point $x (local.get $p)) (local.get $dx))
    )
    (struct.set $point $y
      (local.get $p)
      (f64.add (struct.get $point $y (local.get $p)) (local.get $dy))
    )
  )

  ;; Distance between two points
  (func $point_distance (export "point_distance")
    (param $p1 (ref $point)) (param $p2 (ref $point))
    (result f64)
    (local $dx f64)
    (local $dy f64)

    (local.set $dx
      (f64.sub
        (struct.get $point $x (local.get $p2))
        (struct.get $point $x (local.get $p1))
      )
    )
    (local.set $dy
      (f64.sub
        (struct.get $point $y (local.get $p2))
        (struct.get $point $y (local.get $p1))
      )
    )

    ;; sqrt(dx^2 + dy^2)
    (f64.sqrt
      (f64.add
        (f64.mul (local.get $dx) (local.get $dx))
        (f64.mul (local.get $dy) (local.get $dy))
      )
    )
  )

  ;; Create employee
  (func $create_employee (export "create_employee")
    (param $person (ref $person)) (param $id i32) (param $salary f64)
    (result (ref $employee))

    (struct.new $employee
      (local.get $person)
      (local.get $id)
      (local.get $salary)
    )
  )

  ;; Get employee ID
  (func $employee_id (export "employee_id") (param $emp (ref $employee)) (result i32)
    (struct.get $employee $id (local.get $emp))
  )

  ;; Get employee salary
  (func $employee_salary (export "employee_salary") (param $emp (ref $employee)) (result f64)
    (struct.get $employee $salary (local.get $emp))
  )

  ;; Give employee a raise
  (func $employee_raise (export "employee_raise")
    (param $emp (ref $employee)) (param $percent f64)
    (struct.set $employee $salary
      (local.get $emp)
      (f64.mul
        (struct.get $employee $salary (local.get $emp))
        (f64.add (f64.const 1.0) (local.get $percent))
      )
    )
  )

  ;; Get employee age (through nested person)
  (func $employee_age (export "employee_age") (param $emp (ref $employee)) (result i32)
    (struct.get $person $age
      (struct.get $employee $person (local.get $emp))
    )
  )

  ;; Create a list of people
  (func $create_person_list (export "create_person_list") (param $size i32) (result (ref $person_list))
    (array.new_default $person_list (local.get $size))
  )

  ;; Set person in list
  (func $list_set (export "list_set")
    (param $list (ref $person_list)) (param $index i32) (param $person (ref $person))
    (array.set $person_list (local.get $list) (local.get $index) (local.get $person))
  )

  ;; Get person from list
  (func $list_get (export "list_get")
    (param $list (ref $person_list)) (param $index i32)
    (result (ref null $person))
    (array.get $person_list (local.get $list) (local.get $index))
  )

  ;; Get list length
  (func $list_length (export "list_length") (param $list (ref $person_list)) (result i32)
    (array.len (local.get $list))
  )

  ;; Simple example returning anyref
  (func $new_object (export "new_object") (result anyref)
    (struct.new $point (f64.const 42.0) (f64.const 13.0))
  )
)
