import test from 'node:test';
import assert from 'node:assert/strict';
import { initialSensors, updateSensor, sensorTransition, targetDepthDistance, processedObstacleMarker, formatSensorEvent } from '../src/sensors/model.ts';
// Fixtures are isolated from the application and never published to its store.
test('sensor defaults do not fabricate frames scans or metadata',()=>{
 for(const sensor of Object.values(initialSensors)){assert.equal(sensor.status,'DISCONNECTED');assert.equal(sensor.source,null);assert.equal(sensor.timestamp,null);}
 assert.equal(initialSensors.rgb.fps,null);assert.equal(initialSensors.rgb.resolution,null);assert.equal(initialSensors.lidar.scanPoints,null);
});
test('sensor failure affects only that sensor',()=>{
 const update={id:'depth',data:{...initialSensors.depth,status:'WARNING'}};
 const state=updateSensor(initialSensors,update);
 assert.equal(state.depth.status,'WARNING');assert.equal(state.rgb,initialSensors.rgb);assert.equal(state.lidar,initialSensors.lidar);
});
test('identical status produces no console event, metadata changes do not repeat events',()=>{
 const update={id:'rgb',data:{...initialSensors.rgb,status:'CONNECTED'}};
 const event=sensorTransition(initialSensors,update,'2026-10-27T10:00:00Z');
 assert.equal(formatSensorEvent(event),'[SENSOR] RGB Camera connected');
 const state=updateSensor(initialSensors,update);
 assert.equal(sensorTransition(state,{...update,data:{...update.data,fps:1}},'2026-10-27T10:00:01Z'),null);
});
test('depth distance requires matching target and connected depth',()=>{
 const depth={...initialSensors.depth,status:'CONNECTED',targetDistance:{targetId:'fixture',distanceMeters:2,timestamp:'2026-10-27T10:00:00Z'}};
 assert.equal(targetDepthDistance(depth,'other'),null);assert.equal(targetDepthDistance(depth,'fixture'),2);
 assert.equal(targetDepthDistance({...depth,status:'DISCONNECTED'},'fixture'),null);
});
test('only processed obstacle data is adapted; raw scan is not converted',()=>{
 assert.equal(initialSensors.lidar.scanPoints,null);
 const marker=processedObstacleMarker({id:'processed-fixture',position:{frame:'LOCAL_NED',north:1,east:2},altitude:null,info:{risk:'WARNING',direction:null,distance:null,sensor:'LiDAR',affectsPath:null},timestamp:'2026-10-27T10:00:00Z'});
 assert.equal(marker.type,'OBSTACLE');assert.equal(marker.distance,null);
});
