import { stationShell } from '../platform/stationShell';
import { useSyncExternalStore } from 'react';
import { initialSensors, sensorTransition, updateSensor } from './model';
import type { SensorSnapshot, SensorUpdate, SensorEvent } from './model';
import { isSensorSnapshot, isSensorEvent, isSensorUpdate } from './validation';
interface SensorStoreSnapshot {sensors:SensorSnapshot;events:readonly SensorEvent[]}
let snapshot:SensorStoreSnapshot={sensors:initialSensors,events:[]};
let sequence=0;
const listeners=new Set<()=>void>();
const diagnostic=stationShell.currentView()==='sensors';
// UI synchronization only. Closing this channel never controls sensor processing.
let channel:BroadcastChannel|null=null;
try {if(typeof BroadcastChannel!=='undefined')channel=new BroadcastChannel('uav-sensor-ui-v1');} catch { /* Sandboxed shells may not support channels. */ }
function post(message:unknown){try{channel?.postMessage(message);}catch{/* Closed/unavailable channels must not break the local UI. */}}
const emit=()=>listeners.forEach(listener=>listener());
if(channel){
 channel.onmessage=(event:MessageEvent<unknown>)=>{
   const message=event.data;
   if(!message||typeof message!=='object'||!('type' in message))return;
   if(!diagnostic && message.type==='request')post({type:'snapshot',snapshot});
   if(diagnostic && message.type==='snapshot' && 'snapshot' in message){
     const incoming=message.snapshot;
     if(!incoming||typeof incoming!=='object'||!('sensors' in incoming)||!isSensorSnapshot(incoming.sensors)||!('events' in incoming)||!Array.isArray(incoming.events)||!incoming.events.every(isSensorEvent))return;
     snapshot={sensors:incoming.sensors,events:incoming.events.slice(-100)};emit();
   }
 };
 if(diagnostic)post({type:'request'});
}
/** Future main-window bridge adapter calls this. Sensor Monitor remains read-only. */
export function publishSensorUpdate(update:SensorUpdate):void {
 if(diagnostic||!isSensorUpdate(update,snapshot.sensors))return;
 const event=sensorTransition(snapshot.sensors,update,new Date().toISOString());
 const incoming=event?{...event,id:`${event.id}:${++sequence}`}:null;
 snapshot={sensors:updateSensor(snapshot.sensors,update),events:incoming?[...snapshot.events,incoming].slice(-100):snapshot.events};
 emit();post({type:'snapshot',snapshot});
}
const subscribe=(listener:()=>void)=>{listeners.add(listener);return ()=>{listeners.delete(listener);};};
export function useSensors(){return useSyncExternalStore(subscribe,()=>snapshot,()=>snapshot);}
if(import.meta.hot)import.meta.hot.dispose(()=>{channel?.close();listeners.clear();});
