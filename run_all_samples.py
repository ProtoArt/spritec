#!/usr/bin/env python3

import shlex
import subprocess

from os.path import join, exists
from glob import glob

for path in glob("samples/*"):
    path = join(path, "spritec.toml")
    if exists(path):
        command = ["cargo", "run", "--", shlex.quote(path)]
        print(' '.join(command))
        subprocess.run(command, check=True)
