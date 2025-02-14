default:
  @just --list

# Run tests without coverage
test:
    cargo test

# Run tests with coverage via tarpaulin
test-cov:
    cargo tarpaulin --verbose --all-features --workspace -o stdout -o html -o lcov --engine llvm --rustflags="-C opt-level=0"

# Regenerate the documentation file for the CLI
cli-docs:
    cargo run -q --features=markdown-help -- --markdown-help check | sed -z -e 's/\n\n *Possible values: `true`, `false`\n//g'  > CommandLineHelp.md
    -markdownlint-cli2 CommandLineHelp.md --fix

# run the tests fr fr on god (run tests but allow dbg! and println! to display output)
test-nocap:
    cargo test -- --nocapture
