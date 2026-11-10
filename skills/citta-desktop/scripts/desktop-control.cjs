#!/usr/bin/env node
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const {execFileSync} = require('node:child_process');
const {performance} = require('node:perf_hooks');

function integer(value, name) {
  if (typeof value !== 'number' || !Number.isSafeInteger(value) || value < -2147483648 || value > 2147483647) throw Error(`${name} must be a signed 32-bit integer`);
  return value;
}
function context(helper = path.join(__dirname, 'desktop-context')) {
  const current = JSON.parse(execFileSync(helper, [], {encoding: 'utf8', timeout: 5000}));
  const raw = execFileSync('/usr/sbin/ioreg', ['-n', 'Root', '-d1'], {encoding: 'utf8', timeout: 5000});
  const match = raw.match(/"IOConsoleLocked"\s*=\s*(Yes|No)/);
  current.screen_locked = match ? match[1] === 'Yes' : null;
  return current;
}
function focus(request, current) {
  integer(request.expect_pid, 'expect_pid');
  if (request.expect_pid <= 0) throw Error('expect_pid must be positive');
  if (current.screen_locked !== false) throw Error('Session unlock state is not established');
  if (current.frontmost_pid !== request.expect_pid) throw Error('Foreground application changed; refresh the observation before input');
}
function pngDimensions(bytes) {
  if (bytes.length < 24 || bytes.subarray(0,8).toString('hex') !== '89504e470d0a1a0a') throw Error('Capture did not return PNG data');
  const width = bytes.readUInt32BE(16), height = bytes.readUInt32BE(20);
  if (!width || !height) throw Error('Capture returned empty dimensions');
  return {width, height};
}
async function execute(request, deps) {
  const {native, getContext = context, run = execFileSync} = deps;
  if (request.action === 'status') {
    const result = {permissions: native.getPermissions(), process: {pid: process.pid, executable: process.execPath}, context: getContext(), execution_verified: false};
    try { result.cursor = native.getMousePos(); } catch (error) { result.cursor_error = error.message; }
    return result;
  }
  const permissions = native.getPermissions();
  const ready = getContext();
  if (permissions.accessibility !== true || permissions.screenCapture !== true) throw Error('Desktop permission refused');
  if (ready.screen_locked !== false) throw Error('Session unlock state is not established');
  if (ready.display_error !== 0 || !ready.displays?.length) throw Error('No active display');
  if (request.action === 'capture') {
    if (typeof request.output !== 'string' || !path.isAbsolute(request.output)) throw Error('output must be an absolute PNG path');
    const current = getContext();
    if (current.screen_locked !== false) throw Error('Session unlock state is not established');
    let bytes, metadata;
    if ((request.backend || 'rusty-robot') === 'rusty-robot') {
      const width = request.width === undefined ? 1440 : integer(request.width, 'width');
      if (width < 1 || width > 16384) throw Error('width is outside 1..16384');
      const frame = await native.captureFrame(width, 85, null, true, 'png');
      if (!frame.data) throw Error('Forced capture returned no image data');
      bytes = Buffer.from(frame.data);
      const main = current.displays.find(d => d.main);
      metadata = {backend: 'rusty-robot', source_width: frame.sourceWidth, source_height: frame.sourceHeight,
        logical_display: main || null};
    } else if (request.backend === 'system-window') {
      const id = integer(request.window_id, 'window_id');
      if (id <= 0) throw Error('window_id must be positive');
      // Explicit selected-window backend; never silently substitute it for native capture.
      // A successful subprocess may produce nothing. Never mistake an old output
      // for a fresh capture; keep it intact until a new private PNG is validated.
      const temporary = fs.mkdtempSync(path.join(path.dirname(request.output), '.citta-capture-'));
      try {
        const captured = path.join(temporary, 'capture.png');
        run('/usr/sbin/screencapture', ['-x', '-o', '-l', String(id), captured], {timeout: 15000});
        const dimensions = pngDimensions(fs.readFileSync(captured));
        fs.chmodSync(captured, 0o600);
        fs.renameSync(captured, request.output);
        return {backend: 'system-window', window_id: id, ...dimensions, path: request.output, visual_verified: false};
      } finally { fs.rmSync(temporary, {recursive: true, force: true}); }
    } else throw Error('Unsupported capture backend');
    const dimensions = pngDimensions(bytes);
    if (metadata.backend === 'rusty-robot') fs.writeFileSync(request.output, bytes, {mode: 0o600});
    fs.chmodSync(request.output, 0o600);
    return {...metadata, ...dimensions, path: request.output, visual_verified: false};
  }
  if (['type', 'click', 'key', 'scroll'].includes(request.action)) {
    // Validate before querying focus, then issue one requested action. A zero exit
    // status alone never establishes delivery or an external effect.
    if (request.action === 'type' && typeof request.text !== 'string') throw Error('text must be a string');
    if (request.action === 'click') {
      integer(request.x, 'x'); integer(request.y, 'y');
      if (!['left', 'right', 'middle'].includes(request.button || 'left')) throw Error('Invalid mouse button');
    }
    if (request.action === 'scroll') { integer(request.x, 'x'); integer(request.y, 'y'); }
    if (request.action === 'key') {
      if (typeof request.key !== 'string' || !Array.isArray(request.modifiers || []) || !(request.modifiers || []).every(v => typeof v === 'string')) throw Error('Invalid key/modifiers');
    }
    focus(request, getContext());
    switch (request.action) {
      case 'type': native.typeString(request.text); break;
      case 'click': native.moveMouse(request.x, request.y); native.mouseClick(request.button || 'left'); break;
      case 'key': native.keyTap(request.key, request.modifiers || []); break;
      case 'scroll': native.scrollMouse(request.x, request.y); break;
    }
    return {action: request.action, issued: true, delivery_verified: false};
  }
  if (request.action === 'verify-fixture') {
    if (typeof request.state !== 'string' || !path.isAbsolute(request.state) || typeof request.marker !== 'string' || !request.marker) throw Error('An absolute fixture state path and marker are required');
    const read = () => JSON.parse(fs.readFileSync(request.state, 'utf8'));
    const before = read();
    if (before.marker !== request.marker || before.text !== '' || !before.frontmost || !before.field_focused || Date.now()/1000 - before.updated_at > 2) throw Error('Owned fixture is not freshly focused and empty');
    focus({expect_pid: before.pid}, getContext());
    native.typeString(request.marker);
    await new Promise(resolve => setTimeout(resolve, 400));
    const after = read();
    if (after.pid !== before.pid || after.marker !== before.marker || after.text !== request.marker || Date.now()/1000 - after.updated_at > 2) throw Error('Owned fixture did not confirm marker delivery');
    return {action: request.action, delivery_verified: true, pid: after.pid, window_id: after.window_id};
  }
  throw Error('Unknown action');
}
async function withCaptureDeadline(request, operation) {
  if (request.action !== 'capture') return operation();
  const timeout = request.timeout_ms === undefined ? 15000 : integer(request.timeout_ms, 'timeout_ms');
  if (timeout < 500 || timeout > 60000) throw Error('timeout_ms must be within 500..60000');
  const started = performance.now();
  const terminate = () => {
    // A rejected Promise or exitCode alone can leave native capture handles alive.
    // Emit the small result synchronously, then terminate this helper process.
    fs.writeSync(1, JSON.stringify({ok:false,error:'capture_deadline_exceeded',
      action:'capture',timeout_ms:timeout,backend:request.backend || 'rusty-robot'}) + '\n');
    process.exit(124);
  };
  const timer = setTimeout(terminate, timeout);
  try {
    const result = await operation();
    // Synchronous native work can prevent a timer callback. Do not report a late
    // result as timely success; an outer subprocess deadline handles permanent blocking.
    if (performance.now() - started >= timeout) terminate();
    return result;
  } finally { clearTimeout(timer); }
}
function loadNative(env = process.env, home = os.homedir()) {
  const location = env.CITTA_DESKTOP_ADDON || path.join(home, '.local/share/citta/desktop-control/node_modules/@priyankark/rusty-robot');
  return require(location);
}
async function main() {
  const file = process.argv[2];
  if (!file) throw Error('usage: desktop-control.cjs REQUEST_JSON_OR_MINUS');
  const request = JSON.parse(fs.readFileSync(file === '-' ? 0 : file, 'utf8'));
  if (request.action !== 'status') {
    const permit = process.env.CITTA_DESKTOP_PERMIT_FD;
    if (!permit) {
      let result;
      try { result = execFileSync('python3', [path.join(__dirname, 'desktop-session.py'), 'action', '--request', '-'],
        {input: JSON.stringify(request), encoding: 'utf8', timeout: 28000});
      } catch (error) {
        if (error.stdout) { process.stdout.write(error.stdout); process.exitCode = 1; return; }
        throw error;
      }
      process.stdout.write(result);
      return;
    }
    if (fs.readFileSync(Number(permit), 'utf8') !== 'guarded-session\n') throw Error('Valid owning session required');
    fs.closeSync(Number(permit));
  }
  const result = await withCaptureDeadline(request, () => {
    const native = loadNative();
    return execute(request, {native, getContext: () => context(process.env.CITTA_DESKTOP_CONTEXT)});
  });
  console.log(JSON.stringify({ok: true, ...result}));
}
if (require.main === module) main().catch(error => { console.log(JSON.stringify({ok:false,error:error.message})); process.exitCode = 1; });
module.exports = {execute, focus, pngDimensions, withCaptureDeadline, loadNative};
