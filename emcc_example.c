#if 0

mkfile() { grep -q "^$1$" .git/info/exclude || echo "$1" >>.git/info/exclude || true; }
mkfile "emcc_example.wasm"
mkfile "emcc_example.bin.log"

emcc $0 -O0 \
  -sSTANDALONE_WASM \
  -sEXPORTED_FUNCTIONS='["_add", "_main"]' \
  -o emcc_example.wasm

export PATH="$PWD/../wabt/build:$PWD/../wasm3/build:$PATH" 

if [ -z $status ]; then
	echo "emcc_example.wasm:" >emcc_example.bin.log
	xxd emcc_example.wasm >>emcc_example.bin.log
	echo -e "\nemcc_example.wasm DUMP:" >>emcc_example.bin.log
	wasm-tools dump emcc_example.wasm >>emcc_example.bin.log
	echo -e "\nemcc_example.wasm OBJDUMP:" >>emcc_example.bin.log
	wasm-objdump -s -x -d emcc_example.wasm >>emcc_example.bin.log
    wasm3 emcc_example.wasm "$@"
fi

exit ${status:-0}
#endif

// main.c
#include <stdio.h>
#include <emscripten.h>

// EMSCRIPTEN_KEEPALIVE prevents the compiler from optimizing this function away
EMSCRIPTEN_KEEPALIVE
int add(int a, int b) {
    return a + b;
}

int main() {
    printf("Hello from Wasm3!\n");
    return 0;
}

