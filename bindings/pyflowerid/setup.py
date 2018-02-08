#!/usr/bin/env python3

from setuptools import setup
from setuptools.extension import Extension
from Cython.Build import cythonize
import setuptools.command.build_py
import subprocess
import os
import locale

c_libs_win = ['Ws2_32', 'Userenv', 'Advapi32', 'Shell32']
c_libs_nix = ['pthread', 'dl']
extensions = [
    Extension("pyflowerid", ["pyflowerid.pyx"],
        include_dirs = ['../flowerid_c/include'],
        libraries = ['flowerid_c'] + (c_libs_win if os.name == 'nt' else c_libs_nix),
        library_dirs = ['target/release'])
]

def build_rust():
    cargo_target = os.path.join(os.getcwd(), 'target')
    os.putenv('CARGO_TARGET_DIR', cargo_target)
    os.makedirs(cargo_target, exist_ok=True)
    proc = subprocess.Popen( \
        ['cargo', 'build', '--release'], \
        cwd='../flowerid_c' \
        )
    proc.wait()
    assert proc.returncode == 0
    pass

class BuildRust(setuptools.command.build_ext.build_ext):
  def run(self):
    build_rust()
    super().run()

setup(
    name = "pyflowerid",
    version="0.1.0",
    author="Andrei V",
    author_email="anderi@ptaxa.net",
    description="Flower ID python binding",
    license="all right reserved",
    keywords="identificator id",
    url="https://github.com/PtaxLaine/flowerid",
    ext_modules = cythonize(extensions),
    setup_requires=['cython'],
    cmdclass={
        'build_ext': BuildRust,
    },
    test_suite="test_all"
)
