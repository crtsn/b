/*/b/shebang/$0_$(shuf -i 1-100 -n 1 2>/dev/null)_$0/start 2>/dev/null
# could be rewritten with awk, but this is funnier
set -e

make build/b
export RUST_BACKTRACE=1

grep -q "^a.log$" .git/info/exclude || echo "a.log" >>.git/info/exclude || true
grep -q "^b.log$" .git/info/exclude || echo "b.log" >>.git/info/exclude || true
grep -q "^ir.log$" .git/info/exclude || echo "ir.log" >>.git/info/exclude || true
grep -q "^b_example$" .git/info/exclude || echo "b_example" >>.git/info/exclude || true
grep -q "^b_example.rom$" .git/info/exclude || echo "b_example.rom" >>.git/info/exclude || true
grep -q "^b_example.6502$" .git/info/exclude || echo "b_example.6502" >>.git/info/exclude || true

SCRIPT_DIR=$( dirname -- "$( readlink -f -- "$0"; )"; )
cd $SCRIPT_DIR
export PATH="$PWD/build:$PWD/../posix6502/build:$PWD/../uxncli/bin:$PATH" 
statuses=()
b -hist $0 -q -t 6502-posix -ir >ir.log
b -hist $0 -q -t 6502-posix || statuses[1]=$?
if [ -z ${statuses[1]} ]; then 
	echo "b_example.6502:"
	xxd b_example.6502
fi;
b -hist $0 -q -t uxn || statuses[2]=$?
if [ -z ${statuses[1]} ]; then 
	echo "b_example.rom:"
	xxd b_example.rom
fi;
# exit 0

# b -q -hist $0 -t gas-x86_64-linux -o "b_example" || statuses[0]=$?
# b -hist $0 -q -t 6502-posix || statuses[1]=$?
# b -hist $0 -q -t uxn || statuses[2]=$?
for status in "${statuses[@]}"; do
	(exit $status)
done
# b -nobuild -run -hist $0 -q -t gas-x86_64-linux -o "b_example" || true
b -nobuild -run -hist $0 -q -t 6502-posix >a.log 2>&1 || true
b -nobuild -run -hist $0 -q -t uxn -C "runner=uxncli" >b.log 2>&1 || true
exit 0
*/

A "ABCDEFGHIJKLMNOPQ";                                                /* */
B 'AB', 'CD', 'EF', 'GH';                                             /* */
C 041101, 042103, 043105;                                             /* */
D 040502, 041504, 042506;                                             /* */
W;                                                                    /* */

print_addr_deref(label, addr)
{
	printf("%s: %p*n", label, addr);		              /* */
	printf("%s for mos: %p*n", label, addr - 0100000);		              /* */
	printf("%s for uxn: %p*n", label, addr - 0400);		              /* */
	printf("%s: 0x%x: '%c'*n", label, *addr, *addr);		              /* */
	printf("%s: hi: 0x%x*n", label, char(addr, 0));		              /* */
	printf("%s: lo: 0x%x*n", label, char(addr, 1));		              /* */
	printf("*n");		              /* */
}

print_three_derefs(label, addr)
{
	auto i;
	auto cur;
	auto temp_label 8;
	i = 0;
	cur = char(label, i);
	printf("cur: 0x%x*n", cur);
	while(cur != '*0')
	{
		lchar(temp_label, i, cur);
        i++;
        cur = char(label, i);
		printf("cur: 0x%x*n", cur);
	}
	lchar(temp_label, i, cur);
	printf("temp_lavel: *"%s*"*n", temp_label);
	print_addr_deref(temp_label, addr);
	lchar(temp_label, i, ' ');
	lchar(temp_label, i + 1, '+');
	lchar(temp_label, i + 2, ' ');
	lchar(temp_label, i + 3, '1');
	lchar(temp_label, i + 4, '*0');
	print_addr_deref(temp_label, addr + 1);
	lchar(temp_label, i + 3, 'W');
	print_addr_deref(temp_label, addr + W);
	printf("---*n");                                                  /* */
}


main()
{
    W = &0[1];                                                        /* */
	/* auto a;
	/* a = A + 2;
	printf("W: %d*n", W);                       					  /* */
	/* printf("is LE: %c*n", is_le() ? 'Y' : 'N');                       /* */
	/* printf("is BE: %c*n", is_be() ? 'Y' : 'N');                       /* */
	/* printf("---*n");                                                  /* */
	/* printf("A: %p*n", A);		              /* */
	/* printf("A for mos: %p*n", A - 0100000);		              /* */
	/* printf("A for uxn: %p*n", A - 0400);		              /* */
	print_three_derefs("A", A);                                         /* */
	print_three_derefs("&B", &B);                                         /* */
	print_three_derefs("&C", &C);                                         /* */
	print_three_derefs("&D", &D);                                         /* */
}

is_le()
{
	return('AB' == 041101);                                           /* */
}

is_be()
{
	return('AB' == 040502);                                           /* */
}

/*
printf() {}
/* */
