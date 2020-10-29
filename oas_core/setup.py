#!/usr/bin/env python3

import setuptools

with open("README.md", "r") as readme:
    long_description = readme.read()

setuptools.setup(
    name='open-audio-search',
    version='0.0.1',
    scripts=['open-audio-search'],
    author="",
    description="",
    install_requires=[
        'Click',
    ],
    entry_points='''
        [console_scripts]
        open-audio-search=open-audio-search:cli
    ''',
    long_description=long_description,
    packages=setuptools.find_packages()
)
