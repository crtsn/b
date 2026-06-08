#if 0
set -e

TMP_BIN="./c_example"
cleanup() {
	rm -f "$TMP_BIN"
}
trap cleanup EXIT INT TERM
# you could also use clang-18 here
gcc -std=c99 -D_GNU_SOURCE -g -O0 -o "$TMP_BIN" "$0"
"$TMP_BIN" "$@"
exit 0
#endif

#include <stdio.h>
#include <stdint.h>
#include <string.h>

int E00 = 010120441504; */
int E01 = {'ABCDEFGH67', 0}; /* */
int E02 = {'ABCD', 0424000000000000033067}; /* */
int E03[] = {'ЯЕБLA', 044101}; /* */

int main()
{
	uint64_t z = 0x4500000000003637;
	uint64_t a = 0x4142434445464748;
	// uint16_t a[] = {0xDEF0, 0x9ABC, 0x5678, 0x1234};
	// uint8_t a[] = {0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12};
	// uint8_t a[] = "ABCDEFGH67";
	// uint8_t a[] = "ABCD";
	// uint64_t a = 'ABCD';
	// uint64_t a = '🥕';
	// uint8_t a[] = "🥕ABCDEFGH67";
	uint64_t e = 0x6564000000000035;
	// printf("%02X\n", *((uint8_t *) &a));
	// printf("%02X\n", *((uint8_t *) &a + 1));
	// printf("%02X\n", *((uint8_t *) &a + 2));
	// printf("%02X\n", *((uint8_t *) &a + 3));
	// printf("%02X\n", *((uint8_t *) &a + 4));
	// printf("%02X\n", *((uint8_t *) &a + 5));
	// printf("%02X\n", *((uint8_t *) &a + 6));
	// printf("%02X\n", *((uint8_t *) &a + 7));
	// printf("%02X\n", *((uint8_t *) &a + 8));
	// printf("%s\n", (uint8_t *) &a);
	// printf("%c\n", *(uint8_t *) &a);
}

