default:
  just --list

test:
    cargo test
test-cov:
    cargo tarpaulin --verbose --all-features --workspace -o stdout -o html -o lcov --engine llvm --rustflags="-C opt-level=0"
cli-docs:
    cargo run -q --features=markdown-help -- --markdown-help check | sed -z -e 's/\n\n *Possible values: `true`, `false`\n//g'  > CommandLineHelp.md
    -markdownlint-cli2 CommandLineHelp.md --fix
test-nocap:
    cargo test -- --nocapture
