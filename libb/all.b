/* This file contains definitions that are cross-platform and shared across all targets.
 * This usually means that the code in this file is implemented on top of platform specific
 * parts code.
 */

/*
ch = char(string, i);
returns the ith character in a string pointed to by string, 0 based
*/
char(s, i) {
    auto w, b, c;
    w = 0;
    b = 0;

    while (1) {
        c = (s[w] >> ((&0[1] - 1 - b) * 8)) & 0377;

        if (c == '*e') {
            return(c);
        } else if (c != '*0' && c != 0) {
            /* Found a non-zero character */
            if (i == 0) {
                return(c);
            } else {
                i--;
            }
        }

        b++;
        if (b >= &0[1]) {
            w++;
            b = 0;
        }
    }
}

/*
ch = lchar(string, i, char);
replaces the ith character in the string pointed to by string with the character char.
The value LCHAR returns is the character char that was placed in the string.
*/
lchar(s, i, c) {
    auto shift;

    shift = (&0[1] - 1 - (i % &0[1])) * 8;
    s[i / &0[1]] = (s[i / &0[1]] & (-1 - (0377 << shift))) | ((c & 0377) << shift);

    return(c);
}

/* 
   Non-recursive version, without fixed buffer, 
   so it will not stack overflow for mos6502,
   but also would be possible to run on any potential platform without fixed buffer
*/
printn(n, b, sign) {
    extrn putchar;
    auto a, c, count, i, t, more;

    if (sign & (n < 0)) {
        putchar('-');
        n = -n;
    }

    /* count digits */
    count = 0;
    t = n;
    more = 1;
    while (more) {
        count =+ 1;

        /* '/' operator is idiv, so we need to simulate unsigned division */
        if (t < 0) {
            /* ignore last bit for now (div by 2) */
            a = (t >> 1);
            /* Do unsigned rem to get last digit, and then * 2 to add last bit, to get original last digit */
            c = ((a % b) << 1) | (t & 1);
            /* Do unsigned div, to get rest of number, and then * 2 and add c/b if it is greater than base */
            t = ((a / b) << 1) + (c / b);
        } else {
            t = t / b;
        }

        if (t == 0)
            more = 0;
    }

    /* Redo the same but now print number left to right */
    while (count > 0) {
        count =- 1;

        t = n;
        i = count;
        while (i > 0) {
            /* same as above */
            if (t < 0) {
                a = (t >> 1);
                c = ((a % b) << 1) | (t & 1);
                t = ((a / b) << 1) + (c / b);
            } else {
                t = t / b;
            }
            i =- 1;
        }

        /* same as above, but just get last digit to print it */
        if (t < 0) {
            a = (t >> 1);
            c = (((a % b) << 1) | (t & 1)) % b;
        } else {
            c = t % b;
        }

        c =+ '0';
        if (c > '9') c =+ 7;
        putchar(c);
    }
}

/* same as printn(), but prints letters uppercase */
/* TODO: deduplicate if it could fit to mos6502 stack */
printn_upper(n, b, sign) {
    extrn putchar;
    auto a, c, count, i, t, more;

    if (sign & (n < 0)) {
        putchar('-');
        n = -n;
    }

    /* count digits */
    count = 0;
    t = n;
    more = 1;
    while (more) {
        count =+ 1;

        if (t < 0) {
            a = (t >> 1);
            c = ((a % b) << 1) | (t & 1);
            t = ((a / b) << 1) + (c / b);
        } else {
            t = t / b;
        }

        if (t == 0)
            more = 0;
    }

    while (count > 0) {
        count =- 1;

        t = n;
        i = count;
        while (i > 0) {
            /* same as above */
            if (t < 0) {
                a = (t >> 1);
                c = ((a % b) << 1) | (t & 1);
                t = ((a / b) << 1) + (c / b);
            } else {
                t = t / b;
            }
            i =- 1;
        }

        if (t < 0) {
            a = (t >> 1);
            c = (((a % b) << 1) | (t & 1)) % b;
        } else {
            c = t % b;
        }

        c =+ '0';
        if (c > '9') c =+ 7;
        putchar(toupper(c));
    }
}

printf(str, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12) {
    auto i, j, arg, c;
    i = 0;
    arg = &x1;

    c = char(str, i);
    while (c != '*e') {
        if (c == '%') {
            i++;
            c = char(str, i);

            while ((c == 'l') | (c == 'z') | (c == 'h')) {
                i++;
                c = char(str, i);
            }

            if (c == '*e') {
                return;
            } else if (c == '%') {
                putchar('%');
            } else if (c == 'd') {
                printn(*arg, 10, 1);
                arg = arg - &0[1];
            } else if (c == 'u') {
                printn(*arg, 10, 0);
                arg = arg - &0[1];
            } else if (c == 'p') {
                putchar('$');
                printn(*arg, 16, 0);
                arg = arg - &0[1];
            } else if (c == 'o') {
                printn(*arg, 8, 0);
                arg = arg - &0[1];
            } else if (c == 'x') {
                printn(*arg, 16, 0);
                arg = arg - &0[1];
            } else if (c == 'X') {
                printn_upper(*arg, 16, 0);
                arg = arg - &0[1];
            } else if (c == 'c') {
                putchar(*arg);
                arg = arg - &0[1];
            } else if (c == 's') {
                j = 0;
                c = char(*arg, j);
                while (c != '*e') {
                    putchar(c);
                    j++;
                    c = char(*arg, j);
                }
                arg = arg - &0[1];
            } else {
                putchar('%');
                putchar(c);
            }
        } else {
            putchar(c);
        }

        i++;
        c = char(str, i);
    }
}

strlen(s) {
    auto n;
    n = 0;
    while (char(s, n) != '*e') n++;
    return (n);
}

toupper(c) {
    if ('a' <= c & c <= 'z') return (c - 'a' + 'A');
    return (c);
}

