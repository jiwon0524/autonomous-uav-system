import { LocalMap, DestinationControls } from '../navigation/NavigationUI';
import type { NavigationProps } from '../navigation/NavigationUI';
import { useEffect, useRef, useState } from 'react';
import { ArrowLeft, ArrowUp, ArrowDown, ArrowRight, MapPin, X } from 'lucide-react';
import { Panel } from './Panel';
import type { ActionHandler } from './DashboardPanels';

export interface Notice { title: string; message: string; confirmation?: boolean }
export function ActionModal({notice,onClose,onConfirm}: {notice: Notice; onClose:()=>void; onConfirm:()=>void}) {
  const dialog=useRef<HTMLDialogElement>(null);
  useEffect(()=>{const previous=document.activeElement as HTMLElement|null;dialog.current?.showModal();return ()=>previous?.focus();},[]);
  return <dialog ref={dialog} className="action-dialog" onCancel={onClose} onClick={e=>{if(e.target===e.currentTarget)onClose();}}><header><h2>{notice.title}</h2><button aria-label="닫기" onClick={onClose}><X size={18}/></button></header><p>{notice.message}</p><footer>{notice.confirmation && <button onClick={onClose}>취소</button>}<button className={notice.title==='비상 착륙'?'danger-action':'blue-action'} autoFocus onClick={notice.confirmation?onConfirm:onClose}>확인</button></footer></dialog>;
}

export function PositionControl({onBack,onAction,...navigation}: {onBack:()=>void;onAction:ActionHandler} & NavigationProps) {
  const [step,setStep]=useState('0.5');
  return <Panel title="위치 제어" icon={MapPin} className="map-panel position-panel" extra={<button className="text-button" onClick={onBack}>지도 보기</button>}><LocalMap {...navigation} compact/><div className="position-body"><h3>목표 위치</h3>{navigation.state.destination.stage==='empty' && <><p className="muted">데이터 없음 · 지도 연결 대기 중</p><button className="wide-button" disabled>목표 위치 확인</button></>}<DestinationControls {...navigation}/><div className="position-heading"><h3>미세 이동</h3><label>이동 단위 <select value={step} onChange={e=>setStep(e.target.value)}><option value="0.5">0.5m</option><option value="1">1m</option><option value="2">2m</option></select></label></div><div className="direction-grid"><button className="forward" onClick={()=>onAction('전진')}><ArrowUp size={14}/>전진</button><button onClick={()=>onAction('좌측 이동')}><ArrowLeft size={14}/>좌측</button><button onClick={()=>onAction('HOLD / 위치 유지')}>HOLD</button><button onClick={()=>onAction('우측 이동')}>우측<ArrowRight size={14}/></button><button className="backward" onClick={()=>onAction('후진')}><ArrowDown size={14}/>후진</button></div><h3>Yaw</h3><div className="control-grid"><button onClick={()=>onAction('Yaw 좌회전')}>↶ 좌회전</button><button onClick={()=>onAction('Yaw 우회전')}>우회전 ↷</button></div><details className="advanced-position"><summary>고급 위치 제어</summary><div className="advanced-fields">{['North','East','Altitude','Yaw'].map(label=><label key={label}>{label}<input type="number" placeholder="--" /></label>)}</div><button className="wide-button" onClick={()=>onAction('고급 위치 제어')}>위치 입력 확인</button></details></div></Panel>;
}

