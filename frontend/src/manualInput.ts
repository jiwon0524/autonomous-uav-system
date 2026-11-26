// Preview-only state machine: no transport, actuator or flight-control integration.
export type Mode = 'MONITOR' | 'MANUAL_PREVIEW' | 'MANUAL_CONTROL' | 'AUTONOMOUS';
export type Action = 'forward' | 'back' | 'left' | 'right' | 'up' | 'down' | 'yawLeft' | 'yawRight';
export const KEY_BINDINGS: Readonly<Record<string, Action>> = Object.freeze({KeyW:'forward',KeyS:'back',KeyA:'left',KeyD:'right',ArrowUp:'up',ArrowDown:'down',ArrowLeft:'yawLeft',ArrowRight:'yawRight'});
export const HOLD_TIMEOUT_MS = 2000;
export type PreviewState = {mode:Mode; held:Partial<Record<Action,number>>; blocked:Action[]; reason:string};
export type Event = {type:'enter';ready:boolean} | {type:'exit';reason:string} | {type:'down';action:Action;at:number;repeat:boolean} | {type:'up';action:Action} | {type:'clear'} | {type:'tick';at:number;ready:boolean};
export const initialPreview = ():PreviewState => ({mode:'MONITOR',held:{},blocked:[],reason:''});
export function previewReducer(state:PreviewState,event:Event):PreviewState {
  if(event.type==='exit') return {mode:'MONITOR',held:{},blocked:[],reason:event.reason};
  if(event.type==='enter' && state.mode!=='MONITOR') return state;
  if(event.type==='enter') return event.ready ? {mode:'MANUAL_PREVIEW',held:{},blocked:[],reason:''} : {mode:'MONITOR',held:{},blocked:[],reason:'연결 및 최신 비행 모드를 확인하세요'};
  if(state.mode!=='MANUAL_PREVIEW') return state;
  if(event.type==='tick') {
    if(!event.ready) return {mode:'MONITOR',held:{},blocked:[],reason:'연결 또는 데이터 최신성 상실'};
    if(Object.values(state.held).some(at=>event.at-at>=HOLD_TIMEOUT_MS)) return {mode:'MONITOR',held:{},blocked:[],reason:'입력 시간 초과'};
    return state;
  }
  if(event.type==='clear') return {...state,held:{},blocked:Array.from(new Set([...state.blocked,...Object.keys(state.held) as Action[]]))};
  if(event.type==='up') { const held={...state.held};delete held[event.action];return {...state,held,blocked:state.blocked.filter(a=>a!==event.action)}; }
  if(event.type==='down' && !event.repeat && !state.blocked.includes(event.action) && state.held[event.action]===undefined) return {...state,held:{...state.held,[event.action]:event.at}};
  return state;
}
export function axes(held:PreviewState['held']) { const on=(a:Action)=>held[a]!==undefined ? 1:0;return {forward:on('forward')-on('back'),right:on('right')-on('left'),up:on('up')-on('down'),yaw:on('yawRight')-on('yawLeft')}; }
export function isEditable(target:EventTarget|null):boolean { return target instanceof Element && !!target.closest('input,textarea,select,[contenteditable]:not([contenteditable="false"]),[role="textbox"]'); }
