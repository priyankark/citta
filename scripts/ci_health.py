#!/usr/bin/env python3
"""Snapshot Citta CI attempts and retain failure history. Run on demand; no background polling."""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

INFRA_MARKERS = (
    'not acquired by runner', 'lost communication with the server',
    'runner has received a shutdown signal', 'runner failed to start',
    'the job was not started because recent account payments have failed or your spending limit needs to be increased',
)
TERMINAL_BAD = {'failure', 'cancelled', 'timed_out', 'action_required', 'startup_failure'}


def api(endpoint):
    result = subprocess.run(['gh', 'api', endpoint], check=True, capture_output=True, text=True)
    return json.loads(result.stdout)


def pages(endpoint, key, fetch=api):
    items = []
    separator = '&' if '?' in endpoint else '?'
    for page in range(1, 101):
        value = fetch(f'{endpoint}{separator}per_page=100&page={page}')
        batch = value if key is None else value[key]
        items.extend(batch)
        if len(batch) < 100:
            return items
    raise RuntimeError('API pagination exceeded 100 pages; refusing an incomplete snapshot')


def classify_job(job):
    if job['status'] != 'completed':
        return 'pending'
    conclusion = job.get('conclusion')
    if conclusion == 'success':
        return 'success'
    if conclusion in {'neutral', 'skipped'}:
        return conclusion
    messages = ' '.join(a.get('message', '') for a in job.get('annotations', [])).lower()
    if any(marker in messages for marker in INFRA_MARKERS):
        return 'infrastructure'
    if conclusion == 'cancelled':
        return 'cancelled'
    if conclusion == 'timed_out':
        return 'timeout'
    # A failed check without diagnostics is not evidence of a compiler/test bug.
    return 'check_failure' if conclusion in TERMINAL_BAD else 'unknown'


def snapshot_run(repo, run, fetch=api):
    attempt = run['run_attempt']
    raw_jobs = pages(f'repos/{repo}/actions/runs/{run["id"]}/attempts/{attempt}/jobs', 'jobs', fetch)
    jobs = []
    for job in raw_jobs:
        annotations = []
        if job.get('conclusion') in TERMINAL_BAD:
            check_url = job.get('check_run_url')
            prefix = f'https://api.github.com/repos/{repo}/check-runs/'
            if check_url and check_url.startswith(prefix):
                check_id = check_url.removeprefix(prefix)
                if check_id.isdigit():
                    annotations = pages(f'repos/{repo}/check-runs/{check_id}/annotations', None, fetch)
        entry = {
            'id': job['id'], 'name': job['name'], 'status': job['status'],
            'conclusion': job.get('conclusion'), 'url': job.get('html_url'),
            'annotations': [{'level': a.get('annotation_level'), 'message': a.get('message', '')[:4096]} for a in annotations],
            'failed_steps': [s['name'] for s in job.get('steps', []) if s.get('conclusion') in TERMINAL_BAD],
        }
        entry['category'] = classify_job(entry)
        jobs.append(entry)
    return {
        'run_id': run['id'], 'attempt': attempt, 'commit': run['head_sha'],
        'branch': run.get('head_branch'), 'workflow': run['name'], 'url': run['html_url'],
        'status': run['status'], 'conclusion': run.get('conclusion'),
        'created_at': run['created_at'], 'updated_at': run['updated_at'], 'jobs': jobs,
    }


def merge(history, snapshots, observed_at):
    """Reruns append attempt records; a success never replaces an earlier failure."""
    records = {f'{r["run_id"]}:{r["attempt"]}': r for r in history.get('attempts', [])}
    for snapshot in snapshots:
        key = f'{snapshot["run_id"]}:{snapshot["attempt"]}'
        previous = records.get(key, {})
        # Keep immutable evidence if an in-flight request arrives after completion.
        if previous.get('status') == 'completed' and snapshot['status'] != 'completed':
            continue
        records[key] = {**snapshot, 'first_observed_at': previous.get('first_observed_at', observed_at), 'last_observed_at': observed_at}
    return {**history, 'version': 1, 'observed_at': observed_at,
            'attempts': sorted(records.values(), key=lambda r: (r['run_id'], r['attempt']))}


def summarize(history):
    attempts = history.get('attempts', [])
    latest = max(attempts, key=lambda r: (r['run_id'], r['attempt']), default=None)
    failures = []
    for record in attempts:
        if record.get('conclusion') not in TERMINAL_BAD and not any(j['category'] in {'infrastructure','check_failure','timeout','cancelled','unknown'} for j in record['jobs']):
            continue
        recovered = any(r['run_id'] == record['run_id'] and r['attempt'] > record['attempt'] and r.get('conclusion') == 'success' for r in attempts)
        failures.append({'run_id':record['run_id'], 'attempt':record['attempt'], 'commit':record['commit'],
                         'url':record['url'], 'recovered_by_rerun':recovered,
                         'jobs':[{'name':j['name'], 'category':j['category'], 'failed_steps':j['failed_steps']} for j in record['jobs'] if j['category'] not in {'success','skipped','neutral'}]})
    return {'observed_at':history.get('observed_at'), 'latest':latest, 'failure_history':failures}


def save(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary = tempfile.mkstemp(prefix=path.name + '.', dir=path.parent)
    try:
        with os.fdopen(fd, 'w') as output:
            json.dump(data, output, indent=2)
            output.write('\n')
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', default='priyankark/citta')
    parser.add_argument('--workflow', default='ci.yml')
    parser.add_argument('--branch', default='main')
    parser.add_argument('--state', type=Path, default=Path(__file__).resolve().parents[1] / 'local/ci-history.json')
    parser.add_argument('--run', type=int, action='append', help='Also record a specific historical run; repeatable')
    args = parser.parse_args()
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', args.repo):
        parser.error('repo must be owner/name')
    from urllib.parse import quote
    endpoint = f'repos/{args.repo}/actions/workflows/{quote(args.workflow, safe="")}/runs?branch={quote(args.branch, safe="")}&per_page=20'
    runs = {r['id']:r for r in api(endpoint)['workflow_runs']}
    for run_id in args.run or []:
        run = api(f'repos/{args.repo}/actions/runs/{run_id}')
        runs[run['id']] = run
    history = json.loads(args.state.read_text()) if args.state.exists() else {'attempts':[]}
    for field, expected in [('repo',args.repo),('workflow',args.workflow),('branch',args.branch)]:
        if history.get(field, expected) != expected:
            parser.error(f'state belongs to another {field}; choose a separate state file')
    if history.get('version', 1) != 1:
        parser.error('unsupported history version')
    for record in history['attempts']:
        for job in record['jobs']:
            job['category'] = classify_job(job)
    known = {(r['run_id'],r['attempt']):r for r in history['attempts']}
    snapshots = []
    for run in runs.values():
        # Fetch unobserved earlier attempts as well, even if first observed after recovery.
        for attempt in range(1, run['run_attempt'] + 1):
            old = known.get((run['id'], attempt))
            if old and old['status'] == 'completed':
                continue
            version = run if attempt == run['run_attempt'] else api(f'repos/{args.repo}/actions/runs/{run["id"]}/attempts/{attempt}')
            snapshots.append(snapshot_run(args.repo, version))
    observed = datetime.now(timezone.utc).isoformat()
    history = merge({**history, 'repo':args.repo, 'workflow':args.workflow, 'branch':args.branch}, snapshots, observed)
    save(args.state, history)
    print(json.dumps(summarize(history), indent=2))

if __name__ == '__main__':
    main()
