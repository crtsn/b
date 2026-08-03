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

lchar(s, i, c) {
    auto shift;

    shift = (&0[1] - 1 - (i % &0[1])) * 8;
    s[i / &0[1]] = (s[i / &0[1]] & (-1 - (0377 << shift))) | ((c & 0377) << shift);

    return(c);
}

printn(n, b, sign) {
    extrn putchar;
    auto a, c;

    if (sign & (n < 0)) {
        putchar('-');
        n = -n;
    }

    if (n < 0) {
        a = (n >> 1) & ((1 << ((&0[1] << 3) - 1)) - 1); /* Temporary 'k' */
        c = ((a % b) << 1) | (n & 1);                  /* Temporary 'rem' */
        a = ((a / b) << 1) + (c / b);                  /* Final quotient 'a' */
        c = c % b;                                     /* Final remainder 'c' */
    } else {
        a = n / b;
        c = n % b;
    }

    if (a)
        printn(a, b, 0);

    c =+ '0';
    if (c > '9') c =+ 7;
    putchar(c);
}

printf(str, x1, x2, x3, x4, x5, x6, x7, x8, x9) {
    auto i, j, arg, c;
    i = 0;
    arg = &x1;

    c = char(str, i);
    while ((c != '*e') & (c != 0)) {
        if (c == '%') {
            i++;
            c = char(str, i);

            while ((c == 'l') | (c == 'z') | (c == 'h')) {
                i++;
                c = char(str, i);
            }

            if ((c == 0) | (c == '*e')) {
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
            } else if (c == 'c') {
                putchar(*arg);
                arg = arg - &0[1];
            } else if (c == 's') {
                j = 0;
                c = char(*arg, j);
                while ((c != '*e') & (c != 0)) {
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

