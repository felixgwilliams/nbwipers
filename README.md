# nbwipers

![Test](https://github.com/felixgwilliams/nbwipers/actions/workflows/testing.yml/badge.svg)
[![License:MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![PyPI - Version](https://img.shields.io/pypi/v/nbwipers)](https://pypi.org/project/nbwipers/)
[![Crates.io](https://img.shields.io/crates/v/nbwipers)](https://crates.io/crates/nbwipers)
[![Conda](https://img.shields.io/conda/v/conda-forge/nbwipers)](https://anaconda.org/conda-forge/nbwipers)
[![codecov](https://codecov.io/gh/felixgwilliams/nbwipers/graph/badge.svg?token=PLGJFNRHSQ)](https://codecov.io/gh/felixgwilliams/nbwipers)

nbwipers is a command line tool to wipe clean jupyter notebooks, written in Rust.

The interface and functionality are based on [nbstripout](https://github.com/kynan/nbstripout) and the idea to implement it in rust comes from [nbstripout-fast](https://github.com/deshaw/nbstripout-fast).

## Usage

nbwipers has a few subcommands that provide functionality related to cleaning Jupyter notebooks.

- `clean`: clean a single notebook. This is more-or-less equivalent to `nbstripout`.
- `check`: check notebooks in a given path for elements that would be removed by `clean`. This could be used in a CI context to enforce clean notebooks.
- `clean-all` clean all notebooks in a given path. This one should be used carefully!
- `install` register nbwipers as a git filter for `ipynb` files. Equivalent to `nbstripout --install`
- `uninstall` remove nbwipers as a git filter.
- `check-install` check that `nbwipers` or `nbstripout` is installed in the local repo. This is used in the pre-commit hook.
- `show-config` show the effective configuration nbwipers would use, merging the config file with any CLI overrides.
- `record` record kernel metadata for notebooks in a local, git-untracked store, so it can be restored later even though `strip-kernel-info` removes it from committed notebooks. See [Preserving kernel info locally](#preserving-kernel-info-locally) below.
- `hook` subcommands used by pre-commit-style hooks &mdash; currently `check-large-files`, which checks notebook file sizes after cleaning.

The full options can be found in [`CommandLineHelp.md`](CommandLineHelp.md).

### Examples

To set up nbwipers as a git filter in your repository, use

```shell
nbwipers install local
```

If this step is performed on a pre-existing repo, you can `touch` your notebooks so that git can detect the changes.
In bash:

```bash
for f in $(git ls-files '*.ipynb'); do touch $f; done
```

To check the notebooks in your folder, you can run the following

```shell
nbwipers check .
```

To see the configuration nbwipers would use in the current directory, you can run

```shell
nbwipers show-config
```

Add `--show-all` to also see the default values for settings that have not been explicitly configured.

### Preserving kernel info locally

`nbwipers install` sets up both a clean filter, which strips notebooks before they are committed, and a smudge filter, which runs when notebooks are checked out.

If you enable `strip-kernel-info` (see [Configuration](#configuration)) so that kernelspec and python-version metadata never gets committed, you can still keep that information around locally with `record`:

```shell
nbwipers record .
```

This saves the kernel metadata for notebooks under the given path to `.git/x-nbwipers/kernelspec_store.json` &mdash; local to your clone and never committed. The next time you check out one of those notebooks, the smudge filter automatically restores its recorded kernel metadata, so each collaborator keeps their own kernel/python version info without it living in version control.

To keep the local store tidy as notebooks come and go:

- `nbwipers record --sync .` discards the whole store and rebuilds it from the notebooks currently found under `.`, dropping entries for notebooks that no longer exist.
- `nbwipers record --remove path/to/notebook.ipynb` removes a specific notebook's entry, leaving the rest of the store untouched.
- `nbwipers record --clear` wipes the store entirely.

### pre-commit

You can add the following to your `pre-commit-config.yaml` file to ensure that `nbwipers` or `nbstripout` is installed in your repo, in order to prevent Jupyter notebook outputs from being committed to version control.

```yaml
  - repo: https://github.com/felixgwilliams/nbwipers-pre-commit
    rev: v0.6.2
    hooks:
      - id: nbwipers-check-install
```

Alternatively, you can use the URL for this repo in your config, but this will compile `nbwipers` from source, rather than retrieving the binary from PyPI, and is therefore not recommended.

If you are using your pre-commit configuration as part of CI, you should set the environment variable `NBWIPERS_CHECK_INSTALL_EXIT_ZERO` which forces this check to pass, since you do not need `nbwipers` configured in your CI environment.

## Configuration

Configuration is currently done via the `tool.nbwipers` section of the `pyproject.toml` file.
Most of the command line options can be set per-project in the `pyproject.toml`, `nbwipers.toml` or `.nbwipers.toml` file.
If you use `pyroject.toml`, you need to put the configuration under `[tool.nbwipers]`.
If you use `nbwipers.toml` or `.nbwipers.toml`, the configuration needs to be at the top level.

For example you can use `strip-kernel-info` to remove metadata on the python version or the details about the Jupyter Kernel.

You can also drop cell ids using `id-action = "drop"`.

To enable these options, you can include the following in your `pyproject.toml` file:

```toml
[tool.nbwipers]
strip-kernel-info = true
id-action = "drop"
```

The equivalent for `nbwipers.toml` or `.nbwipers.toml` is just

```toml
strip-kernel-info = true
id-action = "drop"
```

This can be useful when collaborating, as the precise python version and the name assigned to the kernel are ephemeral and can change from person to person.
Cell IDs are another element of the file which is generated by the tool you use and can change from person to person.

## Motivation

A working copy of a Jupyter notebook contains:

1. Code written by the author.
2. Notebook outputs: tables, logs, tracebacks, images, widgets and so on...
3. Execution counts.
4. Metadata, such as whether cells are collapsed, scrollable etc.

Of these categories of data, only the first &mdash; code written by the author &mdash; should definitely be tracked by version control, since it is the product of the author's intention and hard work.
The other categories of data are subject to change outside of the explicit intentions of the author and are generally noisy from a version control perspective.

Moreover, including notebook outputs in version control

- makes diffs harder to interpret, as they will contain lots of unintended changes.
- increases the risk of a tricky merge conflict if different users run the same cell and get a slightly different result.
- increases the amount of data committed, which can degrade repository performance.
- risks leaking sensitive data.

An effective way to ensure you do not commit problematic parts of your notebooks is to use `nbwipers` or `nbstripout` as a git filter.

A git filter sits between your actual files and what git sees when you stage and commit your changes.
This way, git only sees the transformed version of the file without the problematic elements.
At the same time, you do not have to lose them from your local copy.

An exception is when you checkout a branch or do a git pull, which results in changes to the notebook.
In this case, your local copy will be replaced by the clean version and you will lose your cell outputs.

## Acknowledgements

nbwipers relies on inspiration and code from several projects.
For the projects, whose code was used please see [`LICENSE`](LICENSE) for the third-party notices.

### [nbstripout](https://github.com/kynan/nbstripout)

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

### [pre-commit](https://github.com/pre-commit/pre-commit)

> A framework for managing and maintaining multi-language pre-commit hooks.

This repo contains a version of the check-large-files hook, that will not flag notebook files whose clean size is less that the threshold, even if the size on-disk including outputs is greater than the threshold.
The logic and interface of the hook was adapted from the [pre-commit-hooks](https://github.com/pre-commit/pre-commit-hooks) repository.
