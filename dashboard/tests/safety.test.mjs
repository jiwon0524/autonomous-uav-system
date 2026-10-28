import test from 'node:test';
import assert from 'node:assert/strict';
import { initialSafetyState, safetyReducer, healthSummary, criticalFlightEvents, obstacleMarker } from '../src/safety/model.ts';
const at='2026-10-27T10:00:00Z';
const health={...initialSafetyState.health,depth:'WARNING'};
const flight={id:'fixture-flight',severity:'CRITICAL',scope:'FLIGHT',cause:'fixture collision',device:null,occurredAt:at};
test('default disconnected state is not critical and creates no notifications',()=>{
 assert.equal(healthSummary(initialSafetyState.health).status,'DISCONNECTED');assert.equal(criticalFlightEvents(initialSafetyState).length,0);assert.equal(initialSafetyState.notifications.length,0);
});
test('one device warning leaves other devices independent and deduplicates repeated polling',()=>{
 const state=safetyReducer(initialSafetyState,{type:'health',health,at});
 assert.equal(state.health.rgb,'DISCONNECTED');assert.equal(state.health.depth,'WARNING');assert.deepEqual(healthSummary(state.health),{status:'WARNING',count:1});
 const repeated=safetyReducer(state,{type:'health',health,at});assert.equal(repeated.notifications.length,1);assert.equal(repeated.logs.length,state.logs.length);
});
test('dismissal neither clears health nor an active flight safety event',()=>{
 let state=safetyReducer(initialSafetyState,{type:'event',event:flight});
 state=safetyReducer(state,{type:'dismiss',id:flight.id});assert.equal(state.notifications[0].dismissed,true);assert.equal(criticalFlightEvents(state).length,1);
 state=safetyReducer(state,{type:'clear-event',id:flight.id,at});assert.equal(criticalFlightEvents(state).length,0);assert.equal(state.logs.at(-1).text,'[SAFETY] Critical state cleared');
 assert.equal(safetyReducer(state,{type:'event',event:flight}),state);
});
test('device critical is not automatically flight critical',()=>{
 let state=safetyReducer(initialSafetyState,{type:'health',health:{...health,depth:'CRITICAL'},at});
 assert.equal(criticalFlightEvents(state).length,0);
 state=safetyReducer(state,{type:'event',event:{...flight,id:'device-event',scope:'DEVICE',device:'depth'}});
 assert.equal(criticalFlightEvents(state).length,0);
});
test('recovery logs once without fabricating sensor recovery in other devices',()=>{
 let state=safetyReducer(initialSafetyState,{type:'health',health,at});
 state=safetyReducer(state,{type:'health',health:{...health,depth:'NORMAL'},at});
 assert.match(state.logs.at(-1).text,/recovered/);assert.equal(state.health.lidar,'DISCONNECTED');
});
test('obstacle severity supplied by source is not inferred from distance',()=>{
 const obstacle={id:'fixture-obstacle',status:'WARNING',direction:null,distance:0.1,source:'LIDAR',firstDetected:null,lastDetected:null,position:null,pathImpact:'UNKNOWN',altitude:null};
 assert.equal(obstacleMarker(obstacle),null);
 const withPosition={...obstacle,position:{frame:'LOCAL_NED',north:1,east:2}};
 assert.equal(obstacleMarker(withPosition).obstacle.risk,'WARNING');
 let state=safetyReducer(initialSafetyState,{type:'obstacle',obstacle:withPosition,at});
 state=safetyReducer(state,{type:'obstacle',obstacle:{...withPosition,distance:0.01},at});
 assert.equal(state.notifications.length,1);assert.equal(state.logs.length,1);assert.equal(criticalFlightEvents(state).length,0);
});
test('target lost and sensor disconnect cannot dispatch flight commands or flight events',()=>{
 const state=safetyReducer(initialSafetyState,{type:'health',health:initialSafetyState.health,at});
 assert.deepEqual(state.active,{});assert.equal('command' in state,false);assert.equal(state.failsafe,null);
});
