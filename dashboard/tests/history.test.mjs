import test from 'node:test';
import assert from 'node:assert/strict';
import { updateMissionNotes,filterMissions,missionTimeline } from '../src/history/model.ts';
// Test fixtures only; not imported into the dashboard.
const event={id:'fixture-event',timestamp:'2026-10-27T10:00:00Z',type:'SENSOR',message:'fixture',severity:'WARNING',metadata:null};
const record={id:'fixture',date:'2026-10-27',startTime:null,endTime:null,duration:null,missionType:'TEST_FLIGHT',maxAltitude:null,distance:null,detectedTargetCount:null,warningCount:null,targetLostCount:null,status:'COMPLETED',notes:'',events:[event],flightPath:[],targets:[],mapMarkers:[]};
test('empty records stay empty on note update; no mission is created',()=>{assert.deepEqual(updateMissionNotes([],'missing','note'),[]);});
test('note edits only the chosen existing record and preserves measurements',()=>{const original=[record,{...record,id:'other'}];const updated=updateMissionNotes(original,'fixture','observed');assert.equal(updated[0].notes,'observed');assert.equal(original[0].notes,'');assert.equal(updated[1],original[1]);assert.equal(updated[0].maxAltitude,null);});
test('timeline sorts actual timestamps and deduplicates stable event IDs',()=>{const events=missionTimeline([{...event,timestamp:'2026-10-27T10:01:00Z'},{...event,id:'earlier',timestamp:'2026-10-27T09:00:00Z'},event]);assert.equal(events.length,2);assert.equal(events[0].id,'earlier');assert.equal(events[1].timestamp,event.timestamp);assert.equal(missionTimeline(events,'PX4').length,0);});
test('filters support date type status target and warning events',()=>{assert.equal(filterMissions([record],{hasWarning:true}).length,1);assert.equal(filterMissions([record],{hasTargetEvent:true}).length,0);assert.equal(filterMissions([record],{status:'FAILED'}).length,0);assert.equal(filterMissions([record],{date:'2026-10-27',missionType:'TEST_FLIGHT'}).length,1);});
