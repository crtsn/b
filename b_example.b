/*/b/shebang/$0_$(shuf -i 1-100 -n 1 2>/dev/null)_$0/start 2>/dev/null
# could be rewritten with awk, but this is funnier
set -ex

make build/b
export RUST_BACKTRACE=1

mkfile() { grep -q "^$1$" .git/info/exclude || echo "$1" >>.git/info/exclude || true; }
mkfile "mos.log"
mkfile "uxn.log"
mkfile "mos.bin.log"
mkfile "uxn.bin.log"
mkfile "ir.log"
mkfile "b_example"
mkfile "b_example.rom"
mkfile "b_example.6502"

SCRIPT_DIR=$( dirname -- "$( readlink -f -- "$0"; )"; )
cd $SCRIPT_DIR
export PATH="$PWD/build:$PWD/../posix6502/build:$PWD/../uxncli/bin:$PATH" 
statuses=()
b -hist $0 -q -t 6502-posix -ir >ir.log
b -hist $0 -q -t 6502-posix || statuses[1]=$?
if [ -z ${statuses[1]} ]; then 
	echo "b_example.6502:" >mos.bin.log
	xxd b_example.6502 >>mos.bin.log
fi
b -hist $0 -q -t uxn || statuses[2]=$?
if [ -z ${statuses[1]} ]; then 
	echo "b_example.rom:" >uxn.bin.log
	xxd b_example.rom >>uxn.bin.log
fi
# exit 0

# b -q -hist $0 -t gas-x86_64-linux -o "b_example" || statuses[0]=$?
# b -hist $0 -q -t 6502-posix || statuses[1]=$?
# b -hist $0 -q -t uxn || statuses[2]=$?
for status in "${statuses[@]}"; do
	(exit $status)
done
# b -nobuild -run -hist $0 -q -t gas-x86_64-linux -o "b_example" || true
# b -nobuild -run -hist $0 -q -t 6502-posix >mos.log 2>&1 || statuses[1]=$?
# b -nobuild -run -hist $0 -q -t uxn -C "runner=uxncli" >uxn.log 2>&1 || true
posix6502 b_example.6502 >mos.log 2>&1 || statuses[1]=$?
echo ${statuses[1]}
uxncli b_example.rom >uxn.log 2>&1 || statuses[2]=$?
echo ${statuses[2]}
exit 0
*/

A "ABCDEFGHIJKLMNOPQ";                                                /* */
B 'AB', 'CD', 'EF', 'GH';                                             /* */
C 041101, 042103, 043105;                                             /* */
D 040502, 041504, 042506;                                             /* */
DELIM2 'D2';                                                          /* */
CHAR 'CH';                                                            /* */
CHAR2 'C';                                                            /* */
DELIM3 'D3';                                                          /* */
DELIM0 "D0";                                                          /* */
STRING "S";                                                           /* */
DELIM1 "D1";                                                          /* */
W;                                                                    /* */

main()
{
    W = &0[1];                                                        /* */
	printn(DELIM2, 10, 1);
	putchar('*n*n');                                                  /* */
	printf("%x*n", *&DELIM2);                                         /* 4432 */
	printf("%x*n", DELIM2);                                           /* 4432 */
	printf("%c*n", DELIM2);                                           /* '2' */
	putchar(DELIM2);                                                  /* '2' */
	putchar('*n*n');                                                  /* */
	putchar(char(&DELIM2, 0));                                        /* 'D' */
	putchar(char(&DELIM2, 1));                                        /* '2' */
	putchar('*n*n');                                                  /* */
	putchar(char(DELIM1, 0));                                         /* 'D' */
	putchar(char(DELIM1, 1));                                         /* '1' */
	putchar('*n*n');                                                  /* */
	putchar(CHAR);                                                    /* 'H' */
	putchar('*n*n');                                                  /* */
	putchar(CHAR2 == '*0C' ? 'Y' : 'N');                              /* 'Y' */
	putchar('*n*n');                                                  /* */
	putchar('**\');                                                   /* '\' */
	putchar('\**');                                                   /* '*' */
	putchar('*n*n');                                                  /* */
	putchar(char(STRING, 0));                                         /* 'S' */
	putchar('*n*n');                                                  /* */
	putchar(C);                                                       /* 'A' */
	putchar('*n*n');                                                  /* */
	putchar(char(&C, 0));                                             /* 'B' */
	putchar(char(&D, 0));                                             /* 'A' */
	putchar('*n*n');                                                  /* */
	/* printf("W: %d*n", W);                                          /* */
	/* printf("is LE: %c*n", is_le() ? 'Y' : 'N');                    /* */
	/* printf("is BE: %c*n", is_be() ? 'Y' : 'N');                    /* */
	/* printf("---*n");                                               /* */
	/* printf("0x%x, 0x%x, 0x%x*n", 'AB', 041101, 040502);            /* */
	/* printf("0x%x, 0x%x, 0x%x*n", 'A', 0040400, 0000101);           /* */
	/* printf("---*n");                                               /* */
	/* print_addr_deref("&c", &CHAR);                                 /* */
	/* print_addr_deref("s", STRING);                                 /* */
	/* print_three_derefs("A", A);                                    /* */
	/* print_three_derefs("&B", &B);                                  /* */
	/* print_three_derefs("&C", &C);                                  /* */
	/* print_three_derefs("&D", &D);                                  /* */
	exit(15);                                                         /* */
	return(89);                                                       /* */
}

is_le()
{
	return('AB' == 041101);                                           /* */
}

is_be()
{
	return('AB' == 040502);                                           /* */
}

print_addr_deref(label, addr)
{
	printf("%s: %p*n", label, addr);                                  /* */
	printf("%s for mos: %p*n", label, addr - 0100000);                /* */
	printf("%s for uxn: %p*n", label, addr - 0400);                   /* */
	printf("%s: 0x%x: '%c'*n", label, *addr, *addr);                  /* */
	printf("%s: hi: 0x%x*n", label, (*addr >> 8) & 0377);             /* */
	printf("%s: lo: 0x%x*n", label, *addr & 0377);                    /* */
	printf("%s: char(0): 0x%x*n", label, char(addr, 0));              /* */
	printf("%s: char(1): 0x%x*n", label, char(addr, 1));              /* */
	printf("*n");                                                     /* */
}

print_three_derefs(label, addr)
{
	auto i;                                                           /* */
	auto cur;                                                         /* */
	auto temp_label 8;                                                /* */
	i = 0;                                                            /* */
	cur = char(label, i);                                             /* */
	while(cur != '*0')
	{
		lchar(temp_label, i, cur);                                    /* */
        i++;                                                          /* */
        cur = char(label, i);                                         /* */
	}
	lchar(temp_label, i, cur);                                        /* */
	print_addr_deref(temp_label, addr);                               /* */
	lchar(temp_label, i, ' ');                                        /* */
	lchar(temp_label, i + 1, '+');                                    /* */
	lchar(temp_label, i + 2, ' ');                                    /* */
	lchar(temp_label, i + 3, '1');                                    /* */
	lchar(temp_label, i + 4, '*0');                                   /* */
	print_addr_deref(temp_label, addr + 1);                           /* */
	lchar(temp_label, i + 3, 'W');                                    /* */
	print_addr_deref(temp_label, addr + W);                           /* */
	printf("---*n");                                                  /* */
}

/*
printf() {}
/* */
