/*
 * Testing ability to use const chars and octal constants anywhere
 * and also checking that constants for any platform cannot exceed word size
 * and also that char consts are not bigger/less than a word size for platform
 */
/* TODO: make const test for each platform */
/* TODO: split into separate tests if needed */

E00 48;
E01 041;
E02[1] 0130;
E03[034] 0114, 0112;
/* E04 010210110099989796; */ /* weird error if has digits > 9 */
E05 01777777777777777777777, 0;
/* A+B=C in reverse order -> ((0103<<32)+(0075<<24)+(0102<<16)+(0053<<8)+0101) */
E06 04147520425501, 0;

E10 'UM';
E11[1] 'GI';
/* E12['34'] '112'; */ /* Parsing error */
E13 'U+V+W+X+', 'Y=Z', 0;
E14 "K+L+N+O=M";
/* E15 'K+L+N+O=M'; */

main()
{
    extrn printf;

    auto Aa 5;
    /* auto Ac 0; */ /* Not implemented; vector of size 0 should probably be a simple variable that stores address */
    auto Ad 04;
    Ad[01] = 'A';
    /* Ad[02] = 08; */ /* weird error */
    Ad[03] = 07;

    printf("%c*n", E00);
    printf("%c*n", E01);
    printf("%c*n", *E02);
    printf("%c*n", E03[1]);
    printf("%s*n", &E06);

    printf("%c%c*n", E10, *(&E10 + 1));
    printf("%c%c*n", *E11, *(E11 + 1));

    printf("%s*n", &E13);
    printf("%s*n", E14);

    printf("%c*n", *(&E13 + 2));
    printf("%c*n", *(E14 + 2));

    printf("%c*n", char(&E13, 2));
    printf("%c*n", char(E14, 2));

    Aa[2] = 'UF';
    printf("%c%c*n", Aa[2], *(&Aa[2] + 1));
}

