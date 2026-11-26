import {test} from 'node:test';
import assert from 'node:assert/strict';
import {alternateQuery, createSearchGate, mergeResults, normalizeQuery, rank, searchError} from '../src/locationSearch.ts';
const item=(title,x,y,kind='place',candidate=false)=>({title,point:{x,y},kind,candidate});
test('normalization and bounded alternate candidates',()=>{
 assert.equal(normalizeQuery('  동의  대학교  '),'동의 대학교');
 assert.equal(alternateQuery('동의대'),'동의대학교');
 assert.equal(alternateQuery('동의대학고'),'동의대학교');
 assert.equal(alternateQuery('동의 대학교'),'동의대학교');
 assert.equal(alternateQuery('서울역'),null);
});
test('rank exact, normalized, partial and alternative and retain distinct locations',()=>{
 const entries=[item('동의대학교', '129','35','place',true),item('동의대학교 병원','128','35'),item('동의 대학교','127','35'),item('동의대학교','126','35'),item('동의대학교','126','35','road'),item('동의대학교','125','35'),item('잘못된 좌표','','35')];
 const out=mergeResults(entries,'동의대학교');
 assert.deepEqual(out.map(x=>x.point.x),['126','125','127','128','129']);
 assert.deepEqual(out.map(x=>rank(x,'동의대학교')),[0,0,1,2,4]);
});
test('different unnamed addresses at the same coordinates remain distinct',()=>{
 const a={point:{x:'127',y:'37'},kind:'road',address:{road:'첫 주소'}};
 const b={point:{x:'127',y:'37'},kind:'road',address:{road:'둘 주소'}};
 assert.equal(mergeResults([a,b],'주소').length,2);
});
test('classifies known error codes without revealing server details',()=>{
 assert.match(searchError('INVALID_KEY'),/인증/); assert.match(searchError('429'),/제한/); assert.match(searchError('OTHER'),/오류/);
});

test('late replies and cancelled requests cannot replace newer results', async()=>{
 const gate=createSearchGate(); let shown='';
 const old=gate.start(); let finish; const pending=new Promise(resolve=>finish=resolve);
 const late=pending.then(()=>{if(gate.isCurrent(old)) shown='old';});
 const current=gate.start(); if(gate.isCurrent(current)) shown='new'; finish(); await late;
 assert.equal(shown,'new'); gate.cancel(); assert.equal(gate.isCurrent(current),false);
});
