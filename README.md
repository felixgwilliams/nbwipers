# nbwipers

nbwipers is a command line tool to wipe clean jupyter notebooks, written in Rust.

The interface and functionality are based on [nbsripout](https://github.com/kynan/nbstripout) and the idea to implement it in rust comes from [nbstripout-fast](https://github.com/deshaw/nbstripout-fast).

## Usage

nbwipers has a few subcommands that provide functionality related to cleaning Jupyter notebooks.

- `clean`: clean a single notebook. This is more-or-less equivalent to `nbstripout`
- `check`: check notebooks in a given path for elements that would be removed by `clean`. This could be used in a CI context to enforce clean notebooks.
- `clean-all` clean all notebooks in a given path. This one should be used carefully!
- `install` register nbwipers as a git filter for `ipynb` files. Equivalent to `nbstripout --install`
- `uninstall` remove nbwipers as a git filter
- `check-install` check that `nbwipers` or `nbstripout` is installed in the local repo. This is used in the pre-commit hook.

The full options can be found in [`CommandLineHelp.md`](CommandLineHelp.md).

### Examples

To set up nbwipers as a git filter in your repository, use

```shell
nbwipers install local
```

To check the notebooks in your folder

```shell
nbwipers check .
```

### pre-commit

You can add the following to your `pre-commit-config.yaml` file to ensure that `nbwipers` or `nbstripout` is installed in your repo, as a friendly reminder

```yaml
  - repo: https://github.com/felixgwilliams/nbwipers
    rev: v0.3.0
    hooks:
      - id: nbwipers-check-install
```

## Motivation

A working copy of a Jupyter notebook contains

1. Code written by the author
2. Notebook outputs: tables, logs, tracebacks, images, widgets and so on...
3. Execution counts
4. Metadata, such as whether cells are collapsed, scrollable etc.

Of these categories of data, only the first &mdash; code written by the author &mdash; should definitely be tracked by version control, since it is the product of the author's intension and hard work.
The other categories of data are subject to change outside of the explicit intensions of the author, and are generally noisy from a version control perspective.

Moreover, including notebook outputs in version control

- makes diffs harder to interpret, as they will contain lots of unintended changes
- increases the risk of a tricky merge conflict if different users run the same cell and get a slightly different result
- increases the amount of data committed, which can degrade repository performance
- risks leaking sensitive data

By using nbwipers or nbstripout as a git filter, the problematic parts of the notebook are removed from the version of the file that git sees, while leaving your working copy intact.

## Note on pre-commit for Windows users

Using the above pre-commit hook requires compiling nbwipers from source.
If you do not have rust installed on your system, pre-commit can create a temporary rust installation to do the compilation, but this can fail on Windows
if the [prerequisites](https://rust-lang.github.io/rustup/installation/windows-msvc.html) for the MSVC toolchain are not met.

Running the [rustup](https://rustup.rs/) installer will give you an opportunity to automatically install the prerequisites.
Once you do this and get a working rust installation, pre-commit will use that to compile.

An alternative is install rust [rustup](https://rustup.rs/), but customize the installation to specify `x86_64-pc-windows-gnu` as the default host triple.
This way you do not need to install the multi-gigabyte MSVC dependencies.

## Testing Coverage

To test coverage, use the command:

```shell
cargo tarpaulin -o stdout -o html -o lcov --engine llvm
```

Using the `llvm` engine means that integration tests contribute to coverage.

## Acknowledgements

nbwipers relies on inspiration and code from several projects.
For the projects whose code was used please see [`LICENSE`](LICENSE) for the third-party notices.

### [nbsripout](https://github.com/kynan/nbstripout)

> strip output from Jupyter and IPython notebooks

nbstripout is an invaluable tool for working with Jupyter Notebooks in the context of version control.
This project forms the basis of the interface and logic of this project and is also the source of the testing examples.

### [nbstripout-fast](https://github.com/deshaw/nbstripout-fast)

> A much faster version of nbstripout by writing it in rust (of course).

nbstripout-fast, like this project, implements the functionality of nbstripout in Rust, while also allowing repo-level configuration in a YAML file.

With nbwipers I hoped to recreate the idea of nbstripout-fast, but with the ability to install as a git filter, and configuration via `pyproject.toml`.

### [ruff](https://github.com/astral-sh/ruff)

> An extremely fast Python linter and code formatter, written in Rust.

Ruff is quickly becoming *the* linter for python code, thanks to its performance, extensive set of rules and its ease of use.
It was a definite source of knowledge for the organisation of the configuration and the file discovery.
The schema for Jupyter Notebooks, and some of the file discovery code was adapted from Ruff.
