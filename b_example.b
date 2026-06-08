/*/b/shebang/$0_$(shuf -i 1-100 -n 1 2>/dev/null)_$0/start 2>/dev/null
set -e

TMP_BIN="./b_example"
cleanup() {
	rm -f "$TMP_BIN" b_example.rom b_example.6502
}
trap cleanup EXIT INT TERM
./build/b -hist $0 -q -t gas-x86_64-linux -o "$TMP_BIN"
./build/b -hist $0 -q -t 6502-posix # -nostdlib -ir
# ./build/b -hist $0 -q -t uxn
"$TMP_BIN" "$@"
$PWD/../posix6502/build/posix6502 b_example.6502
exit 0
*/

/* E00 010120441504; /* */
/* E01 "ABCDEFGH67", 0; /* */
/* E02 "ABCD", 0424000000000000033067; /* */
E03[] 'Я', 'Е', 'Б', 00, '/\', 044101; /* */

main()
{
	auto a; /* */
	a = 0377 + 01 + 02; /* */
	a++;
	a =+ 07;
	extrn putchar; /* */
	extrn printf; /* */
	auto W; /* */
    W = &0[1]; /* */
	printf("OGO!*n"); /* */
	printf("%o*n", 0377); /* */
	printf("%o*n", *(E03 + 0) & 0377); /* */
	printf("%o*n", *(E03 + 1) & 0377); /* */
	printf("%o*n", *(E03 + (0 * W)) & 0377); /* */
	printf("%o*n", *(E03 + (1 * W)) & 0377); /* */
	printf("%s*n", E03);				 /* */
	printf("%c%c*n", *(E03 + 0) & 0377, *(E03 + 1) & 0377);	 /* */
}
