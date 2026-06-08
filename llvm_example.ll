source_filename="llvm_example.ll" ; lli-18 "$0" "$@"; exit

define i32 @main() {
    ret i32 42
}
