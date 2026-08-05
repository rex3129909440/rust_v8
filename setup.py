from __future__ import annotations

import os
import sysconfig
import tomllib
from pathlib import Path

from setuptools import Distribution, setup
from setuptools.command.bdist_wheel import bdist_wheel as _bdist_wheel
from setuptools.command.build_py import build_py as _build_py


ROOT = Path(__file__).resolve().parent
PACKAGE = ROOT / "examples"
NATIVE = PACKAGE / "_native"
METADATA = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["package"]
PYPI_DISTRIBUTION = "rexisohe-sandbox"


class BinaryDistribution(Distribution):
    """Mark a ctypes package as platform-specific without tying it to CPython ABI."""

    def has_ext_modules(self) -> bool:
        return True


class BuildPythonWithNativeLibrary(_build_py):
    _PACKAGE_MODULES = {
        "__init__",
        "edge_profile",
        "edge_runtime_options",
        "edge_sandbox_pool",
        "mac_edge_profile",
        "run_sandbox",
    }

    def find_package_modules(self, package: str, package_dir: str):
        modules = super().find_package_modules(package, package_dir)
        return [module for module in modules if module[1] in self._PACKAGE_MODULES]

    def run(self) -> None:
        libraries = tuple(NATIVE.glob("edge_sandbox.dll"))
        libraries += tuple(NATIVE.glob("libedge_sandbox.so"))
        libraries += tuple(NATIVE.glob("libedge_sandbox.dylib"))
        if len(libraries) != 1:
            raise RuntimeError(
                "exactly one staged native library is required in examples/_native"
            )
        build_package = Path(self.build_lib) / "edge_sandbox"
        if build_package.is_dir():
            for module in build_package.glob("*.py"):
                if module.stem not in self._PACKAGE_MODULES:
                    module.unlink()
        super().run()


class PlatformWheel(_bdist_wheel):
    """Emit a py3-none-platform wheel for the stable ctypes C ABI."""

    def finalize_options(self) -> None:
        super().finalize_options()
        self.root_is_pure = False

    def get_tag(self) -> tuple[str, str, str]:
        platform_tag = os.environ.get("EDGE_SANDBOX_WHEEL_PLATFORM")
        if not platform_tag:
            platform_tag = sysconfig.get_platform().replace("-", "_").replace(".", "_")
        return "py3", "none", platform_tag


setup(
    name=PYPI_DISTRIBUTION,
    version=METADATA["version"],
    description=METADATA["description"],
    long_description=(ROOT / "README.md").read_text(encoding="utf-8"),
    long_description_content_type="text/markdown",
    license=METADATA["license"],
    url="https://github.com/heshengqing/rust_v8",
    project_urls={
        "Source": "https://github.com/heshengqing/rust_v8",
        "Issues": "https://github.com/heshengqing/rust_v8/issues",
    },
    python_requires=">=3.11",
    packages=["edge_sandbox"],
    package_dir={"edge_sandbox": "examples"},
    package_data={
        "edge_sandbox": [
            "_native/edge_sandbox.dll",
            "_native/libedge_sandbox.so",
            "_native/libedge_sandbox.dylib",
        ]
    },
    include_package_data=False,
    zip_safe=False,
    distclass=BinaryDistribution,
    cmdclass={"bdist_wheel": PlatformWheel, "build_py": BuildPythonWithNativeLibrary},
)
