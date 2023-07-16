use trycmd;

#[test]
fn cli_tests() {
    trycmd::TestCases::new().case("docs/man/cli.md");
}
