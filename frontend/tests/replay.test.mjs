import test from 'node:test';
import assert from 'node:assert/strict';
import {advanceReplay,replaySample} from '../src/replayTime.ts';

test('replay speeds and seeking never interpolate a missing or disconnected sample',()=>{
 for(const speed of [0.5,1,2,4]) assert.equal(advanceReplay(1000,1000,speed,10000),1000+1000*speed);
 assert.equal(advanceReplay(9000,1000,4,10000),10000);
 assert.equal(advanceReplay(1000,1000,3,10000),1000);
 const records=[{type:'sample',elapsed_ms:1000,snapshot:{phase:'CONNECTED'}},{type:'sample',elapsed_ms:5000,snapshot:{phase:'CONNECTION_LOST'}},{type:'sample',elapsed_ms:8000,snapshot:{phase:'CONNECTED'}}];
 assert.equal(replaySample(records,999),undefined);
 assert.equal(replaySample(records,1500),records[0]);
 assert.equal(replaySample(records,4000),undefined);
 assert.equal(replaySample(records,6000),undefined);
 assert.equal(replaySample(records,8000),records[2]);
});

test('replay map accepts only live, matching raw GPS fields', async()=>{
 const {build}=await import('esbuild');const {mkdtemp,writeFile,rm}=await import('node:fs/promises');const {pathToFileURL}=await import('node:url');
 const dir=await mkdtemp('node_modules/.firescene-replay-test-');
 try {
  const result=await build({entryPoints:['src/ReplayMap.tsx'],bundle:true,write:false,platform:'node',format:'esm',external:['react','react/jsx-runtime']});
  const path=`${dir}/map.mjs`;await writeFile(path,result.outputFiles[0].text);const {replayPoint}=await import(pathToFileURL(`${process.cwd()}/${path}`));
  const lat={value:37,status:'LIVE',timestamp_ms:100,system_id:42,component_id:1}, lon={...lat,value:127};
  const sample={type:'sample',elapsed_ms:0,snapshot:{phase:'CONNECTED',telemetry:{fields:{gps_lat:lat,gps_lon:lon}}}};
  assert.deepEqual(replayPoint(sample),[127,37]);
  assert.equal(replayPoint({...sample,snapshot:{...sample.snapshot,phase:'CONNECTION_LOST'}}),null);
  assert.equal(replayPoint({...sample,snapshot:{...sample.snapshot,telemetry:{fields:{gps_lat:{...lat,status:'STALE'},gps_lon:lon}}}}),null);
  assert.equal(replayPoint({...sample,snapshot:{...sample.snapshot,telemetry:{fields:{gps_lat:lat,gps_lon:{...lon,system_id:43}}}}}),null);
 } finally {await rm(dir,{recursive:true,force:true});}
});
