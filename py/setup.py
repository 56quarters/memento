# !/usr/bin/env python
# -*- coding: utf-8 -*-

from setuptools import setup


def build_native(spec):
    build = spec.add_external_build(
        cmd=['cargo', 'build', '--release'],
        path='../cabi'
    )

    spec.add_cffi_module(
        module_path='memento._native',
        dylib=lambda: build.find_dylib('memento', in_path='target/release'),
        header_filename=lambda: build.find_header('memento.h', in_path='include')
    )


setup(
    name='memento',
    version='0.1.0',
    packages=['memento'],
    zip_safe=False,
    platforms='any',
    setup_requires=['milksnake'],
    install_requires=['milksnake'],
    milksnake_tasks=[
        build_native
    ]
)
