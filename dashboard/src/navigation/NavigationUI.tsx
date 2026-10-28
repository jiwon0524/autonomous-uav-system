import { formatDateTime } from '../utils/format';
import { useEffect, useRef, useState } from 'react';
import type { Dispatch } from 'react';
import { MapPin, LocateFixed, X } from 'lucide-react';
import { Panel } from '../components/Panel';
import { riskLabels } from '../models';
import { navigationMarkers } from './model';
import type { MapState, MapAction, NavigationData, MapMarker } from './model';

export interface NavigationProps {data:NavigationData;state:MapState;dispatch:Dispatch<MapAction>;onMove?:()=>void}
const value=(number:number|null)=>number===null?'--':`${number} m`;
export function LocalMap({data,state,dispatch,compact=false}:NavigationProps & {compact?:boolean}){
  // A future map adapter supplies coordinates via select-destination / choose-safe-position.
  // The empty surface has no click-to-coordinate handler and no fake marker renderer.
  const markers=navigationMarkers(data);
  return <><div className={`map-view ${compact?'position-map':''}`} data-map-mode={state.mode} data-selected-marker-id={state.selectedMarkerId ?? undefined}><div className="empty-state"><div className="empty-icon"><MapPin size={28} strokeWidth={1.3}/></div><h3>지도</h3><p>데이터 없음</p></div><span className="map-label">{state.mode==='SIM_MAP'?'SIM MAP':'REAL MAP'} · 지도 소스 연결 대기 중</span></div>{markers.length>0 && <div className="marker-list" aria-label="지도 객체 목록">{markers.map(marker=><button key={marker.id} className={marker.obstacle?`obstacle-marker risk-${marker.obstacle.risk.toLowerCase()}`:undefined} aria-pressed={state.selectedMarkerId===marker.id} onClick={()=>dispatch({type:'select-marker',id:marker.id})}>{marker.name}<small>고도 {value(marker.altitude)} · 거리 {value(marker.distance)}</small></button>)}</div>}</>;
}
export function NavigationMap(props:NavigationProps){return <Panel title="로컬 지도 / 내비게이션" icon={MapPin} className="map-panel" extra={<LocateFixed size={15} className="muted"/>}><LocalMap {...props}/><dl className="navigation-data">{[['HOME',props.data.home?.name ?? '데이터 없음'],['SAFE POINT',props.data.safePoints.length?`${props.data.safePoints.length}개`:'데이터 없음'],['목표 위치',props.state.destination.stage==='empty'?'데이터 없음':'선택됨'],['경로',props.data.paths.length?'경로 데이터 있음':'데이터 없음']].map(([label,text])=><div key={label}><dt>{label}</dt><dd>{text}</dd></div>)}</dl><DestinationControls {...props}/></Panel>;}
export function DestinationControls({state,dispatch,onMove}:NavigationProps){
  const selection=state.destination;
  if(selection.stage==='empty')return null;
  return <div className="destination-confirm"><h3>목표 위치</h3><dl className="navigation-data"><div><dt>거리</dt><dd>{value(selection.destination.distance)}</dd></div><div><dt>고도</dt><dd>{value(selection.destination.altitude)}</dd></div></dl>{selection.stage==='selected'?<button onClick={()=>dispatch({type:'confirm-destination'})}>목표 정보 확인</button>:<><p>이 위치로 이동하시겠습니까?</p><button onClick={onMove} disabled={!onMove}>이 위치로 이동</button></>}<button onClick={()=>dispatch({type:'cancel-destination'})}>취소</button></div>;
}
function DetailDialog({title,onClose,children}: {title:string;onClose:()=>void;children:React.ReactNode}){
  const ref=useRef<HTMLDialogElement>(null);
  useEffect(()=>{const previous=document.activeElement as HTMLElement|null;ref.current?.showModal();return ()=>previous?.focus();},[]);
  return <dialog className="action-dialog navigation-dialog" ref={ref} onCancel={onClose}><header><h2>{title}</h2><button aria-label="닫기" onClick={onClose}><X size={18}/></button></header>{children}</dialog>;
}
export function MarkerDetail({marker,onClose}: {marker:MapMarker;onClose:()=>void}){
 const obstacle=marker.obstacle;
 const position=marker.position.frame==='LOCAL_NED'?`N ${marker.position.north} / E ${marker.position.east}`:`${marker.position.latitude}, ${marker.position.longitude}`;
 return <DetailDialog title={marker.name || '지도 객체 상세'} onClose={onClose}><dl className="target-detail-data">{[['종류',marker.type],['위치',position],['고도',value(marker.altitude)],['거리',value(marker.distance)],['상태',obstacle?riskLabels[obstacle.risk]:marker.status ?? '--'],...(obstacle?[['방향',obstacle.direction ?? '--'],['장애물 거리',value(obstacle.distance)],['감지 센서',obstacle.sensor ?? '--'],['최초 감지',formatDateTime(obstacle.firstDetected)],['최근 감지',formatDateTime(obstacle.lastDetected)],['경로 처리',obstacle.pathImpact?({UNKNOWN:'--',NONE:'영향 없음',BLOCKED:'경로 차단',REPLANNING:'재계획 중',NEW_PATH:'새 경로 생성'}[obstacle.pathImpact]):'--'],['경로 영향',obstacle.affectsPath===null?'--':obstacle.affectsPath?'있음':'없음']]:[])].map(([label,text])=><div key={label}><dt>{label}</dt><dd className={label==='상태' && obstacle?`risk-${obstacle.risk.toLowerCase()}`:''}>{text}</dd></div>)}</dl><footer><button onClick={onClose}>닫기</button></footer></DetailDialog>;
}
export function ReturnDestinationDialog({data,onClose,onConfirm}: {data:NavigationData;onClose:()=>void;onConfirm:(marker:MapMarker)=>void}){
  const choices=navigationMarkers(data).filter(marker=>marker.type==='HOME'||marker.type==='SAFE_POINT');
  const [id,setId]=useState('');
  const selected=choices.find(marker=>marker.id===id);
  return <DetailDialog title="복귀 위치 선택" onClose={onClose}>{choices.length?<fieldset className="return-options"><legend>복귀 목적지</legend>{choices.map(marker=><label key={marker.id}><input type="radio" name="return-point" value={marker.id} checked={id===marker.id} onChange={()=>setId(marker.id)}/>{marker.name}</label>)}</fieldset>:<p>사용 가능한 복귀 지점이 없습니다.</p>}<footer><button onClick={onClose}>취소</button><button className="blue-action" disabled={!selected} onClick={()=>selected && onConfirm(selected)}>안전 복귀</button></footer></DetailDialog>;
}
export function SafePointSetup({state,dispatch,onClose,onUnavailable}:NavigationProps & {onClose:()=>void;onUnavailable:()=>void}){
  const draft=state.safePointDraft;
  return <DetailDialog title="SAFE POINT 설정" onClose={onClose}>{draft.stage==='selecting'?<p>실제 지도에서 위치를 선택해 주세요. 지도 소스 연결 대기 중입니다.</p>:'name' in draft?<><label className="safe-point-name">이름<input maxLength={80} value={draft.name} onChange={event=>dispatch({type:'name-safe-point',name:event.target.value})}/></label>{draft.stage==='confirming' && <p>{draft.name}을 SAFE POINT로 설정하시겠습니까?</p>}</>:null}<footer><button onClick={onClose}>취소</button>{draft.stage==='naming' && <button disabled={!draft.name.trim()} onClick={()=>dispatch({type:'confirm-safe-point'})}>설정 정보 확인</button>}{draft.stage==='confirming' && <button onClick={onUnavailable}>확인</button>}</footer></DetailDialog>;
}
