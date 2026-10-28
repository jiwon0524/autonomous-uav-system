import type { RiskLevel } from '../models';
import type { Target } from '../targets/model';
export type MapMode = 'SIM_MAP' | 'REAL_MAP';
export type PanelMode = 'NAVIGATION' | 'POSITION_CONTROL';
export type MapPosition = {frame:'LOCAL_NED';north:number;east:number} | {frame:'WGS84';latitude:number;longitude:number};
export type MarkerType = 'UAV'|'HOME'|'SAFE_POINT'|'DESTINATION'|'TARGET'|'LAST_SEEN'|'OBSTACLE';
export interface ObstacleInfo { risk:RiskLevel; direction:string|null; distance:number|null; sensor:'Depth'|'LiDAR'|null; affectsPath:boolean|null;firstDetected?:string|null;lastDetected?:string|null;pathImpact?:'UNKNOWN'|'NONE'|'BLOCKED'|'REPLANNING'|'NEW_PATH' }
export interface MapMarker {id:string;type:MarkerType;name:string;position:MapPosition;altitude:number|null;distance:number|null;status:string|null;targetId?:string;obstacle?:ObstacleInfo}
export interface SafePoint {id:string;name:string;position:MapPosition;altitude:number|null;createdAt:string;distance:number|null}
export interface NavigationPath {id:string;type:'PLANNED'|'ACTUAL'|'REPLANNED';points:readonly MapPosition[]}
export interface NavigationData {sourceReady:boolean;uav:MapMarker|null;home:MapMarker|null;safePoints:readonly SafePoint[];markers:readonly MapMarker[];paths:readonly NavigationPath[]}
export const emptyNavigationData:NavigationData={sourceReady:false,uav:null,home:null,safePoints:[],markers:[],paths:[]};
export interface Destination {position:MapPosition;altitude:number|null;distance:number|null}
export type DestinationSelection={stage:'empty'}|{stage:'selected'|'confirming';destination:Destination};
export type SafePointDraft={stage:'idle'}|{stage:'selecting'}|{stage:'naming'|'confirming';position:MapPosition;altitude:number|null;name:string};
export interface MapState {mode:MapMode;panelMode:PanelMode;selectedMarkerId:string|null;destination:DestinationSelection;safePointDraft:SafePointDraft}
export const initialMapState:MapState={mode:'SIM_MAP',panelMode:'NAVIGATION',selectedMarkerId:null,destination:{stage:'empty'},safePointDraft:{stage:'idle'}};
export type MapAction = {type:'panel';mode:PanelMode}|{type:'mode';mode:MapMode}|{type:'select-marker';id:string|null}|{type:'select-destination';destination:Destination}|{type:'confirm-destination'}|{type:'cancel-destination'}|{type:'begin-safe-point'}|{type:'choose-safe-position';position:MapPosition;altitude:number|null}|{type:'name-safe-point';name:string}|{type:'confirm-safe-point'}|{type:'cancel-safe-point'};
/** UI only: confirmation never dispatches flight commands or invents positions. */
export function mapReducer(state:MapState,action:MapAction):MapState {
  switch(action.type){
    case 'panel':return {...state,panelMode:action.mode};
    case 'mode':return {...state,mode:action.mode};
    case 'select-marker':return {...state,selectedMarkerId:action.id};
    case 'select-destination':return {...state,destination:{stage:'selected',destination:action.destination}};
    case 'confirm-destination':return state.destination.stage==='selected'?{...state,destination:{...state.destination,stage:'confirming'}}:state;
    case 'cancel-destination':return {...state,destination:{stage:'empty'}};
    case 'begin-safe-point':return {...state,safePointDraft:{stage:'selecting'}};
    case 'choose-safe-position':return state.safePointDraft.stage==='selecting'?{...state,safePointDraft:{stage:'naming',position:action.position,altitude:action.altitude,name:''}}:state;
    case 'name-safe-point':return (state.safePointDraft.stage==='naming'||state.safePointDraft.stage==='confirming')?{...state,safePointDraft:{...state.safePointDraft,name:action.name,stage:'naming'}}:state;
    case 'confirm-safe-point':return state.safePointDraft.stage==='naming' && state.safePointDraft.name.trim()?{...state,safePointDraft:{...state.safePointDraft,stage:'confirming'}}:state;
    case 'cancel-safe-point':return {...state,safePointDraft:{stage:'idle'}};
  }
}
export type SafePointAction={type:'upsert';point:SafePoint}|{type:'rename';id:string;name:string}|{type:'remove';id:string};
export function safePointReducer(points:readonly SafePoint[],action:SafePointAction):readonly SafePoint[]{
  if(action.type==='upsert')return [...points.filter(p=>p.id!==action.point.id),action.point];
  if(action.type==='remove')return points.filter(p=>p.id!==action.id);
  return action.name.trim()?points.map(p=>p.id===action.id?{...p,name:action.name.trim()}:p):points;
}
export function navigationMarkers(data:NavigationData):MapMarker[]{return [...(data.uav?[data.uav]:[]),...(data.home?[data.home]:[]),...data.safePoints.map(point=>({...point,type:'SAFE_POINT' as const,status:'사용 가능'})),...data.markers];}
/** Future adapter: no position means no marker; coordinates are never guessed. */
export function targetLastSeenMarker(target:Target):MapMarker|null {
  const p=target.lastSeenPosition;
  if(!p || !['LOST','REACQUIRING'].includes(target.status))return null;
  return {id:`last-seen:${target.id}`,type:'LAST_SEEN',targetId:target.id,name:target.label,position:{frame:'LOCAL_NED',north:p.north,east:p.east},altitude:null,distance:null,status:target.status};
}
