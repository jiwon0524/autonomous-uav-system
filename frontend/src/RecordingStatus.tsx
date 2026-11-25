import { memo, useSyncExternalStore } from 'react';
import { invoke, isTauri } from '@tauri-apps/api/core';
export type Status={active:boolean;id:string|null;started_ms:number|null;bytes:number;error:string|null};
const emptyStatus:Status={active:false,id:null,started_ms:null,bytes:0,error:null};
let current=emptyStatus, timer:number|undefined, generation=0, pending=false;
const listeners=new Set<()=>void>();
export function subscribeRecordingStatus(listener:()=>void) {
  listeners.add(listener);
  if(listeners.size===1 && isTauri()) {
    const token=++generation;
    const poll=async()=>{if(pending)return;pending=true;let deadline:number|undefined;try {const state=await Promise.race([invoke<Status>('recording_status'),new Promise<never>((_,reject)=>{deadline=window.setTimeout(()=>reject(new Error('상태 조회 시간 초과')),1500);})]);if(token===generation)current=state;} catch {if(token===generation)current={...emptyStatus,error:'기록 상태 확인 실패'};} finally {if(deadline!==undefined)window.clearTimeout(deadline);pending=false;if(token===generation)listeners.forEach(fn=>fn());}};
    void poll();timer=window.setInterval(()=>void poll(),1000);
  }
  return ()=>{listeners.delete(listener);if(!listeners.size){generation++;if(timer!==undefined)window.clearInterval(timer);timer=undefined;current=emptyStatus;}};
}
export function recordingStatusSnapshot() { return current; }
export function useRecordingStatus(){return useSyncExternalStore(subscribeRecordingStatus,()=>current,()=>emptyStatus);}
function duration(ms:number){const n=Math.max(0,Math.floor(ms/1000));return `${Math.floor(n/3600).toString().padStart(2,'0')}:${Math.floor(n/60%60).toString().padStart(2,'0')}:${(n%60).toString().padStart(2,'0')}`;}
export const RecordingBadge=memo(function RecordingBadge(){const status=useRecordingStatus();return status.active?<div className="recording-badge" role="status">● 기록 중 {duration(Date.now()-(status.started_ms||Date.now()))}</div>:status.error?<div className="recording-badge" role="alert">기록 오류</div>:null;});
