// Testing char() and lchar()
main() {
    extrn char, printf, malloc, strlen, lchar, toupper;
    auto s, q, i, n;
    s = "hello, world";
    printf("lower: %s\n", s);
    q = malloc(n + 1);
    n = strlen(s);
	i = 0; while (i < n) {
        lchar(q, i, toupper(char(s, i)));
        i++;
    }
    lchar(q, n, '*e');
    printf("UPPER: %s\n", q);
}
