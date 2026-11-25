import { useEffect, useState } from 'react';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { MapSelection } from './MapView';
import { priorTarget } from './priorMapSelection';
export function PriorMap({selection}:{selection:MapSelection|null}) {
 const [enabled,setEnabled]=useState(false),[open,setOpen]=useState(false),[message,setMessage]=useState('3D 실험 기능 비활성 · 도메인 및 권한 검증 필요');const target=priorTarget(selection);
 useEffect(()=>{if(!isTauri())return;let active=true;let stop:(()=>void)|undefined;let generation=0;
  void (async()=>{try{const available=await invoke<boolean>('map3d_available');if(!active||!available)return;stop=await listen('map3d-closed',()=>{if(active){generation++;setOpen(false);setMessage('3D 창 종료 · Prior Map');}});if(!active){stop();return;}const current=generation;const opened=await invoke<boolean>('map3d_is_open');if(!active)return;if(current===generation)setOpen(opened);setEnabled(true);setMessage('실험 기능 · 실제 지도와 권한 미검증');}catch{if(active)setMessage('3D 상태 확인 실패');}})();return()=>{active=false;stop?.();};
 },[]);
 async function start(){if(!target||!enabled||open)return;setOpen(true);setMessage('3D 창 여는 중 · 실제 데이터 미확인');try{await invoke('open_map3d',target);setMessage('3D 창 열림 · 지형 및 건물 수신 여부 미확인');}catch{setOpen(false);setMessage('3D 창 열기 실패 · 키, HTTPS 호스트 및 도메인 설정 확인');}}
 return <section className="prior-map"><div className="section-heading"><div><span className="eyebrow">PRIOR MAP</span><h2>VWorld 사전 3D 공간정보</h2></div><span>{enabled?'실험 · 미검증':'비활성'}</span></div><p>2D 지도에서 선택한 위치를 별도 3D 창으로 전달합니다. VWorld 데이터는 실시간 현장 정보가 아닙니다.</p>
 {target?<p>선택: {selection?.name||'이름 없음'} · 경도 {target.lon.toFixed(6)}, 위도 {target.lat.toFixed(6)} · 조회 반경 {target.radius} m</p>:<p>종합 관제 또는 임무 계획의 2D 지도에서 위치를 먼저 선택하세요.</p>}
 <button className="connection-button" disabled={!target||!enabled||open} onClick={()=>void start()}>선택 위치 3D 창 열기</button><p role="status">{message}</p><small>3D 데이터 제공 범위, HTTPS 인증 및 외부 WebView의 권한 격리는 확인되지 않았습니다. Combined View는 정합 검증 전까지 비활성입니다.</small></section>;
}
