from pathlib import Path
import os
import subprocess


def _script_path() -> Path:
    return Path(__file__).resolve().parents[1] / "bashers" / "bashers"


def _run(*args: str) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env["BASHERS_DIR"] = str(Path(__file__).resolve().parents[1] / "bashers")
    return subprocess.run(
        [_script_path().as_posix(), *args],
        check=True,
        text=True,
        capture_output=True,
        env=env,
    )


def test_help_lists_commands() -> None:
    result = _run("--help")
    assert "Commands:" in result.stdout
    assert "setup" in result.stdout
    assert "show" in result.stdout
    assert "update" in result.stdout


def test_commands_output() -> None:
    result = _run("_commands")
    commands = {line.strip() for line in result.stdout.splitlines() if line.strip()}
    assert {"setup", "show", "update"}.issubset(commands)


def test_completion_script() -> None:
    result = _run("completion")
    assert "_bashers_complete" in result.stdout
    assert "complete -F _bashers_complete bashers" in result.stdout
