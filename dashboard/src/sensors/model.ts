import type { MapMarker, ObstacleInfo, MapPosition } from '../navigation/model';
export type SensorId='rgb'|'depth'|'lidar';
export type SensorStatus='DISCONNECTED'|'CONNECTED'|'NO_DATA'|'WARNING'|'ERROR';
export const sensorLabels:Record<SensorId,string>={rgb:'RGB Camera',depth:'Depth Camera',lidar:'2D LiDAR'};
export const sensorStatusLabels:Record<SensorStatus,string>={DISCONNECTED:'연결 안 됨',CONNECTED:'연결됨',NO_DATA:'데이터 없음',WARNING:'주의',ERROR:'오류'};
export interface DataSource {kind:'frame'|'scan';adapterId:string;reference:string|null}
interface BaseSensor {status:SensorStatus;timestamp:string|null;source:DataSource|null;receiving:boolean|null}
export interface RgbSensorData extends BaseSensor {resolution:{width:number;height:number}|null;fps:number|null}
export interface TargetDistance {targetId:string;distanceMeters:number;timestamp:string}
export interface DepthSensorData extends BaseSensor {targetDistance:TargetDistance|null;minimumDistance:number|null;depthRange:{min:number;max:number}|null}
export interface ScanPoint {angleRadians:number;distanceMeters:number}
export interface LidarSensorData extends BaseSensor {minimumDistance:number|null;scanRange:{min:number;max:number}|null;scanPoints:readonly ScanPoint[]|null}
export interface SensorSnapshot {rgb:RgbSensorData;depth:DepthSensorData;lidar:LidarSensorData}
const empty={status:'DISCONNECTED' as const,timestamp:null,source:null,receiving:null};
export const initialSensors:SensorSnapshot={rgb:{...empty,resolution:null,fps:null},depth:{...empty,targetDistance:null,minimumDistance:null,depthRange:null},lidar:{...empty,minimumDistance:null,scanRange:null,scanPoints:null}};
export type SensorUpdate={id:'rgb';data:RgbSensorData}|{id:'depth';data:DepthSensorData}|{id:'lidar';data:LidarSensorData};
export function updateSensor(snapshot:SensorSnapshot,update:SensorUpdate):SensorSnapshot{return {...snapshot,[update.id]:update.data};}
export interface SensorEvent {id:string;sensorId:SensorId;from:SensorStatus;to:SensorStatus;timestamp:string}
export function sensorTransition(before:SensorSnapshot,update:SensorUpdate,timestamp:string):SensorEvent|null {
 const from=before[update.id].status,to=update.data.status;
 return from===to?null:{id:`${update.id}:${timestamp}:${to}`,sensorId:update.id,from,to,timestamp};
}
export function formatSensorEvent(event:SensorEvent):string{return `[SENSOR] ${sensorLabels[event.sensorId]} ${event.to.toLowerCase().replace('_',' ')}`;}
export function targetDepthDistance(depth:DepthSensorData,targetId:string):number|null {
 return depth.status==='CONNECTED' && depth.targetDistance?.targetId===targetId?depth.targetDistance.distanceMeters:null;
}
/** Only processed ROS2 obstacle output can be adapted; raw ScanPoint[] is deliberately not accepted. */
export interface ProcessedSensorObstacle {id:string;position:MapPosition;altitude:number|null;info:ObstacleInfo;timestamp:string}
export function processedObstacleMarker(obstacle:ProcessedSensorObstacle):MapMarker {
 return {id:obstacle.id,type:'OBSTACLE',name:'장애물',position:obstacle.position,altitude:obstacle.altitude,distance:obstacle.info.distance,status:obstacle.info.risk,obstacle:obstacle.info};
}
