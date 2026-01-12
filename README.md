# Bashers
Installable bash command helpers

## Installation

Install the package:

```bash
pip install .
```

Or with uv:

```bash
uv pip install .
```

Or install from a built wheel:

```bash
pip install dist/bashers-*.whl
```

## Usage

After installation, the commands are immediately available on your PATH:

```bash
update [package]
```

## Development

To install in development mode:

```bash
uv sync
```

Or with pip:

```bash
pip install -e .
```

## Adding New Commands

1. Add your bash script to the `bashers/` directory
2. If you want a bash function, define a function with the same name as the file
3. Reinstall: `uv sync` or `pip install -e .`
