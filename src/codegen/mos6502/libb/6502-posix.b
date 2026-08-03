exit(code) {
    0(code);
}

abort() {
    exit(69);
}

putchar(c) {
    0177757(c&0377);
}

/* TODO: fd not supported */
fputc(c, fd) {
    putchar(c);
}

/* TODO: actually allocate something */
__heap_ptr 01000;
malloc(size) {
    extrn printf;
    auto ptr;
    ptr = __heap_ptr;
    __heap_ptr =+ size;
    if (__heap_ptr >= 010000) {
        printf("Allocation reached end: %p*nTODO: allow allocating more, implement free*n", __heap_ptr);
        abort();
    }
    return (ptr);
}
/* TODO: free someting? */
realloc(ptr, size) {
    return (malloc(size));
}

/* TODO: Try to implement this function with assembly
   Problem with this implementation is that it is not
   mapped to the operator
   We cannot call this function `div` as it conflicts
   with the `divmod` test
*/
/* rewritten with shifts with LLM */
_div(a, b) {
    auto q, mask, sign;

    if (b == 0) return (0);

    sign = 0;
    if (a < 0) { sign = !sign; a = -a; }
    if (b < 0) { sign = !sign; b = -b; }

    q = 0;
    mask = 1;

    while (b <= (a >> 1)) {
        b = b << 1;
        mask = mask << 1;
    }

    while (mask) {
        if (a >= b) {
            a = a - b;
            q = q | mask;
        }
        b = b >> 1;
        mask = mask >> 1;
    }

    return (sign ? -q : q);
}

/* TODO: Try to implement this function with assembly
   Problem with this implementation is that it is not
   mapped to the operator */
/* rewritten with shifts with LLM */
_rem(a, b) {
    auto mask, rsign;

    if (b == 0) return (0);

    rsign = (a < 0);
    if (a < 0) a = -a;
    if (b < 0) b = -b;

    mask = 1;

    while (b <= (a >> 1)) {
        b = b << 1;
        mask = mask << 1;
    }

    while (mask) {
        if (a >= b) {
            a = a - b;
        }
        b = b >> 1;
        mask = mask >> 1;
    }

    return (rsign ? -a : a);
}

strlen(s) {
    auto n;
    n = 0;
    while (char(s, n)) n++;
    return (n);
}

toupper(c) {
    if ('a' <= c & c <= 'z') return (c - 'a' + 'A');
    return (c);
}


/* memory related functions */
memset(addr, val, size) {
    extrn lchar;
    auto i;
    i = 0;
    while (i < size) {
        lchar(addr, i, val);
        i =+ 1;
    }
}
