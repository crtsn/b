/*/b/shebang/$0_$(shuf -i 1-100 -n 1 2>/dev/null)_$0/start 2>/dev/null
# could be rewritten with awk, but this is funnier
set -e

TMP_BIN="./b_example"
cleanup() {
	rm -f "$TMP_BIN" b_example.rom b_example.6502
}

trap cleanup EXIT INT TERM
SCRIPT_DIR=$( dirname -- "$( readlink -f -- "$0"; )"; )
cd $SCRIPT_DIR
export PATH="$PWD/build:$PWD/../posix6502/build:$PWD/../uxncli/bin:$PATH" 
statuses=()
b -hist $0 -t gas-x86_64-linux -o "$TMP_BIN" || statuses[0]=$?
# ./build/b -hist $0 -q -t 6502-posix || statuses[1]=$?
# ./build/b -hist $0 -q -t uxn || statuses[2]=$?
for status in "${statuses[@]}"; do
	(exit $status)
done
b -nobuild -run -hist $0 -q -t gas-x86_64-linux -o "$TMP_BIN"
# b -nobuild -run -hist $0 -q -t 6502-posix
# b -nobuild -run -hist $0 -q -t uxn -C "runner=uxncli"
exit 0
*/

A "ABCDEFGHIJKLMNOPQ";                                                /* */
B 'AB', 'CD', 'EF', 'GH';                                             /* */
C 041101, 042103, 043105;                                             /* */
D 040502, 041504, 042506;                                             /* */
W;                                                                    /* */

main()
{
    W = &0[1];                                                        /* */
	printf("is LE: %c*n", is_le() ? 'Y' : 'N');                       /* */
	printf("is BE: %c*n", is_be() ? 'Y' : 'N');                       /* */
	printf("---*n");                                                  /* */
	printf("char(&A[1], -2): '%c'*n", char(&A[1], -2));               /* */
	printf("char(&A[1], -1): '%c'*n", char(&A[1], -1));               /* */
	printf("char(&A[1],  0): '%c'*n", char(&A[1],  0));               /* */
	printf("char(&A[1],  1): '%c'*n", char(&A[1],  1));               /* */
	printf("char(&A[1],  2): '%c'*n", char(&A[1],  2));               /* */
	printf("---*n");                                                  /* */
	printf("char((A + 2), -2): '%c'*n", char((A + 2), -2));           /* */
	printf("char((A + 2), -1): '%c'*n", char((A + 2), -1));           /* */
	printf("char((A + 2),  0): '%c'*n", char((A + 2), 0));            /* */
	printf("char((A + 2),  1): '%c'*n", char((A + 2), 1));            /* */
	printf("char((A + 2),  2): '%c'*n", char((A + 2), 2));            /* */
	printf("---*n");                                                  /* */
	printf("char(&(&B)[1], -2): '%c'*n",  char(&(&B)[1], -2));        /* */
	printf("char(&(&B)[1], -1): '%c'*n",  char(&(&B)[1], -1));        /* */
	printf("char(&(&B)[1],  0): '%c'*n",  char(&(&B)[1],  0));        /* */
	printf("char(&(&B)[1],  1): '%c'*n",  char(&(&B)[1],  1));        /* */
	printf("char(&(&B)[1],  2): '%c'*n",  char(&(&B)[1],  2));        /* */
	printf("---*n");                                                  /* */
	printf("char((&B + W + 2), -2): '%c'*n", char((&B + W + 2), -2)); /* */
	printf("char((&B + W + 2), -1): '%c'*n", char((&B + W + 2), -1)); /* */
	printf("char((&B + W + 2),  0): '%c'*n", char((&B + W + 2),  0)); /* */
	printf("char((&B + W + 2),  1): '%c'*n", char((&B + W + 2),  1)); /* */
	printf("char((&B + W + 2),  2): '%c'*n", char((&B + W + 2),  2)); /* */
	printf("---*n");                                                  /* */
	printf("char((&B + W), -2): '%c'*n", char((&B + W), -2));         /* */
	printf("char((&B + W), -1): '%c'*n", char((&B + W), -1));         /* */
	printf("char((&B + W),  0): '%c'*n", char((&B + W),  0));         /* */
	printf("char((&B + W),  1): '%c'*n", char((&B + W),  1));         /* */
	printf("char((&B + W),  2): '%c'*n", char((&B + W),  2));         /* */
	printf("---*n");                                                  /* */
	printf("**((&B + W) - 2): '%c'*n", *((&B + W) - 2));              /* */
	printf("**((&B + W) - 1): '%c'*n", *((&B + W) - 1));              /* */
	printf("**((&B + W) + 0): '%c'*n", *((&B + W) + 0));              /* */
	printf("**((&B + W) + 1): '%c'*n", *((&B + W) + 1));              /* */
	printf("**((&B + W) + 2): '%c'*n", *((&B + W) + 2));              /* */
	printf("---*n");                                                  /* */
	printf("char(&(&B)[0], 0): '%c'*n", char(&(&B)[0], 0));           /* */
	printf("char(&(&B)[0], 1): '%c'*n", char(&(&B)[0], 1));           /* */
	printf("char(&(&B)[1], 0): '%c'*n", char(&(&B)[1], 0));           /* */
	printf("char(&(&B)[1], 1): '%c'*n", char(&(&B)[1], 1));           /* */
	printf("char(&(&B)[2], 0): '%c'*n", char(&(&B)[2], 0));           /* */
	printf("char(&(&B)[2], 1): '%c'*n", char(&(&B)[2], 1));           /* */
	printf("---*n");                                                  /* */
	printf("**((&C + W) - 2): '%c'*n", *((&C + W) - 2));              /* */
	printf("**((&C + W) - 1): '%c'*n", *((&C + W) - 1));              /* */
	printf("**((&C + W) + 0): '%c'*n", *((&C + W) + 0));              /* */
	printf("**((&C + W) + 1): '%c'*n", *((&C + W) + 1));              /* */
	printf("**((&C + W) + 2): '%c'*n", *((&C + W) + 2));              /* */
	printf("---*n");                                                  /* */
	printf("char((&C + W), -2): '%c'*n", char((&C + W), -2));         /* */
	printf("char((&C + W), -1): '%c'*n", char((&C + W), -1));         /* */
	printf("char((&C + W),  0): '%c'*n", char((&C + W),  0));         /* */
	printf("char((&C + W),  1): '%c'*n", char((&C + W),  1));         /* */
	printf("char((&C + W),  2): '%c'*n", char((&C + W),  2));         /* */
	printf("---*n");                                                  /* */
	printf("**((&D + W) - 2): '%c'*n", *((&D + W) - 2));              /* */
	printf("**((&D + W) - 1): '%c'*n", *((&D + W) - 1));              /* */
	printf("**((&D + W) + 0): '%c'*n", *((&D + W) + 0));              /* */
	printf("**((&D + W) + 1): '%c'*n", *((&D + W) + 1));              /* */
	printf("**((&D + W) + 2): '%c'*n", *((&D + W) + 2));              /* */
	printf("---*n");                                                  /* */
	printf("char((&D + W), -2): '%c'*n", char((&D + W), -2));         /* */
	printf("char((&D + W), -1): '%c'*n", char((&D + W), -1));         /* */
	printf("char((&D + W),  0): '%c'*n", char((&D + W),  0));         /* */
	printf("char((&D + W),  1): '%c'*n", char((&D + W),  1));         /* */
	printf("char((&D + W),  2): '%c'*n", char((&D + W),  2));         /* */
	printf("---*n");                                                  /* */
	printf("0101 == 'A': %c*n", 0101 == 'A' ? 'Y' : 'N');             /* */
	auto E;                                                           /* */
	E = "ABC";                                                        /* */
	printf("E[0]: 0x%x*n", E[0]);                                     /* */
	printf("E[0] & 0377: 0x%x*n", E[0] & 0377);                       /* */
	printf("E[0] == 'A': %c*n", E[0] == 'A' ? 'Y' : 'N');             /* */
	printf("*n*n*n");                                                 /* */
}

is_le()
{
	return('AB' == 041101);                                           /* */
}

is_be()
{
	return('AB' == 040502);                                           /* */
}

