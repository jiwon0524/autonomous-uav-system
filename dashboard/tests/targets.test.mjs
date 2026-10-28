import test from 'node:test';
import assert from 'node:assert/strict';
import { initialTargetState, targetReducer, formatTargetEvent } from '../src/targets/model.ts';
const frame = (status='UNKNOWN', lastSeen='2026-10-27T10:00:00Z') => ({id:'unit-test-id',type:'PERSON',status,label:'test fixture',confidence:null,firstSeen:'2026-10-27T09:00:00Z',lastSeen});
test('repeated frames replace a stable ID rather than adding rows',()=>{
  let state=initialTargetState;
  for(let i=0;i<30;i++)state=targetReducer(state,{type:'upsert',targets:[frame()]});
  assert.equal(Object.keys(state.byId).length,1);
  assert.equal(initialTargetState.selectedId,null);
  assert.equal(Object.keys(initialTargetState.byId).length,0);
});
test('older frame cannot overwrite newer status or first seen time',()=>{
  let state=targetReducer(initialTargetState,{type:'upsert',targets:[frame('LOST')]});
  state=targetReducer(state,{type:'upsert',targets:[frame('TRACKING','2026-10-27T09:59:00Z')]});
  assert.equal(state.byId['unit-test-id'].status,'LOST');
});
test('tracking request records intent without claiming actual tracking',()=>{
  let state=targetReducer(initialTargetState,{type:'upsert',targets:[frame()]});
  state=targetReducer(state,{type:'select',id:'unit-test-id'});
  state=targetReducer(state,{type:'request-tracking',id:'unit-test-id'});
  assert.equal(state.trackingRequestedId,'unit-test-id');
  assert.equal(state.byId['unit-test-id'].status,'UNKNOWN');
  assert.equal(state.selectedId,'unit-test-id');
});
test('source-provided loss, reacquiring and recovery preserve last seen position',()=>{
  let state=targetReducer(initialTargetState,{type:'upsert',targets:[{...frame('LOST'),type:'VEHICLE',lastSeenPosition:{frame:'LOCAL_NED',north:1,east:2,down:null}}]});
  state=targetReducer(state,{type:'upsert',targets:[frame('REACQUIRING')]});
  assert.equal(state.byId['unit-test-id'].status,'REACQUIRING');
  assert.equal(state.byId['unit-test-id'].lastSeenPosition.north,1);
  state=targetReducer(state,{type:'upsert',targets:[{...frame('TRACKING','2026-10-27T10:01:00Z'),reacquiredAt:'2026-10-27T10:01:00Z'}]});
  assert.equal(state.byId['unit-test-id'].status,'TRACKING');
  assert.equal(formatTargetEvent({id:'event',targetId:'unit-test-id',type:'REACQUIRED',occurredAt:'2026-10-27T10:01:00Z'}),'[TRACK] unit-test-id reacquired');
});
test('all types can be authorized and missing IDs cannot be selected',()=>{
  let state=targetReducer(initialTargetState,{type:'upsert',targets:['PERSON','VEHICLE','OBJECT'].map(type=>({...frame('AUTHORIZED'),type,id:type}))});
  assert.equal(Object.values(state.byId).filter(target=>target.status==='AUTHORIZED').length,3);
  assert.equal(targetReducer(state,{type:'select',id:'missing'}),state);
});
