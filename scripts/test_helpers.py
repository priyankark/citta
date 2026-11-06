#!/usr/bin/env python3
"""Behavior checks for service rendering and capability adapters; no live model required."""
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import plistlib
import runpy
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('service', ROOT / 'scripts/render_service.py')
service = importlib.util.module_from_spec(spec)
spec.loader.exec_module(service)

class Helpers(unittest.TestCase):
    def test_launchd_preserves_paths_with_spaces(self):
        output = service.render('macos', Path('/Applications/My Tools/citta'), Path('/Users/test/My Config/citta.toml'), Path('/tmp/logs'))
        plist = plistlib.loads(output)
        self.assertEqual(plist['ProgramArguments'], ['/Applications/My Tools/citta', '--config', '/Users/test/My Config/citta.toml', 'daemon'])
        self.assertEqual(plist['KeepAlive'], {'SuccessfulExit':False})

    def test_systemd_escapes_specifiers_and_variable_expansion(self):
        output = service.render('linux', Path('/opt/100%/$bin/citta'), Path('/tmp/space here/config.toml'), Path('/tmp/logs')).decode()
        self.assertIn('"/opt/100%%/$$bin/citta"', output)
        self.assertIn('"/tmp/space here/config.toml"', output)
        self.assertIn('KillMode=control-group', output)
        with self.assertRaises(ValueError):
            service.systemd_quote('line\nbreak')

    def test_renderer_refuses_to_overwrite_existing_service(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            config = directory / 'config.toml'
            config.write_text('')
            output = directory / 'service.plist'
            command = [sys.executable, str(ROOT/'scripts/render_service.py'), '--platform','macos', '--binary',sys.executable, '--config',str(config), '--output',str(output)]
            subprocess.run(command, check=True, capture_output=True)
            before = output.read_bytes()
            self.assertNotEqual(subprocess.run(command, capture_output=True).returncode, 0)
            self.assertEqual(output.read_bytes(), before)

    def test_prompt_adapter_uses_fixed_argv_and_literal_stdin(self):
        prompt = 'literal prompt; $(touch should-not-exist)'
        env = {'CITTA_AGENT_PROGRAM':sys.executable, 'CITTA_AGENT_ARGS':json.dumps(['-c','import sys; print(sys.stdin.read())'])}
        response = subprocess.run([sys.executable, str(ROOT/'examples/prompt_worker.py')], input=json.dumps({'prompt':prompt}), text=True, capture_output=True, env=env, check=True)
        self.assertEqual(response.stdout.strip(), prompt)
        response = subprocess.run([sys.executable, str(ROOT/'examples/prompt_worker.py')], input=json.dumps({'prompt':prompt,'args':['untrusted']}), text=True, capture_output=True, env=env)
        self.assertNotEqual(response.returncode, 0)

    def test_model_adapter_fixes_endpoint_model_and_token_budget(self):
        response = io.BytesIO(json.dumps({'response':'result','eval_count':3}).encode())
        output = io.StringIO()
        with patch.dict(os.environ, {'CITTA_OLLAMA_MODEL':'installed-model','CITTA_OLLAMA_MAX_TOKENS':'17'}, clear=True), patch('sys.stdin',io.StringIO('{"prompt":"question"}')), patch('urllib.request.build_opener') as opener, contextlib.redirect_stdout(output):
            opener.return_value.open.return_value = response
            runpy.run_path(str(ROOT/'examples/ollama_worker.py'), run_name='__main__')
            request = opener.return_value.open.call_args.args[0]
            self.assertEqual(request.full_url,'http://127.0.0.1:11434/api/generate')
            payload = json.loads(request.data)
            self.assertEqual(payload['model'],'installed-model')
            self.assertEqual(payload['options']['num_predict'],17)
        self.assertEqual(json.loads(output.getvalue())['response'],'result')

if __name__ == '__main__':
    unittest.main(verbosity=2)
