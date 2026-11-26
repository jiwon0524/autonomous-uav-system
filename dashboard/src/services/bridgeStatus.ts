export type Connection='connected'|'disconnected';
export interface ConnectionStatus {ros2:Connection;px4:Connection}
export const disconnectedStatus:ConnectionStatus={ros2:'disconnected',px4:'disconnected'};
const valid=(value:unknown):value is Connection=>value==='connected'||value==='disconnected';
export function parseBridgeStatus(body:unknown):ConnectionStatus {
  if(!body || typeof body!=='object' || !('ros2' in body) || !('px4' in body) || !valid(body.ros2) || !valid(body.px4) || (body.px4==='connected' && body.ros2!=='connected'))throw new Error('Invalid bridge status');
  return {ros2:body.ros2,px4:body.px4};
}
/** Read-only service boundary. Relative URL is served by the configured bridge proxy. */
export async function readBridgeStatus(signal:AbortSignal):Promise<ConnectionStatus>{
  const response=await fetch('/api/status',{signal,cache:'no-store'});
  if(!response.ok)throw new Error('Bridge unavailable');
  return parseBridgeStatus(await response.json());
}
