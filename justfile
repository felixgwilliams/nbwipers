default:
  just --list

test:
    cargo test
test-cov:
    cargo tarpaulin -o stdout -o html -o lcov --engine llvm
cli-docs:
    cargo run -q --features=markdown-help -- --markdown-help check | sed -z -e 's/\n\n *Possible values: `true`, `false`\n//g'  > CommandLineHelp.md
    -markdownlint-cli2 CommandLineHelp.md --fix
test-nocap:
    cargo test -- --nocapture
