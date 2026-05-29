/* Standard Library for the Uxn target */

/*
value = uxn_dei(device);
reads 8 bit value off a device
*/

uxn_dei __asm__(
    "lit 0", "lit 4", "stz", /* zero the high byte of arg0/return */
    "lit 5", "ldzk", /* low byte of arg0 */
    "dei",
    "swp",
    "stz",
    "jmp2r"
);

/*
value = uxn_dei2(device);
reads 16 bit value off a device
*/

uxn_dei2 __asm__(
    "lit 5", "ldz", /* low byte of arg0 */
    "dei2",
    "lit 4", "stz2",
    "jmp2r"
);

/*
uxn_deo(device, value);
outputs 8 bit value to a device
*/

uxn_deo __asm__(
    "lit 7", "ldz", /* low byte of arg1 */
    "lit 5", "ldz", /* low byte of arg0 */
    "deo",
    "lit2 0", "lit 4", "stz2", /* return 0 */
    "jmp2r"
);

/*
uxn_deo2(device, value);
outputs 16 bit value to a device
*/

uxn_deo2 __asm__(
    "lit 6", "ldz2", /* arg1 */
    "lit 5", "ldz", /* low byte of arg0 */
    "deo2",
    "lit2 0", "lit 4", "stz2", /* return 0 */
    "jmp2r"
);

fputc(c, fd) {
    uxn_deo(fd + 030, c); /* 0x18 - Console/write,
                             0x19 - Console/error */
}

putchar(c) {
    fputc(c, 0);
}

exit(code) {
    uxn_deo(017, code | 0200); /* System/state */
}

_exit_after_main 1;

uxn_disable_exit_after_main() {
    _exit_after_main = 0;
}

_exit_main(code) {
    if (_exit_after_main) {
        exit(code);
    }
}

abort() {
    printf("Aborted\n");
    exit(1);
}

/* TODO: doesn't skip whitespace, doesn't handle negative numbers */
atoi(s) {
    auto i, result, c;
    i = 0;
    while (1) {
        c = char(s, i++);
        if (c < '0' | c > '9') {
            goto out;
        }
        result = result * 10 + (c - '0');
    }
out:
    return (result);
}

/* simple bump allocator */

__alloc_ptr 0100000; /* provide __heap_base by the compiler? */

malloc(size) {
    auto ret;
    ret = __alloc_ptr;
    __alloc_ptr =+ size;
    return (ret);
}

memset(addr, val, size) {
    auto i;
    i = 0;
    while (i < size) {
        lchar(addr, i, val);
        i =+ 1;
    }
}

stdout 0; stderr 1;

_args_count 1;
_args_items 077400; /* 128 arguments ought to be enough for everyone */
_prog_name "-";

_start_with_arguments() {
    auto type, c;
    type = uxn_dei(027); /* Console/type */
    c = uxn_dei(022);
    if (type == 2) { /* argument */
        lchar(__alloc_ptr++, 0, c);
    } else if (type == 3) { /* argument spacer */
        lchar(__alloc_ptr++, 0, 0);
        *(_args_items + (_args_count++)*2) = __alloc_ptr;
    } else if (type == 4) { /* arguments end */
        lchar(__alloc_ptr++, 0, 0);
        uxn_deo2(020, 0);
        _exit_main(main(_args_count, _args_items));
    }
}

_start() {
    *_args_items = _prog_name;
    if (uxn_dei(027) != 0) {
        *(_args_items + (_args_count++)*2) = __alloc_ptr;
        uxn_deo2(020, &_start_with_arguments);
    } else {
        _exit_main(main(_args_count, _args_items));
    }
}

