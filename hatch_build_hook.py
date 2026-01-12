from hatchling.builders.hooks.plugin.interface import BuildHookInterface
from pathlib import Path
import re
import tempfile


class CustomBuildHook(BuildHookInterface):
    def initialize(self, version, build_data):
        scripts_dir = Path("bashers")
        if not scripts_dir.exists():
            return

        scripts = {}
        self._temp_dir = Path(tempfile.mkdtemp())

        for script_file in scripts_dir.glob("*"):
            if (
                script_file.is_file()
                and script_file.name != "__main__.py"
                and not script_file.name.endswith((".pyc", ".py"))
            ):
                if script_file.name.startswith("."):
                    continue

                script_content = script_file.read_text()
                script_name = script_file.name

                if re.search(
                    rf"^\s*{re.escape(script_name)}\s*\(\)\s*\{{",
                    script_content,
                    re.MULTILINE,
                ):
                    lines = script_content.splitlines()
                    if lines and lines[0].startswith("#!"):
                        shebang = lines[0]
                        body = "\n".join(lines[1:])
                    else:
                        shebang = "#!/usr/bin/env bash"
                        body = script_content

                    standalone_script = (
                        f"{shebang}\n\n{body}\n\n{script_name} \"$@\"\n"
                    )
                    wrapper_path = self._temp_dir / script_name
                    wrapper_path.write_text(standalone_script)
                    wrapper_path.chmod(0o755)
                    scripts[script_name] = str(wrapper_path)
                else:
                    scripts[script_name] = str(script_file.absolute())

        if scripts:
            build_data["scripts"] = scripts

    def finalize(self, version, build_data):
        temp_dir = getattr(self, "_temp_dir", None)
        if temp_dir and temp_dir.exists():
            for child in temp_dir.iterdir():
                if child.is_file():
                    child.unlink()
            temp_dir.rmdir()
