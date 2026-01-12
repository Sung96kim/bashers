import importlib.resources
import subprocess
import sys
from pathlib import Path


def _create_script_runner(script_name: str):
    def runner() -> int:
        try:
            with importlib.resources.path("bashers", script_name) as script_path:
                if script_path.read_text().strip().startswith("#!"):
                    content = script_path.read_text()
                    if f"{script_name}()" in content:
                        result = subprocess.run(
                            [
                                "bash",
                                "-c",
                                f'source {script_path} && {script_name} "$@"',
                                "--",
                            ]
                            + sys.argv[1:],
                        )
                    else:
                        result = subprocess.run([str(script_path)] + sys.argv[1:])
                    return result.returncode
        except Exception:
            script_path = Path(__file__).parent / script_name
            if script_path.exists():
                content = script_path.read_text()
                if f"{script_name}()" in content:
                    result = subprocess.run(
                        [
                            "bash",
                            "-c",
                            f'source {script_path} && {script_name} "$@"',
                            "--",
                        ]
                        + sys.argv[1:],
                    )
                else:
                    result = subprocess.run([str(script_path)] + sys.argv[1:])
                return result.returncode
        return 1

    return runner


def _discover_scripts():
    scripts_dir = Path(__file__).parent
    scripts = {}
    for script_file in scripts_dir.glob("*"):
        if (
            script_file.is_file()
            and not script_file.name.endswith((".py", ".pyc"))
            and not script_file.name.startswith("__")
        ):
            script_name = script_file.name
            if script_file.read_text().strip().startswith("#!"):
                scripts[script_name] = _create_script_runner(script_name)
    return scripts


_scripts = _discover_scripts()
for name, runner in _scripts.items():
    globals()[name] = runner
