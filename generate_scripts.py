from pathlib import Path
import sys


def generate_script_entries():
    scripts_dir = Path("bashers")
    if not scripts_dir.exists():
        return {}

    entries = {}
    for script_file in scripts_dir.glob("*"):
        if (
            script_file.is_file()
            and not script_file.name.endswith((".py", ".pyc"))
            and not script_file.name.startswith("__")
        ):
            script_name = script_file.name
            content = script_file.read_text()
            if content.strip().startswith("#!"):
                entries[script_name] = f"bashers:{script_name}"

    return entries


if __name__ == "__main__":
    entries = generate_script_entries()
    if entries:
        print("# Add these to [project.scripts] in pyproject.toml:")
        for name, entry in sorted(entries.items()):
            print(f'{name} = "{entry}"')
    else:
        print("No bash scripts found in bashers/ directory", file=sys.stderr)
        sys.exit(1)
