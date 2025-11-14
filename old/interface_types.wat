(module
  ;; Memory for storing strings and data structures
  (memory $mem (export "memory") 10)

  ;; Global offset tracker
  (global $offset (mut i32) (i32.const 0))

  ;;=== String Operations ===

  ;; Allocate space and return pointer
  (func $alloc (param $size i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (global.get $offset))
    (global.set $offset (i32.add (global.get $offset) (local.get $size)))
    (local.get $ptr)
  )
  (export "alloc" (func $alloc))

  ;; Write string length and data
  ;; Returns pointer to string object (length:i32 + data)
  (func $create_string (param $len i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (call $alloc (i32.add (local.get $len) (i32.const 4))))
    ;; Store length
    (i32.store (local.get $ptr) (local.get $len))
    ;; Return pointer
    (local.get $ptr)
  )
  (export "create_string" (func $create_string))

  ;; Get string length
  (func $string_length (param $ptr i32) (result i32)
    (i32.load (local.get $ptr))
  )
  (export "string_length" (func $string_length))

  ;; Get pointer to string data
  (func $string_data (param $ptr i32) (result i32)
    (i32.add (local.get $ptr) (i32.const 4))
  )
  (export "string_data" (func $string_data))

  ;;=== Record/Struct Operations ===

  ;; Person record layout:
  ;; offset+0:  name_ptr (i32)
  ;; offset+4:  name_len (i32)
  ;; offset+8:  age (i32)
  ;; offset+12: has_email (i32) - 0 or 1
  ;; offset+16: email_ptr (i32)
  ;; offset+20: email_len (i32)
  ;; Total: 24 bytes

  (func $person_new (param $age i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (call $alloc (i32.const 24)))

    ;; Initialize to empty/null
    (i32.store (local.get $ptr) (i32.const 0))           ;; name_ptr
    (i32.store offset=4 (local.get $ptr) (i32.const 0))  ;; name_len
    (i32.store offset=8 (local.get $ptr) (local.get $age)) ;; age
    (i32.store offset=12 (local.get $ptr) (i32.const 0)) ;; has_email
    (i32.store offset=16 (local.get $ptr) (i32.const 0)) ;; email_ptr
    (i32.store offset=20 (local.get $ptr) (i32.const 0)) ;; email_len

    (local.get $ptr)
  )
  (export "person_new" (func $person_new))

  ;; Set person name
  (func $person_set_name (param $person i32) (param $str_ptr i32) (param $str_len i32)
    (i32.store (local.get $person) (local.get $str_ptr))
    (i32.store offset=4 (local.get $person) (local.get $str_len))
  )
  (export "person_set_name" (func $person_set_name))

  ;; Get person age
  (func $person_get_age (param $person i32) (result i32)
    (i32.load offset=8 (local.get $person))
  )
  (export "person_get_age" (func $person_get_age))

  ;; Set person email (option type - can be null)
  (func $person_set_email (param $person i32) (param $str_ptr i32) (param $str_len i32)
    (i32.store offset=12 (local.get $person) (i32.const 1)) ;; has_email = true
    (i32.store offset=16 (local.get $person) (local.get $str_ptr))
    (i32.store offset=20 (local.get $person) (local.get $str_len))
  )
  (export "person_set_email" (func $person_set_email))

  ;; Check if person has email (option type)
  (func $person_has_email (param $person i32) (result i32)
    (i32.load offset=12 (local.get $person))
  )
  (export "person_has_email" (func $person_has_email))

  ;; Get name pointer
  (func $person_get_name_ptr (param $person i32) (result i32)
    (i32.load (local.get $person))
  )
  (export "person_get_name_ptr" (func $person_get_name_ptr))

  ;; Get name length
  (func $person_get_name_len (param $person i32) (result i32)
    (i32.load offset=4 (local.get $person))
  )
  (export "person_get_name_len" (func $person_get_name_len))

  ;; Get email pointer
  (func $person_get_email_ptr (param $person i32) (result i32)
    (i32.load offset=16 (local.get $person))
  )
  (export "person_get_email_ptr" (func $person_get_email_ptr))

  ;; Get email length
  (func $person_get_email_len (param $person i32) (result i32)
    (i32.load offset=20 (local.get $person))
  )
  (export "person_get_email_len" (func $person_get_email_len))

  ;;=== List Operations ===

  ;; List layout:
  ;; offset+0: length (i32)
  ;; offset+4: capacity (i32)
  ;; offset+8: data_ptr (i32)

  (func $list_new (param $capacity i32) (result i32)
    (local $list_ptr i32)
    (local $data_ptr i32)

    ;; Allocate list header (12 bytes)
    (local.set $list_ptr (call $alloc (i32.const 12)))

    ;; Allocate data array (capacity * 4 bytes for i32 pointers)
    (local.set $data_ptr (call $alloc (i32.mul (local.get $capacity) (i32.const 4))))

    ;; Initialize list
    (i32.store (local.get $list_ptr) (i32.const 0))              ;; length = 0
    (i32.store offset=4 (local.get $list_ptr) (local.get $capacity)) ;; capacity
    (i32.store offset=8 (local.get $list_ptr) (local.get $data_ptr)) ;; data_ptr

    (local.get $list_ptr)
  )
  (export "list_new" (func $list_new))

  ;; Add item to list (stores pointer)
  (func $list_push (param $list i32) (param $item i32)
    (local $length i32)
    (local $data_ptr i32)

    (local.set $length (i32.load (local.get $list)))
    (local.set $data_ptr (i32.load offset=8 (local.get $list)))

    ;; Store item at data_ptr + (length * 4)
    (i32.store
      (i32.add (local.get $data_ptr) (i32.mul (local.get $length) (i32.const 4)))
      (local.get $item)
    )

    ;; Increment length
    (i32.store (local.get $list) (i32.add (local.get $length) (i32.const 1)))
  )
  (export "list_push" (func $list_push))

  ;; Get list length
  (func $list_length (param $list i32) (result i32)
    (i32.load (local.get $list))
  )
  (export "list_length" (func $list_length))

  ;; Get item at index
  (func $list_get (param $list i32) (param $index i32) (result i32)
    (local $data_ptr i32)
    (local.set $data_ptr (i32.load offset=8 (local.get $list)))
    (i32.load (i32.add (local.get $data_ptr) (i32.mul (local.get $index) (i32.const 4))))
  )
  (export "list_get" (func $list_get))

  ;;=== Variant/Enum Operations ===

  ;; Result variant layout:
  ;; offset+0: tag (0=Success, 1=NotFound, 2=InvalidInput)
  ;; offset+4: data_ptr (for InvalidInput message)
  ;; offset+8: data_len (for InvalidInput message)

  (func $result_success (result i32)
    (local $ptr i32)
    (local.set $ptr (call $alloc (i32.const 12)))
    (i32.store (local.get $ptr) (i32.const 0)) ;; tag = Success
    (local.get $ptr)
  )
  (export "result_success" (func $result_success))

  (func $result_not_found (result i32)
    (local $ptr i32)
    (local.set $ptr (call $alloc (i32.const 12)))
    (i32.store (local.get $ptr) (i32.const 1)) ;; tag = NotFound
    (local.get $ptr)
  )
  (export "result_not_found" (func $result_not_found))

  (func $result_invalid (param $msg_ptr i32) (param $msg_len i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (call $alloc (i32.const 12)))
    (i32.store (local.get $ptr) (i32.const 2))              ;; tag = InvalidInput
    (i32.store offset=4 (local.get $ptr) (local.get $msg_ptr))
    (i32.store offset=8 (local.get $ptr) (local.get $msg_len))
    (local.get $ptr)
  )
  (export "result_invalid" (func $result_invalid))

  ;; Get result tag
  (func $result_get_tag (param $result i32) (result i32)
    (i32.load (local.get $result))
  )
  (export "result_get_tag" (func $result_get_tag))

  ;; Get error message pointer (for InvalidInput)
  (func $result_get_msg_ptr (param $result i32) (result i32)
    (i32.load offset=4 (local.get $result))
  )
  (export "result_get_msg_ptr" (func $result_get_msg_ptr))

  ;; Get error message length
  (func $result_get_msg_len (param $result i32) (result i32)
    (i32.load offset=8 (local.get $result))
  )
  (export "result_get_msg_len" (func $result_get_msg_len))
)
