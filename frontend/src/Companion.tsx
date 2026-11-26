import { useEffect, useState } from 'react';
import { invoke, isTauri } from '@tauri-apps/api/core';

type Message = { device_id:string; timestamp_ms:number; connection_state:string; cpu_usage_percent:number|null; memory_usage_percent:number|null; cpu_temperature_c:number|null; ai_status:string|null; camera_status:string|null; mapping_status:string|null; sensor_status:string|null; error_status:string|null };
type Status = { phase:string; endpoint:string|null; last_received_ms:number|null; message:Message|null; error:string|null };
const empty:Status = {phase:'DISCONNECTED',endpoint:null,last_received_ms:null,message:null,error:null};
const labels:Record<string,string> = { DISCONNECTED:'미연결', CONNECTING:'연결 중', WAITING:'상태 수신 대기', CONNECTED:'상태 수신 중', RECONNECTING:'재연결 중', LOST:'연결 손실' };
function useCompanion() {
  const [status,setStatus] = useState<Status>(empty);
  useEffect(()=>{ if (!isTauri()) return; let active=true, pending=false;
    const poll=async()=>{ if(pending)return; pending=true; try { const next=await invoke<Status>('companion_status'); if(active)setStatus(next); } catch { if(active)setStatus({...empty,error:'상태 조회 실패'}); } finally {pending=false;} };
    void poll(); const timer=window.setInterval(poll,1000); return ()=>{active=false;window.clearInterval(timer);};
  },[]);
  return status;
}
export function Companion({ settings=false, page }: {settings?:boolean;page?:string}) {
  const status=useCompanion(); const [ip,setIp]=useState(''); const [port,setPort]=useState(''); const [error,setError]=useState('');
  const live=status.phase==='CONNECTED'; const m=live?status.message:null;
  async function connect() { const value=Number(port); if(!ip.trim() || !Number.isInteger(value) || value<1 || value>65535) {setError('IP 주소와 1–65535 포트를 입력하세요.');return;} try {await invoke('companion_connect',{ip:ip.trim(),port:value});setError('');} catch(e){setError(String(e));} }
  async function disconnect() {try {await invoke('companion_disconnect');setError('');} catch(e){setError(String(e));}}
  return <section className="telemetry-section companion-panel"><div className="section-heading"><div><span className="eyebrow">COMPANION COMPUTER</span><h2>{settings?'Raspberry Pi 연결':`${page} 연동 상태`}</h2></div><span>{labels[status.phase]||'확인 불가'}</span></div>
    <p>Companion · {labels[status.phase]||'확인 불가'}{status.endpoint ? ` · ${status.endpoint}` : ''}</p>
    <p>출처 및 장치 ID 인증되지 않음 · 비행 제어와 독립</p>
    <p>장치: {m?.device_id ?? '미수신'} · 장치 상태: {m?.connection_state ?? '미수신'} · AI: {m?.ai_status ?? '미수신'} · 카메라: {m?.camera_status ?? '미수신'} · 매핑: {m?.mapping_status ?? '미수신'} · 센서: {m?.sensor_status ?? '미수신'}</p>
    <p>CPU: {m?.cpu_usage_percent != null ? `${m.cpu_usage_percent}%` : '미수신'} · 메모리: {m?.memory_usage_percent != null ? `${m.memory_usage_percent}%` : '미수신'} · 온도: {m?.cpu_temperature_c != null ? `${m.cpu_temperature_c}°C` : '미수신'}</p>
    <p>마지막 수신: {status.last_received_ms ? new Date(status.last_received_ms).toLocaleString() : '없음'}{!live && status.last_received_ms ? ' (이전 연결)' : ''}</p>
    {m?.error_status && <p role="status">장치 오류: {m.error_status}</p>}{(error||status.error) && <p role="alert">{error||status.error}</p>}
    {settings && <><div className="companion-form"><label>IP 주소<input value={ip} onChange={e=>setIp(e.target.value)} placeholder="예: 192.168.1.10" autoComplete="off"/></label><label>포트<input value={port} onChange={e=>setPort(e.target.value)} inputMode="numeric" placeholder="예: 8765" autoComplete="off"/></label><button type="button" onClick={()=>void connect()} disabled={!isTauri()}>연결</button><button type="button" onClick={()=>void disconnect()} disabled={!isTauri()}>연결 해제</button></div><small>사용자가 지정한 주소에만 접속합니다. 암호화·인증이 없으므로 신뢰할 수 있는 격리망에서만 시험하세요.</small></>}
    {!settings && <small>실제 {page} 기능은 개발 예정입니다. 수신 상태는 장치가 주장한 값입니다.</small>}
  </section>;
}
