import type { MapPosition, MapMarker } from '../navigation/model';
import type { TargetType, TargetStatus } from '../targets/model';
export type MissionType='MANUAL_CONTROL'|'POSITION_CONTROL'|'TARGET_TRACKING'|'PATROL'|'TEST_FLIGHT';
export type MissionStatus='IN_PROGRESS'|'COMPLETED'|'CANCELLED'|'FAILED';
export const missionTypeLabels:Record<MissionType,string>={MANUAL_CONTROL:'수동 운용',POSITION_CONTROL:'위치 제어',TARGET_TRACKING:'타겟 추적',PATROL:'자동 순찰',TEST_FLIGHT:'시험 비행'};
export const missionStatusLabels:Record<MissionStatus,string>={IN_PROGRESS:'진행 중',COMPLETED:'완료',CANCELLED:'중단',FAILED:'실패'};
export type MissionEventType='SYSTEM'|'PX4'|'ROS2'|'FLIGHT'|'DETECT'|'TARGET'|'TRACK'|'SENSOR'|'OBSTACLE'|'SAFETY'|'NAVIGATION';
export interface MissionEvent {id:string;timestamp:string|null;type:MissionEventType;message:string;severity:'NORMAL'|'WARNING'|'CRITICAL'|'ERROR'|null;metadata:Record<string,unknown>|null}
export interface FlightPathPoint {timestamp:string|null;position:MapPosition;altitude:number|null}
export interface MissionTargetRecord {targetId:string;type:TargetType;initialStatus:TargetStatus;finalStatus:TargetStatus;firstSeen:string|null;lastSeen:string|null}
export interface MissionMetrics {trackingDuration:number|null;reacquisitionTime:number|null;obstacleCount:number|null;replanningCount:number|null;completionTime:number|null}
export interface MissionRecord {
 id:string;date:string|null;startTime:string|null;endTime:string|null;duration:number|null;missionType:MissionType|null;maxAltitude:number|null;distance:number|null;detectedTargetCount:number|null;warningCount:number|null;targetLostCount:number|null;status:MissionStatus|null;notes:string;
 events:readonly MissionEvent[];flightPath:readonly FlightPathPoint[];targets:readonly MissionTargetRecord[];mapMarkers:readonly MapMarker[];metrics?:MissionMetrics;
}
export interface MissionFilter {date?:string;missionType?:MissionType;status?:MissionStatus;hasTargetEvent?:boolean;hasWarning?:boolean}
/** Future storage boundary. No endpoint, database, or lifecycle implementation is assumed. */
export interface MissionRepository {list(filter?:MissionFilter):Promise<readonly MissionRecord[]>;get(id:string):Promise<MissionRecord|null>;saveNotes(id:string,notes:string):Promise<void>}
/** UI-session cache only. No creation API: records must come from a future real adapter. */
export function updateMissionNotes(records:readonly MissionRecord[],id:string,notes:string):readonly MissionRecord[]{return records.map(record=>record.id===id?{...record,notes}:record);}
export function filterMissions(records:readonly MissionRecord[],filter:MissionFilter):readonly MissionRecord[]{return records.filter(record=>(!filter.date||record.date===filter.date)&&(!filter.missionType||record.missionType===filter.missionType)&&(!filter.status||record.status===filter.status)&&(filter.hasTargetEvent===undefined||record.events.some(event=>['DETECT','TARGET','TRACK'].includes(event.type))===filter.hasTargetEvent)&&(filter.hasWarning===undefined||(record.warningCount!==null?record.warningCount>0:record.events.some(event=>['WARNING','CRITICAL'].includes(event.severity ?? '')))===filter.hasWarning));}
export function missionTimeline(events:readonly MissionEvent[],type?:MissionEventType):readonly MissionEvent[]{
 const byId=new Map<string,MissionEvent>();for(const event of events)byId.set(event.id,event);
 return [...byId.values()].filter(event=>!type||event.type===type).sort((a,b)=>{const first=a.timestamp?Date.parse(a.timestamp):NaN,second=b.timestamp?Date.parse(b.timestamp):NaN;return (Number.isFinite(first)?first:Infinity)-(Number.isFinite(second)?second:Infinity);});
}
