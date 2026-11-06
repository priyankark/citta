#!/usr/bin/env python3
"""A fixed local model capability; install/choose the model separately."""
import json
import os
import sys
import urllib.request

request = json.load(sys.stdin)
if not isinstance(request, dict) or set(request) != {'prompt'} or not isinstance(request['prompt'], str):
    raise ValueError('input must be an object with one string field: prompt')
model = os.environ['CITTA_OLLAMA_MODEL']
max_tokens = int(os.environ.get('CITTA_OLLAMA_MAX_TOKENS', '1024'))
if not 1 <= max_tokens <= 8192:
    raise ValueError('operator token budget must be 1..8192')
payload = {'model':model, 'prompt':request['prompt'], 'stream':False, 'options':{'num_predict':max_tokens}}
req = urllib.request.Request('http://127.0.0.1:11434/api/generate', data=json.dumps(payload).encode(), headers={'Content-Type':'application/json'})
# Ignore proxy environment; the model endpoint is always local to this device.
opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
with opener.open(req, timeout=300) as response:
    raw = response.read(262145)
if len(raw) > 262144:
    raise ValueError('model response exceeds 256 KiB')
result = json.loads(raw)
print(json.dumps({'model':model, 'response':result['response'], 'eval_count':result.get('eval_count')}))
