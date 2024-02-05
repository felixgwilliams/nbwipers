# Command-Line Help for `nbwipers`

This document contains the help content for the `nbwipers` command-line program.

**Command Overview:**

* [`nbwipers`↴](#nbwipers)
* [`nbwipers install`↴](#nbwipers-install)
* [`nbwipers clean-all`↴](#nbwipers-clean-all)
* [`nbwipers check`↴](#nbwipers-check)
* [`nbwipers clean`↴](#nbwipers-clean)
* [`nbwipers uninstall`↴](#nbwipers-uninstall)
* [`nbwipers check-install`↴](#nbwipers-check-install)

## `nbwipers`

Wipe clean your Jupyter Notebooks!

**Usage:** `nbwipers <COMMAND>`

###### **Subcommands:**

* `install` — Register nbwipers as a git filter for `ipynb` files
* `clean-all` — clean all notebooks in a given path
* `check` — check notebooks in a given path for elements that would be removed by `clean`
* `clean` — clean a single notebook
* `uninstall` — uninstall nbwipers as a git filter
* `check-install` — check whether nbwipers is setup as a git filter

###### **Options:**

* `--markdown-help`

## `nbwipers install`

Register nbwipers as a git filter for `ipynb` files

**Usage:** `nbwipers install [OPTIONS] <CONFIG_TYPE>`

###### **Arguments:**

* `<CONFIG_TYPE>` — Git config type that determines which file to modify

  Possible values:
  * `system`:
    System-wide git config
  * `global`:
    User level git config, typically corresponding to ~/.gitconfig
  * `local`:
    Repository level git config, corresponding to .git/config

###### **Options:**

* `-g`, `--git-config-file <GIT_CONFIG_FILE>` — Optional path to git config file
* `-a`, `--attribute-file <ATTRIBUTE_FILE>` — optional attribute file. If not specified, will write to .git/info/attributes

## `nbwipers clean-all`

clean all notebooks in a given path

**Usage:** `nbwipers clean-all [OPTIONS] [FILES]...`

###### **Arguments:**

* `<FILES>` — paths containing ipynb files to clean. Stdin is not supported

###### **Options:**

* `-d`, `--dry-run` — set to true to avoid writing to files
* `-y`, `--yes` — skip confirmation and assume yes
* `-c`, `--config <CONFIG>` — path to pyproject.toml file containing nbwipers settings. If not given use the file in the current working directory or the first such file in its containing folders
* `--allow-no-notebooks` — Do not return an error if no notebooks are found
* `--extra-keys <EXTRA_KEYS>` — extra keys to remove in the notebook or cell metadata, separated by commas. Must start with `metadata` or `cell.metadata`
* `--drop-empty-cells` — drop empty cells. Disable with `--keep-empty-cells`
* `--keep-empty-cells`
* `--keep-output` — keep cell output. Disable with `--drop-output`
* `--drop-output`
* `--keep-count` — keep cell execution count. Disable with `--drop count`
* `--drop-count`
* `--drop-id` — replace cell ids with sequential ids. Disable with `--keep-id`
* `--keep-id`
* `--strip-init-cell` — Strip init cell. Disable with `--keep-init-cell`
* `--keep-init-cell`
* `--drop-tagged-cells <DROP_TAGGED_CELLS>` — comma-separated list of tags that will cause the cell to be dropped
* `--keep-keys <KEEP_KEYS>` — List of metadata keys that should be kept, regardless of if they appear in

## `nbwipers check`

check notebooks in a given path for elements that would be removed by `clean`

**Usage:** `nbwipers check [OPTIONS] [FILES]...`

###### **Arguments:**

* `<FILES>` — paths containing ipynb files to check. Use `-` to read from stdin

###### **Options:**

* `-o`, `--output-format <OUTPUT_FORMAT>` — desired output format for diagnostics

  Possible values: `text`, `json`

* `-c`, `--config <CONFIG>` — path to pyproject.toml file containing nbwipers settings. If not given use the file in the current working directory or the first such file in its containing folders
* `--allow-no-notebooks` — Do not return an error if no notebooks are found
* `--extra-keys <EXTRA_KEYS>` — extra keys to remove in the notebook or cell metadata, separated by commas. Must start with `metadata` or `cell.metadata`
* `--drop-empty-cells` — drop empty cells. Disable with `--keep-empty-cells`
* `--keep-empty-cells`
* `--keep-output` — keep cell output. Disable with `--drop-output`
* `--drop-output`
* `--keep-count` — keep cell execution count. Disable with `--drop count`
* `--drop-count`
* `--drop-id` — replace cell ids with sequential ids. Disable with `--keep-id`
* `--keep-id`
* `--strip-init-cell` — Strip init cell. Disable with `--keep-init-cell`
* `--keep-init-cell`
* `--drop-tagged-cells <DROP_TAGGED_CELLS>` — comma-separated list of tags that will cause the cell to be dropped
* `--keep-keys <KEEP_KEYS>` — List of metadata keys that should be kept, regardless of if they appear in

## `nbwipers clean`

clean a single notebook

**Usage:** `nbwipers clean [OPTIONS] <FILE>`

###### **Arguments:**

* `<FILE>` — path to ipynb file to clean. Use `-` to read from stdin and write to stdout

###### **Options:**

* `-t`, `--textconv` — write cleaned file to stdout instead of to the file
* `-c`, `--config <CONFIG>` — path to pyproject.toml file containing nbwipers settings. If not given use the file in the current working directory or the first such file in its containing folders
* `--allow-no-notebooks` — Do not return an error if no notebooks are found
* `--extra-keys <EXTRA_KEYS>` — extra keys to remove in the notebook or cell metadata, separated by commas. Must start with `metadata` or `cell.metadata`
* `--drop-empty-cells` — drop empty cells. Disable with `--keep-empty-cells`
* `--keep-empty-cells`
* `--keep-output` — keep cell output. Disable with `--drop-output`
* `--drop-output`
* `--keep-count` — keep cell execution count. Disable with `--drop count`
* `--drop-count`
* `--drop-id` — replace cell ids with sequential ids. Disable with `--keep-id`
* `--keep-id`
* `--strip-init-cell` — Strip init cell. Disable with `--keep-init-cell`
* `--keep-init-cell`
* `--drop-tagged-cells <DROP_TAGGED_CELLS>` — comma-separated list of tags that will cause the cell to be dropped
* `--keep-keys <KEEP_KEYS>` — List of metadata keys that should be kept, regardless of if they appear in

## `nbwipers uninstall`

uninstall nbwipers as a git filter

**Usage:** `nbwipers uninstall [OPTIONS] <CONFIG_TYPE>`

###### **Arguments:**

* `<CONFIG_TYPE>` — Git config type that determines which file to modify

  Possible values:
  * `system`:
    System-wide git config
  * `global`:
    User level git config, typically corresponding to ~/.gitconfig
  * `local`:
    Repository level git config, corresponding to .git/config

###### **Options:**

* `-g`, `--git-config-file <GIT_CONFIG_FILE>` — Optional path to git config file
* `-a`, `--attribute-file <ATTRIBUTE_FILE>` — optional attribute file. If not specified, will write to .git/info/attributes

## `nbwipers check-install`

check whether nbwipers is setup as a git filter

**Usage:** `nbwipers check-install [OPTIONS] [CONFIG_TYPE]`

###### **Arguments:**

* `<CONFIG_TYPE>` — Git config type to check

  Possible values:
  * `system`:
    System-wide git config
  * `global`:
    User level git config, typically corresponding to ~/.gitconfig
  * `local`:
    Repository level git config, corresponding to .git/config

###### **Options:**

* `--exit-zero` — Exit zero regardless of install status

<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
