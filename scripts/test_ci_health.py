#!/usr/bin/env python3
import json
from pathlib import Path
import tempfile
import unittest
from ci_health import classify_job, merge, pages, save, summarize


def record(attempt, conclusion, run_id=10):
    return {'run_id':run_id, 'attempt':attempt, 'commit':'abc', 'status':'completed',
            'conclusion':conclusion, 'url':'https://example.test/run', 'jobs':[]}

class BuildHistory(unittest.TestCase):
    def test_runner_allocation_failure_is_infrastructure(self):
        job = {'status':'completed','conclusion':'cancelled','annotations':[{'message':'The job was not acquired by Runner of type hosted even after multiple attempts'}]}
        self.assertEqual(classify_job(job),'infrastructure')
        job['annotations'] = [{'message':'Due to capacity constraints, jobs may experience longer queue times.'}]
        self.assertEqual(classify_job(job),'cancelled')

    def test_failures_and_timeouts_are_not_guessed_as_compiler_errors(self):
        self.assertEqual(classify_job({'status':'completed','conclusion':'failure'}),'check_failure')
        self.assertEqual(classify_job({'status':'completed','conclusion':'timed_out'}),'timeout')
        self.assertEqual(classify_job({'status':'queued','conclusion':None}),'pending')

    def test_successful_rerun_preserves_failure_and_links_recovery(self):
        history = merge({},[record(1,'failure')],'first')
        history = merge(history,[record(2,'success')],'second')
        summary = summarize(history)
        self.assertEqual(len(history['attempts']),2)
        self.assertEqual(summary['latest']['conclusion'],'success')
        self.assertTrue(summary['failure_history'][0]['recovered_by_rerun'])

    def test_new_commit_does_not_silently_resolve_old_failure(self):
        history = merge({},[record(1,'failure'),record(1,'success',run_id=11)],'now')
        self.assertFalse(summarize(history)['failure_history'][0]['recovered_by_rerun'])

    def test_pending_snapshot_cannot_overwrite_completed_evidence(self):
        completed = record(1,'failure')
        history = merge({},[completed],'first')
        pending = {**completed,'status':'in_progress','conclusion':None}
        self.assertEqual(merge(history,[pending],'second')['attempts'][0]['conclusion'],'failure')

    def test_pagination_and_atomic_persistence(self):
        responses = iter([{'jobs':list(range(100))},{'jobs':[100]}])
        self.assertEqual(len(pages('endpoint','jobs',lambda _:next(responses))),101)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'history.json'
            save(path,{'attempts':[record(1,'failure')]})
            self.assertEqual(json.loads(path.read_text())['attempts'][0]['conclusion'],'failure')
            self.assertEqual(list(Path(directory).iterdir()),[path])

if __name__ == '__main__':
    unittest.main(verbosity=2)
