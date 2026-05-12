default:
  @just --list

# Run tests without coverage
test:
    cargo nextest

# Run tests with coverage via llvm-cov
test-cov:
    cargo llvm-cov nextest --all-features --workspace --lcov --output-path lcov.info
    cargo llvm-cov report

# Regenerate the documentation file for the CLI
cli-docs:
    cargo run -q --features=markdown-help -- --markdown-help check | sed -z -e 's/\n\n *Possible values: `true`, `false`\n//g'  > CommandLineHelp.md
    -rumdl check CommandLineHelp.md --fix

# run the tests fr fr on god (run tests but allow dbg! and println! to display output)
test-nocap:
    cargo nextest --no-capture
