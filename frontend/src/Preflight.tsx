import {useEffect,useState} from 'react';
import {useSafety,useTelemetry} from './Telemetry';
import {preflightObservations} from './preflightModel';

const manual=['조종기와 수신기 상태를 직접 확인','프로펠러와 기체 상태를 직접 확인','안전한 시험 환경을 직접 확인'];
export function Preflight() {
 const safety=useSafety(), telemetry=useTelemetry();
 const [checked,setChecked]=useState<boolean[]>(()=>manual.map(()=>false));
 useEffect(()=>{setChecked(manual.map(()=>false));},[safety.session,safety.source,safety.reachable,safety.connection]);
 return <section className="safety-section" aria-label="비행 전 점검">
  <div className="section-heading"><h2>비행 전 점검 · 참고용</h2></div>
  <p>관찰된 데이터와 현장 직접 확인을 구분합니다. PX4 arming 검사나 비행 허가가 아니며 실제 비행 명령은 잠겨 있습니다.</p>
  <h3>수신 정보</h3><ul>{preflightObservations(safety,telemetry,Date.now()).map(x=><li key={x.name}>{x.name} · <strong>{x.state}</strong></li>)}</ul>
  <h3>사용자 직접 확인</h3>{manual.map((item,i)=><label key={item} style={{display:'block',margin:'0.5rem 0'}}><input type="checkbox" checked={checked[i]} onChange={e=>setChecked(old=>old.map((x,n)=>n===i?e.target.checked:x))}/> {item}</label>)}
  <p>직접 확인 기록은 현재 화면에서만 유지되며 기체의 실제 상태를 확인하거나 명령을 활성화하지 않습니다.</p>
 </section>;
}
