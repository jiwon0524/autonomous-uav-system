import { useEffect, useReducer, useRef, useState } from 'react';
import { useSafety, useTelemetry } from './Telemetry';
import { axes, isEditable, KEY_BINDINGS, initialPreview, previewReducer, type Action } from './manualInput';
const keys:[string,string,Action][]=[['W','전진','forward'],['S','후진','back'],['A','좌측 평행 이동','left'],['D','우측 평행 이동','right'],['↑','상승','up'],['↓','하강','down'],['←','좌회전','yawLeft'],['→','우회전','yawRight']];
export function ManualPreview() {
 const telemetry=useTelemetry(), safety=useSafety(); const [state,dispatch]=useReducer(previewReducer,undefined,initialPreview);const [manual,setManual]=useState(true);const [move,setMove]=useState(2);const [turn,setTurn]=useState(2);
 const [offline,setOffline]=useState(false); const offlineRef=useRef(false); offlineRef.current=offline;
 const mode=telemetry.fields.mode; const latest=useRef({connected:false,at:0,live:false,reachable:false});latest.current={connected:telemetry.connected,at:mode?.timestamp_ms??0,live:mode?.status==='LIVE',reachable:safety.reachable===true};
 const ready=()=>{const s=latest.current;const age=Date.now()-s.at;return s.connected&&s.reachable&&s.live&&age>=0&&age<3000&&document.visibilityState==='visible'&&document.hasFocus();};
 const stateRef=useRef(state);stateRef.current=state;
 useEffect(()=>{
  const down=(e:KeyboardEvent)=>{if(stateRef.current.mode!=='MANUAL_PREVIEW')return;if(e.key==='Escape'){dispatch({type:'exit',reason:'사용자 종료'});return;}if(e.ctrlKey||e.metaKey||e.altKey||e.shiftKey||isEditable(e.target))return;if(e.code==='Space'){e.preventDefault();if(!e.repeat)dispatch({type:'clear'});return;}const a=KEY_BINDINGS[e.code];if(a){e.preventDefault();dispatch({type:'down',action:a,at:performance.now(),repeat:e.repeat});}};
  const up=(e:KeyboardEvent)=>{const a=KEY_BINDINGS[e.code];if(a)dispatch({type:'up',action:a});};
  const blur=()=>{if(stateRef.current.mode==='MANUAL_PREVIEW')dispatch({type:'exit',reason:'창 포커스 상실'});};const visibility=()=>{if(stateRef.current.mode==='MANUAL_PREVIEW' && document.visibilityState!=='visible')dispatch({type:'exit',reason:'화면 숨김 또는 최소화'});};
  const focus=(e:FocusEvent)=>{if(stateRef.current.mode==='MANUAL_PREVIEW'&&isEditable(e.target))dispatch({type:'exit',reason:'문자 입력으로 전환'});};
  window.addEventListener('keydown',down);window.addEventListener('keyup',up);window.addEventListener('blur',blur);document.addEventListener('visibilitychange',visibility);document.addEventListener('focusin',focus);
  const timer=window.setInterval(()=>{if(stateRef.current.mode==='MANUAL_PREVIEW')dispatch({type:'tick',at:performance.now(),ready:offlineRef.current ? document.visibilityState==='visible' && document.hasFocus() : ready()});},100);
  return()=>{window.clearInterval(timer);window.removeEventListener('keydown',down);window.removeEventListener('keyup',up);window.removeEventListener('blur',blur);document.removeEventListener('visibilitychange',visibility);document.removeEventListener('focusin',focus);};
 },[]);
 const active=state.mode==='MANUAL_PREVIEW', a=axes(state.held);
 return <section className="manual-preview" aria-label="키보드 조종 미리보기"><div className="manual-head"><div><span className="eyebrow">CONTROL MODE</span><h2>{active?'MANUAL PREVIEW':'MONITOR'}</h2></div><button className="connection-button" onClick={()=>active?dispatch({type:'exit',reason:'사용자 종료'}):(setOffline(false),setManual(true),dispatch({type:'enter',ready:ready()}))}>{active?'조종 모드 종료':'키보드 조종 모드'}</button></div>
 <button className="connection-button" disabled={active && !offline} onClick={()=>{if(active){dispatch({type:'exit',reason:'사용자 종료'});setOffline(false);}else{setOffline(true);setManual(true);dispatch({type:'enter',ready:document.visibilityState==='visible' && document.hasFocus()});}}}>{active && offline?'오프라인 연습 OFF':'오프라인 입력 연습 ON'}</button>
 {state.reason&&<p role="status">{state.reason}</p>}
 {active&&<>{offline && <p>오프라인 입력 연습 · 기체 상태와 분리된 키 입력 표시</p>}<p>입력 미리보기만 제공 · 실제 비행 명령 잠김 · QGroundControl 제어권 확인 불가</p>{!offline && <p>연결 {telemetry.connected?'수신 중':'미연결'} · 비행 모드 {mode?.status==='LIVE'?(mode.text??'확인 불가'):'확인 불가'} · 안전 감시 {safety.reachable?safety.overall:'확인 불가'}</p>}<div className="manual-speeds"><span>이동 속도 미리보기 {move}/5 <button onClick={()=>setMove(Math.max(1,move-1))} aria-label="이동 속도 낮추기">−</button><button onClick={()=>setMove(Math.min(5,move+1))} aria-label="이동 속도 높이기">+</button></span><span>회전 속도 미리보기 {turn}/5 <button onClick={()=>setTurn(Math.max(1,turn-1))} aria-label="회전 속도 낮추기">−</button><button onClick={()=>setTurn(Math.min(5,turn+1))} aria-label="회전 속도 높이기">+</button></span></div><button className="manual-toggle" aria-expanded={manual} onClick={()=>setManual(!manual)}>키보드 매뉴얼 {manual?'접기':'펼치기'}</button>{manual&&<div className="manual-keys">{keys.map(([key,label,action])=><span key={action} className={state.held[action]!==undefined?'active':''}><kbd>{key}</kbd> {label}</span>)}<small>Space: 입력 초기화 · Escape: 모드 종료</small></div>}<p>입력 합계 · 전후 {a.forward} · 좌우 평행 {a.right} · 상승 {a.up} · 회전 {a.yaw}</p><small>입력 초기화는 기체 정지·위치 유지·모터 정지를 보장하지 않습니다. 실제 수동 제어와 자율비행 실행은 비활성입니다.</small></>}
 </section>;
}
