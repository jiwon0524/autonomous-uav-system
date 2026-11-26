import test from 'node:test';
import assert from 'node:assert/strict';
import {build} from 'esbuild';
import {mkdtemp,writeFile,rm} from 'node:fs/promises';
import {pathToFileURL} from 'node:url';

test('recording status subscribers share polling and ignore late completion after cleanup',async()=>{
 const dir=await mkdtemp('node_modules/.firescene-status-test-');const original=globalThis.window, originalTauri=globalThis.isTauri;
 try {
  const output=await build({entryPoints:['src/RecordingStatus.tsx'],bundle:true,write:false,platform:'node',format:'esm',external:['react','react/jsx-runtime']});
  const path=`${dir}/status.mjs`;await writeFile(path,output.outputFiles[0].text);
  let interval,cleared=0,deadlineCleared=0;const resolvers=[];
  globalThis.isTauri=true;globalThis.window={__TAURI_INTERNALS__:{invoke:()=>new Promise(resolve=>resolvers.push(resolve))},setInterval:fn=>{interval=fn;return 7;},clearInterval:id=>{assert.equal(id,7);cleared++;},setTimeout:()=>9,clearTimeout:id=>{assert.equal(id,9);deadlineCleared++;}};
  const {subscribeRecordingStatus,recordingStatusSnapshot}=await import(pathToFileURL(`${process.cwd()}/${path}`));
  let first=0,second=0;const off1=subscribeRecordingStatus(()=>first++),off2=subscribeRecordingStatus(()=>second++);
  assert.equal(resolvers.length,1);resolvers.shift()({active:true,id:'1',started_ms:1,bytes:10,error:null});await new Promise(resolve=>setImmediate(resolve));
  assert.equal(first,1);assert.equal(second,1);assert.equal(recordingStatusSnapshot().active,true);assert.equal(recordingStatusSnapshot().reachable,true);assert.equal(deadlineCleared,1);
  off1();assert.equal(cleared,0);interval();assert.equal(resolvers.length,1);off2();assert.equal(cleared,1);assert.equal(recordingStatusSnapshot().active,false);assert.equal(recordingStatusSnapshot().reachable,false);
  resolvers.shift()({active:true,id:'late',started_ms:1,bytes:10,error:null});await new Promise(resolve=>setImmediate(resolve));assert.equal(recordingStatusSnapshot().active,false);assert.equal(second,1);
 } finally {globalThis.window=original;globalThis.isTauri=originalTauri;await rm(dir,{recursive:true,force:true});}
});
