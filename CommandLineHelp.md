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
* [`nbwipers show-config`↴](#nbwipers-show-config)
* [`nbwipers record`↴](#nbwipers-record)
* [`nbwipers hook`↴](#nbwipers-hook)
* [`nbwipers hook check-large-files`↴](#nbwipers-hook-check-large-files)

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
* `show-config` — Show configuration
* `record` — Record Kernelspec metadata for notebooks
* `hook` — Commands for pre-commit hooks

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
* `-c`, `--config <CONFIG>` — path to pyproject.toml/.nbwipers.toml/nbwipers.toml file containing nbwipers settings. If not given use the file in the current working directory or the first such file in its containing folders
* `--isolated` — Ignore all configuration files
* `--allow-no-notebooks` — Do not return an error if no notebooks are found
* `--extra-keys <EXTRA_KEYS>` — extra keys to remove in the notebook or cell metadata, separated by commas. Must start with `metadata` or `cell.metadata`
* `--drop-empty-cells` — drop empty cells. Disable with `--keep-empty-cells`
* `--keep-output` — keep cell output. Disable with `--drop-output`
* `--keep-count` — keep cell execution count. Disable with `--drop count`
* `--drop-id` — replace cell ids with sequential ids. Disable with `--keep-id`
* `--strip-init-cell` — Strip init cell. Disable with `--keep-init-cell`
* `--strip-kernel-info` — Strip kernel info. Namely, metadata.kernelspec and metadata.language_info.python_version. Disable with `--keep-kernel-info`
* `--drop-tagged-cells <DROP_TAGGED_CELLS>` — comma-separated list of tags that will cause the cell to be dropped
* `--keep-keys <KEEP_KEYS>` — List of metadata keys that should be kept, regardless of if they appear in
* `--exclude <EXCLUDE>` — List of file patterns to ignore
* `--extend-exclude <EXTEND_EXCLUDE>` — List of additional file patterns to ignore

## `nbwipers check`

check notebooks in a given path for elements that would be removed by `clean`

**Usage:** `nbwipers check [OPTIONS] [FILES]...`

###### **Arguments:**

* `<FILES>` — paths containing ipynb files to check. Use `-` to read from stdin

###### **Options:**

* `-o`, `--output-format <OUTPUT_FORMAT>` — desired output format for diagnostics

  Possible values: `text`, `json`

* `-c`, `--config <CONFIG>` — path to pyproject.toml/.nbwipers.toml/nbwipers.toml file containing nbwipers settings. If not given use the file in the current working directory or the first such file in its containing folders
* `--isolated` — Ignore all configuration files
* `--allow-no-notebooks` — Do not return an error if no notebooks are found
* `--extra-keys <EXTRA_KEYS>` — extra keys to remove in the notebook or cell metadata, separated by commas. Must start with `metadata` or `cell.metadata`
* `--drop-empty-cells` — drop empty cells. Disable with `--keep-empty-cells`
* `--keep-output` — keep cell output. Disable with `--drop-output`
* `--keep-count` — keep cell execution count. Disable with `--drop count`
* `--drop-id` — replace cell ids with sequential ids. Disable with `--keep-id`
* `--strip-init-cell` — Strip init cell. Disable with `--keep-init-cell`
* `--strip-kernel-info` — Strip kernel info. Namely, metadata.kernelspec and metadata.language_info.python_version. Disable with `--keep-kernel-info`
* `--drop-tagged-cells <DROP_TAGGED_CELLS>` — comma-separated list of tags that will cause the cell to be dropped
* `--keep-keys <KEEP_KEYS>` — List of metadata keys that should be kept, regardless of if they appear in
* `--exclude <EXCLUDE>` — List of file patterns to ignore
* `--extend-exclude <EXTEND_EXCLUDE>` — List of additional file patterns to ignore

## `nbwipers clean`

clean a single notebook

**Usage:** `nbwipers clean [OPTIONS] <FILE>`

###### **Arguments:**

* `<FILE>` — path to ipynb file to clean. Use `-` to read from stdin and write to stdout

###### **Options:**

* `-t`, `--textconv` — write cleaned file to stdout instead of to the file
* `-c`, `--config <CONFIG>` — path to pyproject.toml/.nbwipers.toml/nbwipers.toml file containing nbwipers settings. If not given use the file in the current working directory or the first such file in its containing folders
* `--isolated` — Ignore all configuration files
* `--allow-no-notebooks` — Do not return an error if no notebooks are found
* `--extra-keys <EXTRA_KEYS>` — extra keys to remove in the notebook or cell metadata, separated by commas. Must start with `metadata` or `cell.metadata`
* `--drop-empty-cells` — drop empty cells. Disable with `--keep-empty-cells`
* `--keep-output` — keep cell output. Disable with `--drop-output`
* `--keep-count` — keep cell execution count. Disable with `--drop count`
* `--drop-id` — replace cell ids with sequential ids. Disable with `--keep-id`
* `--strip-init-cell` — Strip init cell. Disable with `--keep-init-cell`
* `--strip-kernel-info` — Strip kernel info. Namely, metadata.kernelspec and metadata.language_info.python_version. Disable with `--keep-kernel-info`
* `--drop-tagged-cells <DROP_TAGGED_CELLS>` — comma-separated list of tags that will cause the cell to be dropped
* `--keep-keys <KEEP_KEYS>` — List of metadata keys that should be kept, regardless of if they appear in
* `--exclude <EXCLUDE>` — List of file patterns to ignore
* `--extend-exclude <EXTEND_EXCLUDE>` — List of additional file patterns to ignore

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

## `nbwipers show-config`

Show configuration

**Usage:** `nbwipers show-config [OPTIONS]`

###### **Options:**

* `--show-all` — Show all config including defaults Disable with `--no-show-defaults`
* `-c`, `--config <CONFIG>` — path to pyproject.toml/.nbwipers.toml/nbwipers.toml file containing nbwipers settings. If not given use the file in the current working directory or the first such file in its containing folders
* `--isolated` — Ignore all configuration files
* `--allow-no-notebooks` — Do not return an error if no notebooks are found
* `--extra-keys <EXTRA_KEYS>` — extra keys to remove in the notebook or cell metadata, separated by commas. Must start with `metadata` or `cell.metadata`
* `--drop-empty-cells` — drop empty cells. Disable with `--keep-empty-cells`
* `--keep-output` — keep cell output. Disable with `--drop-output`
* `--keep-count` — keep cell execution count. Disable with `--drop count`
* `--drop-id` — replace cell ids with sequential ids. Disable with `--keep-id`
* `--strip-init-cell` — Strip init cell. Disable with `--keep-init-cell`
* `--strip-kernel-info` — Strip kernel info. Namely, metadata.kernelspec and metadata.language_info.python_version. Disable with `--keep-kernel-info`
* `--drop-tagged-cells <DROP_TAGGED_CELLS>` — comma-separated list of tags that will cause the cell to be dropped
* `--keep-keys <KEEP_KEYS>` — List of metadata keys that should be kept, regardless of if they appear in
* `--exclude <EXCLUDE>` — List of file patterns to ignore
* `--extend-exclude <EXTEND_EXCLUDE>` — List of additional file patterns to ignore

## `nbwipers record`

Record Kernelspec metadata for notebooks

**Usage:** `nbwipers record [OPTIONS] [PATH]`

###### **Arguments:**

* `<PATH>`

###### **Options:**

* `--remove <REMOVE>`
* `--clear`
* `--sync`
* `-c`, `--config <CONFIG>` — path to pyproject.toml/.nbwipers.toml/nbwipers.toml file containing nbwipers settings. If not given use the file in the current working directory or the first such file in its containing folders
* `--isolated` — Ignore all configuration files
* `--allow-no-notebooks` — Do not return an error if no notebooks are found
* `--extra-keys <EXTRA_KEYS>` — extra keys to remove in the notebook or cell metadata, separated by commas. Must start with `metadata` or `cell.metadata`
* `--drop-empty-cells` — drop empty cells. Disable with `--keep-empty-cells`
* `--keep-output` — keep cell output. Disable with `--drop-output`
* `--keep-count` — keep cell execution count. Disable with `--drop count`
* `--drop-id` — replace cell ids with sequential ids. Disable with `--keep-id`
* `--strip-init-cell` — Strip init cell. Disable with `--keep-init-cell`
* `--strip-kernel-info` — Strip kernel info. Namely, metadata.kernelspec and metadata.language_info.python_version. Disable with `--keep-kernel-info`
* `--drop-tagged-cells <DROP_TAGGED_CELLS>` — comma-separated list of tags that will cause the cell to be dropped
* `--keep-keys <KEEP_KEYS>` — List of metadata keys that should be kept, regardless of if they appear in
* `--exclude <EXCLUDE>` — List of file patterns to ignore
* `--extend-exclude <EXTEND_EXCLUDE>` — List of additional file patterns to ignore

## `nbwipers hook`

Commands for pre-commit hooks

**Usage:** `nbwipers hook <COMMAND>`

###### **Subcommands:**

* `check-large-files` — Check for large files, but measure ipynb sizes after clearning

## `nbwipers hook check-large-files`

Check for large files, but measure ipynb sizes after clearning

**Usage:** `nbwipers hook check-large-files [OPTIONS] [FILENAMES]...`

###### **Arguments:**

* `<FILENAMES>` — Files to check for large files

###### **Options:**

* `--enforce-all` — Check all files not just staged files
* `--maxkb <MAXKB>` — Max size in KB to consider a file large
* `-c`, `--config <CONFIG>` — path to pyproject.toml/.nbwipers.toml/nbwipers.toml file containing nbwipers settings. If not given use the file in the current working directory or the first such file in its containing folders
* `--isolated` — Ignore all configuration files

<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
