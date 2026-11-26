import { formatDateTime as dateLabel } from '../utils/format';
import { useEffect, useRef, useState } from 'react';
import { Crosshair, ScanLine, ShieldCheck, X } from 'lucide-react';
import { Panel } from '../components/Panel';
import { targetStatusLabels, targetTypeLabels } from './model';
import type { Target, TargetFilter } from './model';

export function TargetList({variant,targets,selectedId,onSelect}: {
  variant:'authorized'|'unknown';targets:readonly Target[];selectedId:string|null;onSelect:(id:string)=>void;
}) {
  const [filter,setFilter]=useState<TargetFilter>('ALL');
  const authorized=variant==='authorized';
  const group=targets.filter(target=>(target.status==='AUTHORIZED')===authorized);
  const visible=group.filter(target=>filter==='ALL'||target.type===filter);
  return <Panel title={authorized?'허용 대상':'미확인 / 추적 대상'} icon={authorized?ShieldCheck:ScanLine} className={`target-panel ${variant}`} extra={<span className="target-indicator"/>}>
    {group.length>0 && <div className="target-filter"><label>유형 <select value={filter} onChange={event=>setFilter(event.target.value as TargetFilter)}><option value="ALL">전체</option>{Object.entries(targetTypeLabels).map(([type,label])=><option key={type} value={type}>{label}</option>)}</select></label>{!authorized && <span>미확인은 위험 확정을 뜻하지 않습니다.</span>}</div>}
    <div className="target-list" tabIndex={visible.length?0:undefined} aria-label={authorized?'허용 대상 목록':'미확인 / 추적 대상 목록'}>
      {visible.length?visible.map(target=><button type="button" className={`target-row target-status-${target.status.toLowerCase()} ${selectedId===target.id?'selected':''}`} key={target.id} onClick={()=>onSelect(target.id)} aria-pressed={selectedId===target.id}><span><strong>{target.label || `${target.type} #${target.id}`}</strong><small>{target.type} · {target.id}</small></span><span className="target-status"><i className="dot"/>{targetStatusLabels[target.status]}</span></button>):<div className="target-empty"><Crosshair size={23} strokeWidth={1.3}/><strong>{group.length?'필터에 맞는 대상 없음':authorized?'등록된 대상 없음':'탐지된 대상 없음'}</strong><p>대상 데이터 없음</p></div>}
    </div>
  </Panel>;
}

export function TargetDetail({target,distance=null,onClose,onCamera,onTrack}: {target:Target;distance?:number|null;onClose:()=>void;onCamera:()=>void;onTrack:()=>void}) {
  const dialog=useRef<HTMLDialogElement>(null);
  useEffect(()=>{const previous=document.activeElement as HTMLElement|null;dialog.current?.showModal();return ()=>previous?.focus();},[]);
  const confidence=target.confidence!==null && Number.isFinite(target.confidence)?`${Math.round(Math.min(1,Math.max(0,target.confidence))*100)} %`:'--';
  return <dialog ref={dialog} className="action-dialog target-detail" onCancel={onClose} onClick={event=>{if(event.target===event.currentTarget)onClose();}}><header><h2>{target.label || `${target.type} #${target.id}`}</h2><button onClick={onClose} aria-label="대상 상세 닫기"><X size={18}/></button></header><dl className="target-detail-data">{[['ID',target.id],['상태',targetStatusLabels[target.status]],['유형',target.type],['신뢰도',confidence],['거리',distance===null?'--':`${distance} m`],['최초 탐지',dateLabel(target.firstSeen)],['최근 탐지',dateLabel(target.lastSeen)]].map(([label,value])=><div key={label}><dt>{label}</dt><dd>{value}</dd></div>)}{['LOST','REACQUIRING'].includes(target.status) && <><div><dt>마지막 탐지 시간</dt><dd>{dateLabel(target.lastSeen)}</dd></div><div><dt>마지막 탐지 위치</dt><dd>{target.lastSeenPosition?`N ${target.lastSeenPosition.north} / E ${target.lastSeenPosition.east} / D ${target.lastSeenPosition.down ?? '--'} (${target.lastSeenPosition.frame})`:'--'}</dd></div></>}{target.reacquiredAt && <div><dt>목표물 재획득</dt><dd>{dateLabel(target.reacquiredAt)}</dd></div>}</dl>{target.status==='UNKNOWN' && <p className="target-explanation">미확인은 허용 여부가 확인되지 않은 상태이며 위험 확정을 뜻하지 않습니다.</p>}<footer><button onClick={onCamera}>카메라에서 확인</button><button className="blue-action" disabled={target.status!=='UNKNOWN'} onClick={onTrack}>추적 시작</button></footer></dialog>;
}
