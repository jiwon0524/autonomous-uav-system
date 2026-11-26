import { useEffect, useRef, useState } from 'react';
import { disconnectedStatus as disconnected, readBridgeStatus } from './services/bridgeStatus';
import type { ConnectionStatus, Connection } from './services/bridgeStatus';
export type { ConnectionStatus, Connection } from './services/bridgeStatus';
export interface StatusEvent {id: string | number; text: string}
export const connectionLabel = (status: Connection) => status === 'connected' ? '연결됨' : '연결 안 됨';
export function useConnectionStatus(enabled: boolean) {
  const [status,setStatus] = useState<ConnectionStatus>(disconnected);
  const [events,setEvents] = useState<StatusEvent[]>([]);
  const previous = useRef({...disconnected, bridge:false});
  const sequence = useRef(0);
  useEffect(()=>{
    if(!enabled)return;
    let stopped=false;
    let timer: ReturnType<typeof setTimeout>;
    let controller: AbortController;
    const poll = async () => {
      controller = new AbortController();
      const deadline = setTimeout(()=>controller.abort(),2000);
      let next = disconnected;
      let bridge = false;
      try {
        next=await readBridgeStatus(controller.signal);bridge=true;
      } catch { /* Unreachable or invalid API always means disconnected. */ }
      finally {clearTimeout(deadline);}
      if(stopped)return;
      const before=previous.current;
      const messages: string[]=[];
      if(bridge!==before.bridge)messages.push(`[SYSTEM] Dashboard bridge ${bridge?'connected':'disconnected'}`);
      if(next.ros2!==before.ros2)messages.push(`[ROS2] ${next.ros2==='connected'?'Connected':'Disconnected'}`);
      if(next.px4!==before.px4){
        messages.push(next.px4==='connected'?'[PX4] VehicleStatus received':bridge && next.ros2==='connected'?'[PX4] VehicleStatus timeout':'[PX4] VehicleStatus stream unavailable');
        messages.push(`[PX4] ${next.px4==='connected'?'Connected':'Disconnected'}`);
      }
      if(messages.length){
        const incoming=messages.map(text=>({id:++sequence.current,text}));
        setEvents(current=>[...current,...incoming].slice(-200));
      }
      previous.current={...next,bridge};setStatus(next);
      timer=setTimeout(poll,1000);
    };
    void poll();
    return ()=>{stopped=true;clearTimeout(timer);controller?.abort();};
  },[enabled]);
  return {status,events};
}
