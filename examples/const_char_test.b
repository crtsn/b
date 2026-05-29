/*
 * Testing ability to use const chars and octal constants anywhere and
 * also checking that constants for any platform cannot exceed word size
 * and also that char constants are not bigger and not less
 * than a word size for platform
 */
o
Ea 10;
Eb 010;
Ec[1] 011;
Ed[034] 0112;
Ee '10';
Ef[1] '11';
Eg['34'] '112'; /* Parsing error */
Eh[] '115'; /* character more than 2 even if fits the word */

main()
{
    auto Aa 5;
    /* auto Ab 0o; */ /* weird, this worked but core dumped compiler */
    /* auto Ac 0o4; */ /* Not implemented, should not work with -hist */
    auto Ad 04; 
    Ad[01] = 'A';
    /* Ad[02] = 08; */ /* weird error */
    Ad[03] = 07;
    0;
}
