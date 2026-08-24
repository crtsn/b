/* Standard Library for the WASM 32 WASI target */
/* TODO: Add multiple libb support to compiler or to codegen if needed */

exit(code) {
}

putchar(c) {
    __asm__(
        "i32.const 1",
        "i32.const 20",
        "i32.const 1",
        "i32.const 28",
        "call $fd_write",
        "drop"
    );
}
