#!/bin/bash

set -e

export PATH="$PWD/build:$PWD/../posix6502/build:$PWD/../uxncli/bin:$PATH" 

make build/b
make build/btest

# ./build/btest -t gas-x86_64-linux
# ./build/btest -t uxn
# ./build/btest -t 6502-posix

# ./build/btest -t 6502-posix -c upper

# ./build/btest -t uxn -c call_stack_args -c deref_assign -c lexer
# ./build/b ./tests//asm_uxn.b -t uxn -o ./build/tests//asm_uxn.uxn.rom
# ./build/b ./tests//asm_func_uxn.b -t uxn -o ./build/tests//asm_func_uxn.uxn.rom

./build/btest -t 6502-posix -c call_stack_args -c asm_6502 -c asm_func_6502 -c args11
./build/b ./tests/args11.b -t 6502-posix -ir >ir.log
# ./build/b ./tests/asm_func_6502.b -t 6502-posix -o ./build/tests//asm_func_6502.6502-posix.6502
