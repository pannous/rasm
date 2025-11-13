(module
  ;; Memory with 1 page (64KB)
  (memory $mem 1)
  (export "memory" (memory $mem))

  ;; Global counter for struct instances
  (global $next_offset (mut i32) (i32.const 0))

  ;; Struct layout in memory (Person):
  ;; - offset+0: age (i32)
  ;; - offset+4: name_ptr (i32)
  ;; - offset+8: name_len (i32)

  ;; Write a string to memory at given offset
  ;; Returns the offset where string was written
  (func $write_string (param $offset i32) (param $char i32) (param $len i32) (result i32)
    (local $i i32)
    (local.set $i (i32.const 0))

    (block $done
      (loop $continue
        ;; Check if we've written all characters
        (br_if $done (i32.ge_u (local.get $i) (local.get $len)))

        ;; Write character to memory
        (i32.store8
          (i32.add (local.get $offset) (local.get $i))
          (local.get $char)
        )

        ;; Increment counter
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $continue)
      )
    )

    (local.get $offset)
  )

  ;; Create a "Person" struct in memory
  ;; Takes age and returns offset to struct
  (func $create_person (param $age i32) (result i32)
    (local $struct_offset i32)
    (local $name_offset i32)

    ;; Get current offset for struct
    (local.set $struct_offset (global.get $next_offset))

    ;; Store age at offset+0
    (i32.store (local.get $struct_offset) (local.get $age))

    ;; Name will be stored after struct (offset + 12)
    (local.set $name_offset (i32.add (local.get $struct_offset) (i32.const 12)))

    ;; Store name pointer at offset+4
    (i32.store
      (i32.add (local.get $struct_offset) (i32.const 4))
      (local.get $name_offset)
    )

    ;; Store name length at offset+8 (will be 0 initially)
    (i32.store
      (i32.add (local.get $struct_offset) (i32.const 8))
      (i32.const 0)
    )

    ;; Update next offset (struct + name buffer space)
    (global.set $next_offset
      (i32.add (local.get $struct_offset) (i32.const 112))
    )

    (local.get $struct_offset)
  )

  ;; Get age from person struct
  (func $get_age (param $person_offset i32) (result i32)
    (i32.load (local.get $person_offset))
  )

  ;; Set name for person (writes repeating character)
  (func $set_name (param $person_offset i32) (param $char i32) (param $len i32)
    (local $name_ptr i32)

    ;; Get name pointer from struct
    (local.set $name_ptr (i32.load (i32.add (local.get $person_offset) (i32.const 4))))

    ;; Write string to that location
    (drop (call $write_string (local.get $name_ptr) (local.get $char) (local.get $len)))

    ;; Update length in struct
    (i32.store
      (i32.add (local.get $person_offset) (i32.const 8))
      (local.get $len)
    )
  )

  ;; Get name length from person struct
  (func $get_name_len (param $person_offset i32) (result i32)
    (i32.load (i32.add (local.get $person_offset) (i32.const 8)))
  )

  ;; Get name pointer from person struct
  (func $get_name_ptr (param $person_offset i32) (result i32)
    (i32.load (i32.add (local.get $person_offset) (i32.const 4)))
  )

  ;; Sum ages of two people
  (func $sum_ages (param $person1 i32) (param $person2 i32) (result i32)
    (i32.add
      (call $get_age (local.get $person1))
      (call $get_age (local.get $person2))
    )
  )

  ;; Increment age
  (func $birthday (param $person_offset i32)
    (i32.store
      (local.get $person_offset)
      (i32.add (call $get_age (local.get $person_offset)) (i32.const 1))
    )
  )

  ;; Copy memory region
  (func $memory_copy (param $dest i32) (param $src i32) (param $len i32)
    (local $i i32)
    (local.set $i (i32.const 0))

    (block $done
      (loop $continue
        (br_if $done (i32.ge_u (local.get $i) (local.get $len)))

        (i32.store8
          (i32.add (local.get $dest) (local.get $i))
          (i32.load8_u (i32.add (local.get $src) (local.get $i)))
        )

        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $continue)
      )
    )
  )

  ;; Export functions
  (export "create_person" (func $create_person))
  (export "get_age" (func $get_age))
  (export "set_name" (func $set_name))
  (export "get_name_len" (func $get_name_len))
  (export "get_name_ptr" (func $get_name_ptr))
  (export "sum_ages" (func $sum_ages))
  (export "birthday" (func $birthday))
  (export "memory_copy" (func $memory_copy))
  (export "write_string" (func $write_string))
)
