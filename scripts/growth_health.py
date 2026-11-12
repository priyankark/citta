#!/usr/bin/env python3
"""Save private GitHub launch metrics. No client telemetry and no posting."""

import argparse
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess
import uuid


def fetch(endpoint):
    try:
        result = subprocess.run(
            ["gh", "api", endpoint], capture_output=True, text=True, timeout=45
        )
        if result.returncode:
            return {"status": "unavailable", "error": "GitHub request failed"}
        return {"status": "ok", "data": json.loads(result.stdout)}
    except (OSError, ValueError, subprocess.TimeoutExpired) as error:
        return {"status": "unavailable", "error": type(error).__name__}


def collect(repo):
    endpoints = {
        "repository": f"repos/{repo}",
        "views": f"repos/{repo}/traffic/views",
        "clones": f"repos/{repo}/traffic/clones",
        "referrers": f"repos/{repo}/traffic/popular/referrers",
        "release": f"repos/{repo}/releases/tags/v0.1.0",
        "issues": f"repos/{repo}/issues?state=all&sort=created&direction=desc&per_page=100",
    }
    with ThreadPoolExecutor(max_workers=6) as executor:
        results = dict(zip(endpoints, executor.map(fetch, endpoints.values())))
    record = {
        "observed_at": datetime.now(timezone.utc).isoformat(),
        "repo": repo,
        "sources": {},
        "limits": [
            "Views and clones are GitHub's rolling window, not daily new users.",
            "Downloads include maker tests, CI, repeat downloads and upgrades.",
            "First-task and repeat-use counts are voluntary reports, not telemetry.",
            "Owner reports are excluded. Issues are limited to the latest 100 records.",
            "Missing data is unavailable, never zero. Earlier snapshots are retained.",
        ],
    }
    for name, result in results.items():
        if result["status"] != "ok":
            record["sources"][name] = result
            continue
        data = result["data"]
        if name == "repository":
            data = {k: data[k] for k in ("stargazers_count", "forks_count", "subscribers_count")}
        elif name == "release":
            data = {
                "tag": data["tag_name"],
                "assets": [
                    {"name": a["name"], "downloads": a["download_count"]}
                    for a in data["assets"] if a["name"].endswith(".tar.gz")
                ],
            }
            data["archive_downloads"] = sum(a["downloads"] for a in data["assets"])
        elif name == "issues":
            latest = [i for i in data if "pull_request" not in i]
            reports = [i for i in latest if i["title"].startswith("First task:")
                       and i.get("author_association") != "OWNER"]
            successful = [i for i in reports if "I completed an agent or model task on another device" in (i.get("body") or "")]
            repeat = [i for i in successful if "### Have you used it again for another task?\n\nYes" in (i.get("body") or "")]
            data = {
                "window": "latest 100 GitHub issue/PR records",
                "window_full": len(data) == 100,
                "first_task_reporters": len({i["user"]["id"] for i in successful}),
                "repeat_task_reporters": len({i["user"]["id"] for i in repeat}),
                "open_issues": [
                    {"number": i["number"], "title": i["title"], "url": i["html_url"]}
                    for i in latest if i["state"] == "open"
                ],
            }
        record["sources"][name] = {"status": "ok", "data": data}
    return record


def save(record, directory):
    directory.mkdir(parents=True, exist_ok=True, mode=0o700)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    destination = directory / f"{stamp}-{uuid.uuid4().hex[:8]}.json"
    fd = os.open(destination, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, "w") as handle:
        json.dump(record, handle, indent=2)
        handle.write("\n")
    return destination


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", default="priyankark/citta")
    parser.add_argument("--output-dir", type=Path, default=Path("local/growth-history"))
    parser.add_argument("--until", help="Stop collecting at this ISO timestamp (requires a timezone)")
    args = parser.parse_args()
    if args.until:
        try:
            deadline = datetime.fromisoformat(args.until)
            if deadline.tzinfo is None:
                raise ValueError("timezone required")
        except ValueError:
            parser.error("--until must be an ISO timestamp with a timezone")
        if datetime.now(timezone.utc) >= deadline:
            print(json.dumps({"status": "collection_window_ended"}))
            return 0
    record = collect(args.repo)
    path = save(record, args.output_dir)
    print(json.dumps({"snapshot": str(path), "observed_at": record["observed_at"],
                      "sources": {k: v["status"] for k, v in record["sources"].items()}}))
    return int(any(v["status"] != "ok" for v in record["sources"].values()))


if __name__ == "__main__":
    raise SystemExit(main())
