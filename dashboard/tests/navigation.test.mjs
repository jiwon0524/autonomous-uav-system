import test from 'node:test';
import assert from 'node:assert/strict';
import { initialMapState, emptyNavigationData, mapReducer, safePointReducer, targetLastSeenMarker, navigationMarkers } from '../src/navigation/model.ts';
// Isolated fixtures; never imported by the application.
const position={frame:'LOCAL_NED',north:1,east:2};
const destination={position,altitude:null,distance:null};
test('initial state contains no positions markers or paths',()=>{
 assert.equal(navigationMarkers(emptyNavigationData).length,0);
 assert.equal(emptyNavigationData.paths.length,0);
 assert.equal(initialMapState.destination.stage,'empty');
});
test('mode switching preserves destination and selection, click only selects',()=>{
 let state=mapReducer(initialMapState,{type:'select-destination',destination});
 assert.equal(state.destination.stage,'selected');
 state=mapReducer(state,{type:'select-marker',id:'real-source-id'});
 state=mapReducer(state,{type:'panel',mode:'POSITION_CONTROL'});
 state=mapReducer(state,{type:'panel',mode:'NAVIGATION'});
 assert.deepEqual(state.destination.destination,destination);
 assert.equal(state.selectedMarkerId,'real-source-id');
 state=mapReducer(state,{type:'confirm-destination'});
 assert.equal(state.destination.stage,'confirming');
 assert.equal('command' in state,false);
});
test('safe point setup requires position then name then confirmation',()=>{
 let state=mapReducer(initialMapState,{type:'confirm-safe-point'});
 assert.equal(state.safePointDraft.stage,'idle');
 state=mapReducer(state,{type:'begin-safe-point'});
 state=mapReducer(state,{type:'choose-safe-position',position,altitude:null});
 state=mapReducer(state,{type:'confirm-safe-point'});
 assert.equal(state.safePointDraft.stage,'naming');
 state=mapReducer(state,{type:'name-safe-point',name:'source point'});
 state=mapReducer(state,{type:'confirm-safe-point'});
 assert.equal(state.safePointDraft.stage,'confirming');
 assert.equal(emptyNavigationData.safePoints.length,0);
});
test('multiple safe points support upsert rename and removal by stable ID',()=>{
 const point={id:'one',name:'one',position,altitude:null,distance:null,createdAt:'2026-10-27T10:00:00Z'};
 let points=safePointReducer([],{type:'upsert',point});
 points=safePointReducer(points,{type:'upsert',point:{...point,id:'two'}});
 points=safePointReducer(points,{type:'rename',id:'one',name:'renamed'});
 assert.equal(points.length,2);
 assert.equal(points.find(p=>p.id==='one').name,'renamed');
 points=safePointReducer(points,{type:'remove',id:'two'});
 assert.equal(points.length,1);
});
test('target without actual last seen location produces no marker',()=>{
 const target={id:'id',type:'PERSON',status:'LOST',label:'fixture',confidence:null,firstSeen:null,lastSeen:null};
 assert.equal(targetLastSeenMarker(target),null);
 const marker=targetLastSeenMarker({...target,lastSeenPosition:{...position,down:null}});
 assert.equal(marker.type,'LAST_SEEN');assert.equal(marker.targetId,'id');
 assert.equal(marker.altitude,null);
});
