import { formatDateTime } from '../utils/format';
import { TriangleAlert, X } from 'lucide-react';
import { criticalFlightEvents, deviceLabels, healthLabels } from './model';
import type { SafetyState, FailsafeState } from './model';
export function SafetyOverlay({state,onDismiss}: {state:SafetyState;onDismiss:(id:string)=>void}){
 const critical=criticalFlightEvents(state);
 const notifications=state.notifications.filter(n=>!n.dismissed).slice(-4);
 return <>{critical.length>0 && <><div className="safety-critical-border" aria-hidden="true"/><div className="safety-banner" role="alert"><TriangleAlert size={15}/><strong>CRITICAL</strong><span>{critical[0].cause}{critical[0].direction?` · ${critical[0].direction}`:''}</span>{critical.length>1 && <small>외 {critical.length-1}건</small>}</div></>}<aside className="safety-notifications" aria-label="안전 알림" aria-live="polite">{notifications.map(n=><article className={`safety-notification health-${n.severity.toLowerCase()}`} key={n.id}><header><strong>{healthLabels[n.severity]}</strong><button aria-label={`${n.cause} 알림 닫기`} onClick={()=>onDismiss(n.id)}><X size={14}/></button></header><b>{n.device?deviceLabels[n.device]:n.scope==='FLIGHT'?'비행 안전':'장애물'}</b><p>{n.cause}</p><time dateTime={n.occurredAt}>{formatDateTime(n.occurredAt)}</time></article>)}</aside></>;
}
export function FailsafeStatus({state}: {state:FailsafeState|null}){return <div className="failsafe-status"><span>FAILSAFE</span><strong className={state?`health-${state.status.toLowerCase()}`:''}>{state?.reason ?? '상태 없음'}</strong></div>;}
