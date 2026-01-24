use trycmd;

#[test]
fn cli_tests() {
    trycmd::TestCases::new().case("assets/cli.md");
}
