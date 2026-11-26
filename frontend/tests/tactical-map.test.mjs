import test from 'node:test';
import assert from 'node:assert/strict';
import { headingRotation } from '../src/tacticalMap.ts';
const position={system_id:42,component_id:1};
const heading={...position,status:'LIVE',value:90};
test('heading requires fresh finite data from the same vehicle',()=>{
 assert.equal(headingRotation(heading,position),Math.PI/2);
 assert.equal(headingRotation({...heading,value:0},position),0);
 for (const invalid of [undefined,{...heading,status:'STALE'},{...heading,value:null},{...heading,value:NaN},{...heading,system_id:43},{...heading,component_id:2}]) assert.equal(headingRotation(invalid,position),null);
});
