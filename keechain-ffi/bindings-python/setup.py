#!/usr/bin/env python

from setuptools import setup

from pathlib import Path
this_directory = Path(__file__).parent
long_description = (this_directory / "README.md").read_text()

setup(
    name='keechain',
    version='0.0.1',
    description="Keechain Core",
    long_description=long_description,
    long_description_content_type='text/markdown',
    include_package_data = True,
    zip_safe=False,
    packages=['keechain'],
    package_dir={'keechain': './src/keechain'},
    url="https://github.com/yukibtc/keechain",
    author="Yuki Kishimoto <yukikishimoto@protonmail.com>",
    license="MIT",
     # This is required to ensure the library name includes the python version, abi, and platform tags
    # See issue #350 for more information
    has_ext_modules=lambda: True,
)
