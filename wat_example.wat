# unfortunetly there is no way to have funny shebang using wat comments or constructs :(
set -ex

mkfile() { grep -q "^$1$" .git/info/exclude || echo "$1" >>.git/info/exclude || true; }
mkfile "wat_example.wasm"
mkfile "wat_example.bin.log"

export PATH="$PWD/../wabt/build:$PWD/../wasm3/build:$PATH" 
sed -n '/^\;\; WAT_START/,$p' "$0" | wat2wasm --enable-annotations - -o wat_example.wasm || status=$?

if [ -z $status ]; then
	echo "wat_example.wasm:" >wat_example.bin.log
	xxd wat_example.wasm >>wat_example.bin.log
	echo -e "\nwat_example.wasm DUMP:" >>wat_example.bin.log
	wasm-tools dump wat_example.wasm >>wat_example.bin.log
	echo -e "\nwat_example.wasm OBJDUMP:" >>wat_example.bin.log
	wasm-objdump -s -x -d wat_example.wasm >>wat_example.bin.log
    wasm3 wat_example.wasm "$@"
fi

exit ${status:-0}

;; WAT_START

(module
  ;; Section 0: Custom Section
  ;; (@custom "debug_info" "example_metadata_bytes")

  ;; Section 1: Type Section
  (type $fd_write_sig (func (param i32 i32 i32 i32) (result i32)))
  (type $void_sig (func))
  (type $unary_sig (func (param i32) (result i32))) ;; Signature for indirect calls

  ;; Section 2: Import Section
  (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (type $fd_write_sig)))
  (import "wasi_snapshot_preview1" "fd_write" (func $fd_write4 (param i32 i32 i32 i32) (result i32)))

  ;; Section 3 (Function) & Section 10 (Code)
  (func $init (type $void_sig)
    (global.set $initialized (i32.const 1))
  )

  (func $dummy (type $void_sig)
    nop
  )

  ;; Target function 1 (at table index 0)
  (func $add_ten (type $unary_sig) (param $x i32) (result i32)
    local.get $x
    i32.const 10
    i32.add
  )

  ;; Target function 2 (at table index 1)
  (func $double (type $unary_sig) (param $x i32) (result i32)
    local.get $x
    i32.const 2
    i32.mul
  )

  ;; Function that receives a "function reference" (table index) and calls it
  (func $invoke_by_ref (param $func_idx i32) (param $val i32) (result i32)
    local.get $val        ;; Argument passed to target function
    local.get $func_idx   ;; Table index acting as function reference
    call_indirect (type $unary_sig) ;; Dynamic dispatch via table lookup
  )

  (func $main (type $void_sig)
    ;; Call $add_ten (index 0) with value 5
    (call $invoke_by_ref (i32.const 0) (i32.const 5))
    drop

    ;; Call $double (index 1) with value 5
    (call $invoke_by_ref (i32.const 1) (i32.const 5))
    drop

    ;; Original stdout call
    (call $fd_write4
      (i32.const 1)  ;; stdout file descriptor
      (i32.const 20) ;; pointer to iovec
      (i32.const 1)  ;; iovec count
      (i32.const 28) ;; pointer to store nwritten bytes
    )
    drop
  )

  ;; Section 4: Table Section (Expanded to hold 2 function pointers)
  (table $tbl 2 funcref)

  ;; Section 5: Memory Section
  (memory $mem 1)

  ;; Section 6: Global Section
  (global $initialized (mut i32) (i32.const 0))

  ;; Section 7: Export Section
  (export "memory" (memory $mem))
  (export "_start" (func $main))
  (export "status" (global $initialized))
  (export "invoke_by_ref" (func $invoke_by_ref))

  ;; Section 8: Start Section
  (start $init)

  ;; Section 9: Element Section (Populates index 0=$add_ten and index 1=$double)
  (elem (i32.const 0) $add_ten $double)

  ;; Section 11: Data Section
  (data (i32.const 0) "Hello from WASI!\n")
  (data (i32.const 20) "\00\00\00\00\11\00\00\00")
)
