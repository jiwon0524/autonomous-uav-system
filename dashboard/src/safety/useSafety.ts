import { useEffect, useMemo, useReducer } from 'react';
import { initialSafetyState, safetyReducer } from './model';
import type { HealthStatus, HealthSnapshot } from './model';
import type { ConnectionStatus } from '../useConnectionStatus';
import type { SensorSnapshot, SensorStatus } from '../sensors/model';
const sensorHealth=(status:SensorStatus):HealthStatus=>status==='CONNECTED'?'NORMAL':status;
/** Derived from existing real sources. No thresholds, fabricated events or flight commands. */
export function useSafety(connection:ConnectionStatus,sensors:SensorSnapshot){
 const [state,dispatch]=useReducer(safetyReducer,initialSafetyState);
 const health=useMemo<HealthSnapshot>(()=>({px4:connection.px4==='connected'?'NORMAL':'DISCONNECTED',ros2:connection.ros2==='connected'?'NORMAL':'DISCONNECTED',rgb:sensorHealth(sensors.rgb.status),depth:sensorHealth(sensors.depth.status),lidar:sensorHealth(sensors.lidar.status),position:state.health.position,battery:state.health.battery,link:state.health.link}),[connection.px4,connection.ros2,sensors.rgb.status,sensors.depth.status,sensors.lidar.status,state.health.position,state.health.battery,state.health.link]);
 useEffect(()=>{dispatch({type:'health',health,at:new Date().toISOString()});},[health]);
 return {state:{...state,health},dispatch};
}
