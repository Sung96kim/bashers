# Bashers
Installable bash command helpers

## Installation

Install from PyPI:

```bash
pip install bashers
```

Or with uv:

```bash
uv pip install bashers
```

Local install (from this repo):

```bash
pip install .
```

Or install from a built wheel:

```bash
pip install dist/bashers-*.whl
```

## Usage

After installation, use the `bashers` dispatcher:

```bash
bashers update
bashers update requests
bashers show
bashers show requests
bashers setup
bashers setup --frozen
bashers gh
bashers gh --dry-run
```

Verify the command is on PATH:

```bash
which bashers
```

If you are using a virtualenv or `uv`, make sure its `bin/` directory is active in your shell.

## Shell Completion

Bash completion:

```bash
source <(bashers completion)
```

The completion script auto-installs to `~/.local/share/bash-completion/completions/bashers` on first run.
If you prefer, you can enable it manually (example):

```bash
bashers completion > ~/.bash_completion.d/bashers
```

## Spinner

The default loader uses a built-in `dots13` spinner and has no extra dependencies.
Disable it with `NO_SPINNER=1`.

## Development

To install in development mode:

```bash
uv sync
```

Or with pip:

```bash
pip install -e .
```

Run tests:

```bash
pytest
```

## Adding New Commands

1. Add your bash script under the `bashers/` directory (subfolders are OK)
2. If you want a bash function, define a function with the same name as the file
3. Reinstall: `uv sync` or `pip install -e .`
