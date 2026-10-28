import test from 'node:test';
import assert from 'node:assert/strict';
import { parseBridgeStatus } from '../src/services/bridgeStatus.ts';
import { initialSensors } from '../src/sensors/model.ts';
import { isSensorSnapshot,isSensorUpdate,isSensorEvent } from '../src/sensors/validation.ts';
import { formatDateTime } from '../src/utils/format.ts';
import { initialTargetState,targetReducer } from '../src/targets/model.ts';
import { initialSafetyState,safetyReducer } from '../src/safety/model.ts';
test('bridge rejects malformed or contradictory states instead of displaying connected',()=>{
 assert.deepEqual(parseBridgeStatus({ros2:'connected',px4:'disconnected'}),{ros2:'connected',px4:'disconnected'});
 for(const body of [null,{},'connected',{ros2:'disconnected',px4:'connected'},{ros2:'connected',px4:'unknown'}])assert.throws(()=>parseBridgeStatus(body));
});
test('cross-window sensor snapshot rejects missing fields invalid values and malformed events',()=>{
 assert.equal(isSensorSnapshot(initialSensors),true);
 for(const body of [null,{}, {...initialSensors,depth:{}},{...initialSensors,rgb:{...initialSensors.rgb,status:'CRITICAL'}},{...initialSensors,rgb:{...initialSensors.rgb,fps:NaN}},{...initialSensors,lidar:{...initialSensors.lidar,scanPoints:[{angleRadians:0,distanceMeters:Infinity}]}}])assert.equal(isSensorSnapshot(body),false);
 assert.equal(isSensorUpdate({id:'depth',data:{...initialSensors.depth,status:'WARNING'}},initialSensors),true);
 assert.equal(isSensorUpdate({id:'missing',data:initialSensors.rgb},initialSensors),false);
 assert.equal(isSensorEvent({id:'x',sensorId:'bad',from:'CONNECTED',to:'ERROR',timestamp:'invalid'}),false);
});
test('invalid timestamps are displayed as empty values',()=>{assert.equal(formatDateTime(null),'--');assert.equal(formatDateTime('invalid'),'--');});
test('arbitrary target IDs cannot modify record prototypes or disappear from selection',()=>{
 const target={id:'__proto__',type:'OBJECT',status:'UNKNOWN',label:'isolated fixture',confidence:null,firstSeen:null,lastSeen:null};
 let state=targetReducer(initialTargetState,{type:'upsert',targets:[target]});
 assert.equal(Object.hasOwn(state.byId,'__proto__'),true);
 assert.equal(Object.getPrototypeOf(state.byId),Object.prototype);
 state=targetReducer(state,{type:'select',id:'__proto__'});assert.equal(state.selectedId,'__proto__');
});
test('safety accepts valid event IDs matching inherited object names',()=>{
 const state=safetyReducer(initialSafetyState,{type:'event',event:{id:'constructor',severity:'WARNING',scope:'DEVICE',device:'depth',cause:'isolated fixture',occurredAt:'2026-10-28T00:00:00Z'}});
 assert.equal(Object.hasOwn(state.active,'constructor'),true);assert.equal(state.notifications.length,1);
});
