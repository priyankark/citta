#!/usr/bin/env python3
"""Adapt an operator-configured noninteractive harness that reads a prompt on stdin."""
import json
import os
from pathlib import Path
import subprocess
import sys

request = json.load(sys.stdin)
if not isinstance(request, dict) or set(request) != {'prompt'} or not isinstance(request['prompt'], str):
    raise ValueError('input must be an object with one string field: prompt')
program = Path(os.environ['CITTA_AGENT_PROGRAM'])
if not program.is_absolute():
    raise ValueError('CITTA_AGENT_PROGRAM must be absolute')
args = json.loads(os.environ.get('CITTA_AGENT_ARGS', '[]'))
if not isinstance(args, list) or any(not isinstance(v, str) for v in args):
    raise ValueError('CITTA_AGENT_ARGS must be a JSON array of strings')
# Arguments come from operator configuration, never from the submitted task.
result = subprocess.run([str(program), *args], input=request['prompt'].encode(), check=False)
sys.exit(result.returncode if result.returncode >= 0 else 1)
