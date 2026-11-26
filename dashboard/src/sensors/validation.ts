import type { SensorSnapshot, SensorUpdate, SensorStatus, SensorEvent } from './model';
const record=(value:unknown):value is Record<string,unknown>=>!!value && typeof value==='object' && !Array.isArray(value);
const status=(value:unknown):value is SensorStatus=>typeof value==='string' && ['DISCONNECTED','CONNECTED','NO_DATA','WARNING','ERROR'].includes(value);
const nonnegative=(value:unknown)=>typeof value==='number' && Number.isFinite(value) && value>=0;
const nullableNumber=(value:unknown)=>value===null||nonnegative(value);
const timestamp=(value:unknown)=>value===null||(typeof value==='string' && Number.isFinite(Date.parse(value)));
function range(value:unknown):boolean{return value===null||(record(value)&&nonnegative(value.min)&&nonnegative(value.max)&&(value.min as number)<=(value.max as number));}
function base(value:unknown):value is Record<string,unknown>{
 if(!record(value)||!status(value.status)||!timestamp(value.timestamp)||!(value.receiving===null||typeof value.receiving==='boolean'))return false;
 const source=value.source;
 return source===null||(record(source)&&['frame','scan'].includes(String(source.kind))&&typeof source.adapterId==='string'&&(source.reference===null||typeof source.reference==='string'));
}
export function isSensorSnapshot(value:unknown):value is SensorSnapshot {
 if(!record(value)||!base(value.rgb)||!base(value.depth)||!base(value.lidar))return false;
 const rgb=value.rgb,depth=value.depth,lidar=value.lidar;
 if(!nullableNumber(rgb.fps))return false;
 if(rgb.resolution!==null && (!record(rgb.resolution)||!Number.isInteger(rgb.resolution.width)||!Number.isInteger(rgb.resolution.height)||(rgb.resolution.width as number)<=0||(rgb.resolution.height as number)<=0))return false;
 const distance=depth.targetDistance;
 if(distance!==null&&(!record(distance)||typeof distance.targetId!=='string'||!nonnegative(distance.distanceMeters)||typeof distance.timestamp!=='string'||!timestamp(distance.timestamp)))return false;
 if(!nullableNumber(depth.minimumDistance)||!range(depth.depthRange)||!nullableNumber(lidar.minimumDistance)||!range(lidar.scanRange))return false;
 return lidar.scanPoints===null||(Array.isArray(lidar.scanPoints)&&lidar.scanPoints.every(point=>record(point)&&typeof point.angleRadians==='number'&&Number.isFinite(point.angleRadians)&&nonnegative(point.distanceMeters)));
}
export function isSensorUpdate(value:unknown,current:SensorSnapshot):value is SensorUpdate {
 return record(value)&&['rgb','depth','lidar'].includes(String(value.id))&&isSensorSnapshot({...current,[String(value.id)]:value.data});
}
export function isSensorEvent(value:unknown):value is SensorEvent {
 return record(value)&&typeof value.id==='string'&&['rgb','depth','lidar'].includes(String(value.sensorId))&&status(value.from)&&status(value.to)&&typeof value.timestamp==='string'&&timestamp(value.timestamp);
}
