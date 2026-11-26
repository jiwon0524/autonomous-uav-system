import type { MapMarker, MapPosition } from '../navigation/model';
export type HealthStatus='NORMAL'|'WARNING'|'CRITICAL'|'DISCONNECTED'|'NO_DATA'|'ERROR';
export const healthLabels:Record<HealthStatus,string>={NORMAL:'정상',WARNING:'주의',CRITICAL:'위험',DISCONNECTED:'연결 안 됨',NO_DATA:'데이터 없음',ERROR:'오류'};
export type DeviceId='px4'|'ros2'|'rgb'|'depth'|'lidar'|'position'|'battery'|'link';
export const deviceLabels:Record<DeviceId,string>={px4:'PX4',ros2:'ROS2',rgb:'RGB Camera',depth:'Depth Camera',lidar:'LiDAR',position:'위치',battery:'배터리',link:'통신'};
export type HealthSnapshot=Record<DeviceId,HealthStatus>;
export const initialHealth:HealthSnapshot={px4:'DISCONNECTED',ros2:'DISCONNECTED',rgb:'DISCONNECTED',depth:'DISCONNECTED',lidar:'DISCONNECTED',position:'NO_DATA',battery:'NO_DATA',link:'DISCONNECTED'};
export interface SafetyEvent {id:string;severity:'WARNING'|'CRITICAL';scope:'DEVICE'|'FLIGHT';cause:string;device:DeviceId|null;occurredAt:string;direction?:string}
export interface SafetyNotification extends SafetyEvent {dismissed:boolean}
export interface Obstacle {id:string;status:'NORMAL'|'WARNING'|'CRITICAL';direction:string|null;distance:number|null;source:'DEPTH'|'LIDAR'|'UNKNOWN';firstDetected:string|null;lastDetected:string|null;position:MapPosition|null;pathImpact:'UNKNOWN'|'NONE'|'BLOCKED'|'REPLANNING'|'NEW_PATH';altitude:number|null}
export interface FailsafeState {status:HealthStatus;reason:string;occurredAt:string|null}
export interface SafetyLog {id:number;text:string;occurredAt:string}
export interface SafetyState {health:HealthSnapshot;active:Record<string,SafetyEvent>;notifications:readonly SafetyNotification[];obstacles:Record<string,Obstacle>;failsafe:FailsafeState|null;logs:readonly SafetyLog[];sequence:number;seenEventIds:readonly string[]}
export const initialSafetyState:SafetyState={health:initialHealth,active:{},notifications:[],obstacles:{},failsafe:null,logs:[],sequence:0,seenEventIds:[]};
export type SafetyAction={type:'health';health:HealthSnapshot;at:string}|{type:'event';event:SafetyEvent}|{type:'clear-event';id:string;at:string}|{type:'dismiss';id:string}|{type:'obstacle';obstacle:Obstacle;at:string}|{type:'remove-obstacle';id:string;at:string}|{type:'failsafe';state:FailsafeState|null;at:string};
function log(state:SafetyState,text:string,at:string):SafetyState {const id=state.sequence+1;return {...state,sequence:id,logs:[...state.logs,{id,text,occurredAt:at}].slice(-200)};}
function notify(state:SafetyState,event:SafetyEvent):SafetyState {return {...state,notifications:[...state.notifications,{...event,dismissed:false}].slice(-100)};}
export function safetyReducer(state:SafetyState,action:SafetyAction):SafetyState {
 switch(action.type){
  case 'health':{
   let next={...state,health:action.health};
   for(const id of Object.keys(action.health) as DeviceId[]){
    const from=state.health[id],to=action.health[id];if(from===to)continue;
    next=log(next,`[${to==='WARNING'?'WARNING':to==='CRITICAL'?'SAFETY':'SYSTEM'}] ${deviceLabels[id]} ${to==='NORMAL' && ['WARNING','CRITICAL','ERROR'].includes(from)?'recovered':healthLabels[to]}`,action.at);
    if(to==='WARNING'||to==='CRITICAL')next=notify(next,{id:`health:${id}:${next.sequence}`,severity:to,scope:'DEVICE',device:id,cause:`${deviceLabels[id]} 상태: ${healthLabels[to]}`,occurredAt:action.at});
   }return next;
  }
  case 'event':{
   if(state.seenEventIds.includes(action.event.id))return state;
   const previous=Object.hasOwn(state.active,action.event.id)?state.active[action.event.id]:undefined;
   if(previous)return state;
   const next={...state,active:{...state.active,[action.event.id]:action.event},seenEventIds:[...state.seenEventIds,action.event.id].slice(-1000)};
   return notify(log(next,`[SAFETY] ${action.event.cause} ${action.event.severity}`,action.event.occurredAt),action.event);
  }
  case 'clear-event':{
   if(!Object.hasOwn(state.active,action.id))return state;
   const active={...state.active};delete active[action.id];
   return log({...state,active},state.active[action.id].severity==='CRITICAL'?'[SAFETY] Critical state cleared':'[SAFETY] Warning state cleared',action.at);
  }
  case 'dismiss':return {...state,notifications:state.notifications.map(n=>n.id===action.id?{...n,dismissed:true}:n)};
  case 'obstacle':{
   const old=Object.hasOwn(state.obstacles,action.obstacle.id)?state.obstacles[action.obstacle.id]:undefined;
   let next={...state,obstacles:{...state.obstacles,[action.obstacle.id]:action.obstacle}};
   if(!old || old.status!==action.obstacle.status){
    next=log(next,`[OBSTACLE] ${action.obstacle.id} ${old?healthLabels[action.obstacle.status]:'detected'}`,action.at);
    if(action.obstacle.status!=='NORMAL')next=notify(next,{id:`obstacle:${action.obstacle.id}:${next.sequence}`,severity:action.obstacle.status,scope:'DEVICE',device:null,cause:`장애물 ${action.obstacle.id}: ${healthLabels[action.obstacle.status]}`,occurredAt:action.at,direction:action.obstacle.direction ?? undefined});
   }return next;
  }
  case 'remove-obstacle':{
   if(!Object.hasOwn(state.obstacles,action.id))return state;
   const obstacles={...state.obstacles};delete obstacles[action.id];return log({...state,obstacles},`[OBSTACLE] ${action.id} cleared`,action.at);
  }
  case 'failsafe':{
   if(state.failsafe?.status===action.state?.status && state.failsafe?.reason===action.state?.reason)return state;
   return log({...state,failsafe:action.state},`[SAFETY] Failsafe ${action.state?.reason ?? 'cleared'}`,action.at);
  }
 }
}
export function healthSummary(health:HealthSnapshot):{status:HealthStatus;count:number}{
 const values=Object.values(health);
 for(const status of ['CRITICAL','ERROR','WARNING','DISCONNECTED','NO_DATA','NORMAL'] as const){const count=values.filter(value=>value===status).length;if(count)return {status,count};}
 return {status:'NO_DATA',count:0};
}
export function criticalFlightEvents(state:SafetyState):SafetyEvent[]{return Object.values(state.active).filter(event=>event.scope==='FLIGHT' && event.severity==='CRITICAL');}
export function obstacleMarker(obstacle:Obstacle):MapMarker|null {
 if(!obstacle.position)return null;
 return {id:obstacle.id,type:'OBSTACLE',name:'장애물 정보',position:obstacle.position,altitude:obstacle.altitude,distance:obstacle.distance,status:obstacle.status,obstacle:{risk:obstacle.status,direction:obstacle.direction,distance:obstacle.distance,sensor:obstacle.source==='DEPTH'?'Depth':obstacle.source==='LIDAR'?'LiDAR':null,affectsPath:obstacle.pathImpact==='UNKNOWN'?null:obstacle.pathImpact!=='NONE',firstDetected:obstacle.firstDetected,lastDetected:obstacle.lastDetected,pathImpact:obstacle.pathImpact}};
}
