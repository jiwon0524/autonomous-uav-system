import { test } from 'node:test';
import assert from 'node:assert/strict';
import { build } from 'esbuild';
import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';

test('mission creation and map selection preserve actual location and geometry', async () => {
  const dir = await mkdtemp('node_modules/.firescene-mission-test-');
  try {
    const result = await build({entryPoints:['src/Missions.tsx'],bundle:true,write:false,platform:'node',format:'esm',external:['react','react/jsx-runtime'],loader:{'.css':'empty'}});
    const path = `${dir}/mission.mjs`; await writeFile(path,result.outputFiles[0].text);
    const {newMission,selectionOf} = await import(pathToFileURL(`${process.cwd()}/${path}`));
    const geometry={type:'Polygon',coordinates:[[[127,37],[127,37.01],[127.01,37.01],[127,37]]]};
    const selected={id:'building-1',name:'건물',address:'주소',center:[127,37],geometry,radius:100};
    const a=newMission(selected), b=newMission(null);
    assert.equal(a.status,'DRAFT'); assert.notEqual(a.id,b.id); assert.deepEqual(selectionOf(a),selected);
    assert.equal(b.lon,null); assert.equal(b.lat,null); assert.equal(selectionOf(b),null);
  } finally { await rm(dir,{recursive:true,force:true}); }
});
