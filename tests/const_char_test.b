/*
 * Testing ability to use const chars and octal constants anywhere
 * and also checking that constants for any platform cannot exceed word size
 * and also that char consts are not bigger/less than a word size for platform
 */
/* TODO: split into separate tests if needed */
/* TODO: remove all fflush-es after fixing all segfaults and errors */

E00 48;
E01 041;
E02[1] 0130;
E03[034] 0114, 0112;
/* E04 010210110099989796; */ /* weird error */
E05 0102101100;

E10 'UM';
E11[1] 'GI';
/* E12['34'] '112'; */ /* Parsing error */
/* E13[] '115'; */ /* character more than 2 even if fits the word */

main()
{
    extrn printf, fflush, stdout;

    auto Aa 5;
    /* auto Ab 0o; */ /* weird, this worked but core dumped compiler */
    /* auto Ac 0o4; */ /* Not implemented, should not work with -hist */
    auto Ad 04;
    Ad[01] = 'A';
    /* Ad[02] = 08; */ /* weird error */
    Ad[03] = 07;

    printf("%c*n", E00); fflush(stdout);
    printf("%c*n", E01); fflush(stdout);
    printf("%c*n", *E02); fflush(stdout);
    printf("%c*n", E03[1]); fflush(stdout);
    printf("%s*n", E05); fflush(stdout);

    printf("%c%c*n", E10, *(&E10 + 1)); fflush(stdout);
    printf("%c%c*n", *E11, *(E11 + 1)); fflush(stdout);

    Aa[2] = 'UF';
    printf("%c%c*n", Aa[2], *(&Aa[2] + 1)); fflush(stdout);
}

