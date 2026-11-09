'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const {spawnSync} = require('node:child_process');
const {execute, withCaptureDeadline, loadNative} = require('./desktop-control.cjs');
(async () => {
  const calls = [];
  const native = {typeString: text => calls.push(['type', text]), moveMouse: (...v) => calls.push(['move', ...v]), mouseClick: b => calls.push(['click', b])};
  const getContext = () => ({screen_locked:false,frontmost_pid:23});
  await assert.rejects(execute({action:'type',expect_pid:24,text:'x'}, {native,getContext}), /Foreground application changed/);
  assert.deepEqual(calls, []);
  await assert.rejects(execute({action:'click',expect_pid:23,x:0.5,y:4}, {native,getContext}), /integer/);
  assert.deepEqual(calls, []);
  await assert.rejects(execute({action:'type',expect_pid:23,text:'x'}, {native,getContext:()=>({screen_locked:true,frontmost_pid:23})}), /unlock/);
  assert.deepEqual(calls, []);
  const typed = await execute({action:'type',expect_pid:23,text:'x'}, {native,getContext});
  assert.equal(typed.delivery_verified,false);
  assert.deepEqual(calls,[['type','x']]);
  const denied = new Error('native permission denied');
  await assert.rejects(execute({action:'type',expect_pid:23,text:'x'}, {native:{typeString:()=>{throw denied;}},getContext}), e=>e===denied);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(),'citta-desktop-unit-'));
  try {
    const output=path.join(dir,'capture.png');
    const oldCapture=Buffer.from('previous screenshot');
    fs.writeFileSync(output,oldCapture);
    const captureRequest={action:'capture',backend:'system-window',window_id:55,output};
    let temporary;
    await assert.rejects(execute(captureRequest,{native,getContext,run:(_file,args)=>{
      temporary=path.dirname(args.at(-1));
      assert.notEqual(args.at(-1),output);
      assert.equal(fs.statSync(temporary).mode & 0o777,0o700);
      // Simulate screencapture returning success without producing a fresh file.
    }}), /ENOENT/);
    assert.deepEqual(fs.readFileSync(output),oldCapture);
    assert.equal(fs.existsSync(temporary),false);
    await assert.rejects(execute(captureRequest,{native,getContext,run:(_file,args)=>{
      temporary=path.dirname(args.at(-1));
      fs.writeFileSync(args.at(-1),'not a PNG');
    }}), /PNG data/);
    assert.deepEqual(fs.readFileSync(output),oldCapture);
    assert.equal(fs.existsSync(temporary),false);
    const png=Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aX1sAAAAASUVORK5CYII=','base64');
    const captured=await execute(captureRequest,{native,getContext,run:(_file,args)=>{
      temporary=path.dirname(args.at(-1));
      fs.writeFileSync(args.at(-1),png);
    }});
    assert.equal(captured.width,1);
    assert.equal(captured.height,1);
    assert.equal(captured.visual_verified,false);
    assert.deepEqual(fs.readFileSync(output),png);
    assert.equal(fs.statSync(output).mode & 0o777,0o600);
    assert.equal(fs.existsSync(temporary),false);
    const packageDir=path.join(dir,'.local/share/citta/desktop-control/node_modules/@priyankark/rusty-robot');
    fs.mkdirSync(packageDir,{recursive:true});
    fs.writeFileSync(path.join(packageDir,'index.js'),'module.exports = {loaded: "default-private-install"};');
    const override=path.join(dir,'override.cjs');
    fs.writeFileSync(override,'module.exports = {loaded: "explicit-override"};');
    assert.equal(loadNative({},dir).loaded,'default-private-install');
    assert.equal(loadNative({CITTA_DESKTOP_ADDON:override},dir).loaded,'explicit-override');
    const state=path.join(dir,'state.json');
    const record={marker:'marker',text:'',frontmost:true,field_focused:true,pid:23,window_id:55,updated_at:Date.now()/1000};
    fs.writeFileSync(state,JSON.stringify(record));
    await assert.rejects(execute({action:'verify-fixture',state,marker:'marker'},{native:{typeString:()=>{}},getContext}), /did not confirm marker delivery/);
    const result=await execute({action:'verify-fixture',state,marker:'marker'}, {native:{typeString:text=>fs.writeFileSync(state,JSON.stringify({...record,text,updated_at:Date.now()/1000}))},getContext});
    assert.equal(result.delivery_verified,true);
    assert.equal(result.window_id,55);
  } finally { fs.rmSync(dir,{recursive:true,force:true}); }
  let called = false;
  await assert.rejects(withCaptureDeadline({action:'capture',timeout_ms:60001}, () => {called=true;}), /500..60000/);
  assert.equal(called,false);
  const modulePath = JSON.stringify(path.join(__dirname,'desktop-control.cjs'));
  const pending = spawnSync(process.execPath, ['-e', `
    const {execute,withCaptureDeadline}=require(${modulePath});
    const request={action:'capture',output:'/unused.png',timeout_ms:500};
    const native={captureFrame:()=>{setInterval(()=>{},1000);return new Promise(()=>{});}};
    withCaptureDeadline(request,()=>execute(request,{native,getContext:()=>({screen_locked:false})}));
  `], {encoding:'utf8',timeout:5000});
  assert.ifError(pending.error);
  assert.equal(pending.status,124);
  const timedOut = JSON.parse(pending.stdout.trim());
  assert.equal(timedOut.error,'capture_deadline_exceeded');
  assert.equal(timedOut.timeout_ms,500);
  const late=spawnSync(process.execPath,['-e',`
    const {withCaptureDeadline}=require(${modulePath});
    withCaptureDeadline({action:'capture',timeout_ms:500},()=>{
      Atomics.wait(new Int32Array(new SharedArrayBuffer(4)),0,0,650);
      return 'late';
    }).then(()=>console.log('incorrect late success'));
  `],{encoding:'utf8',timeout:3000});
  assert.ifError(late.error);
  assert.equal(late.status,124);
  assert.equal(JSON.parse(late.stdout.trim()).error,'capture_deadline_exceeded');
  for (const fail of [false,true]) {
    const completed=spawnSync(process.execPath,['-e',`
      const {withCaptureDeadline}=require(${modulePath});
      withCaptureDeadline({action:'capture',timeout_ms:500},async()=>{
        if (${fail}) throw Error('native capture failed');
        return 'completed';
      }).then(value=>console.log(value)).catch(error=>{console.log(error.message);process.exitCode=1;});
    `],{encoding:'utf8',timeout:3000});
    assert.ifError(completed.error);
    assert.equal(completed.status,fail?1:0);
    assert.equal(completed.stdout.trim(),fail?'native capture failed':'completed');
  }
  console.log('PASS: private addon loading, focus/unlock/integer checks, native errors, fixture delivery, fresh atomic capture replacement/cleanup, pending-capture termination, and cleared deadlines.');
})().catch(error=>{console.error(error);process.exitCode=1;});
