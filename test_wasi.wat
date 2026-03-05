(module
  (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
  (memory 1)
  (export "memory" (memory 0))
  (data (i32.const 8) "hello\n")
  (func $main (export "main")
    (i32.store (i32.const 0) (i32.const 8))  ;; iov.iov_base
    (i32.store (i32.const 4) (i32.const 6))  ;; iov.iov_len
    (call $fd_write
      (i32.const 1) ;; stdout
      (i32.const 0) ;; iovs
      (i32.const 1) ;; iovs_len
      (i32.const 20) ;; nwritten
    )
    drop
  )
)
