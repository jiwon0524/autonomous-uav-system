export type TargetType = 'PERSON' | 'VEHICLE' | 'OBJECT';
export type TargetStatus = 'AUTHORIZED' | 'UNKNOWN' | 'TRACKING' | 'LOST' | 'REACQUIRING';
export interface LastSeenPosition {
  frame: 'LOCAL_NED';
  north: number;
  east: number;
  down: number | null;
}
export interface Target {
  /** Stable tracking ID supplied by a future detection adapter. */
  id: string;
  type: TargetType;
  status: TargetStatus;
  label: string;
  /** 0..1, or null when unavailable. */
  confidence: number | null;
  /** ISO-8601 timestamps supplied by the data source; never synthesized by the UI. */
  firstSeen: string | null;
  lastSeen: string | null;
  lastSeenPosition?: LastSeenPosition | null;
  /** Optional source-provided confirmation of LOST → REACQUIRING → TRACKING. */
  reacquiredAt?: string | null;
}
export const targetStatusLabels: Record<TargetStatus,string> = {
  AUTHORIZED:'허용 대상', UNKNOWN:'미확인', TRACKING:'추적 중', LOST:'목표물 놓침', REACQUIRING:'재탐색 중',
};
export const targetTypeLabels: Record<TargetType,string> = {PERSON:'사람',VEHICLE:'차량',OBJECT:'기타'};
export type TargetFilter = 'ALL' | TargetType;
export interface TargetState {
  byId: Record<string, Target>;
  selectedId: string | null;
  cameraTargetId: string | null;
  trackingRequestedId: string | null;
}
export const initialTargetState: TargetState = {byId:{},selectedId:null,cameraTargetId:null,trackingRequestedId:null};
export type TargetAction =
  | {type:'upsert'; targets: readonly Target[]}
  | {type:'select'; id:string | null}
  | {type:'inspect-camera'; id:string}
  | {type:'request-tracking'; id:string}
  | {type:'clear-tracking-request'};

/** State updates only. No trackers, publishers, network calls or simulated transitions. */
export function targetReducer(state:TargetState, action:TargetAction):TargetState {
  switch(action.type) {
    case 'upsert': {
      let byId={...state.byId};
      for(const incoming of action.targets) {
        if(!incoming.id.trim())continue;
        const old=Object.hasOwn(byId,incoming.id)?byId[incoming.id]:undefined;
        // Ignore out-of-order frames when both timestamps are known.
        if(old?.lastSeen && incoming.lastSeen && Date.parse(incoming.lastSeen)<Date.parse(old.lastSeen))continue;
        byId={...byId,[incoming.id]:{...old,...incoming,firstSeen:old?.firstSeen ?? incoming.firstSeen}};
      }
      return {...state,byId};
    }
    case 'select': return action.id===null || Object.hasOwn(state.byId,action.id)?{...state,selectedId:action.id}:state;
    case 'inspect-camera': return Object.hasOwn(state.byId,action.id)?{...state,cameraTargetId:action.id}:state;
    case 'request-tracking': {
      const target=state.byId[action.id];
      // Request intent is distinct from backend-confirmed TRACKING status.
      return target?.status==='UNKNOWN'?{...state,trackingRequestedId:action.id}:state;
    }
    case 'clear-tracking-request': return {...state,trackingRequestedId:null};
  }
}
export type TargetEventType = 'DETECTED' | 'SELECTED' | 'TRACKING' | 'LOST' | 'REACQUIRING' | 'REACQUIRED';
export interface TargetEvent { id:string; targetId:string; type:TargetEventType; occurredAt:string }
const eventLabels: Record<TargetEventType,string> = {DETECTED:'detected',SELECTED:'selected',TRACKING:'tracking',LOST:'lost',REACQUIRING:'reacquiring',REACQUIRED:'reacquired'};
/** Future console adapter. Defining an event never emits one. */
export function formatTargetEvent(event:TargetEvent):string {
  const channel=event.type==='DETECTED'?'DETECT':event.type==='SELECTED'?'TARGET':'TRACK';
  return `[${channel}] ${event.targetId} ${eventLabels[event.type]}`;
}
