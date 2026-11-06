#!/usr/bin/env python3
"""Render a launchd or systemd user service. Does not install or enable it."""
import argparse
import os
from pathlib import Path
import plistlib
import sys


def systemd_quote(value):
    if any(c in value for c in '\r\n\x00'):
        raise ValueError('service paths cannot contain control characters')
    return '"' + value.replace('\\', '\\\\').replace('"', '\\"').replace('%', '%%').replace('$', '$$') + '"'


def render(platform, binary, config, logs):
    args = [str(binary), '--config', str(config), 'daemon']
    if platform == 'macos':
        return plistlib.dumps({
            'Label':'io.citta.daemon', 'ProgramArguments':args,
            'RunAtLoad':True, 'KeepAlive':{'SuccessfulExit':False},
            'ThrottleInterval':10, 'ExitTimeOut':30, 'ProcessType':'Background',
            'StandardOutPath':str(logs / 'stdout.log'),
            'StandardErrorPath':str(logs / 'stderr.log'),
            'EnvironmentVariables':{'RUST_LOG':'citta=info'},
        })
    command = ' '.join(systemd_quote(v) for v in args)
    return f'''[Unit]
Description=Citta cross-device agent daemon
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart={command}
Restart=on-failure
RestartSec=5
TimeoutStopSec=30
KillMode=control-group
UMask=0077
NoNewPrivileges=true
Environment=RUST_LOG=citta=info

[Install]
WantedBy=default.target
'''.encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--platform', choices=['macos','linux'], default='macos' if sys.platform == 'darwin' else 'linux')
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--config', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    binary, config = args.binary.resolve(strict=True), args.config.resolve(strict=True)
    if not binary.is_file() or not os.access(binary, os.X_OK) or not config.is_file():
        parser.error('binary must be executable and config must be a file')
    logs = config.parent / 'logs'
    logs.mkdir(mode=0o700, exist_ok=True)
    os.chmod(logs, 0o700)
    content = render(args.platform, binary, config, logs)
    # Exclusive creation protects an existing service from accidental replacement.
    fd = os.open(args.output, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'wb') as output:
        output.write(content)
    print(args.output)

if __name__ == '__main__':
    main()
