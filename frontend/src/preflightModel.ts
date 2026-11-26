import type { SafetyView, View } from './Telemetry';
// Observations only; never an assertion of flight readiness.
export type Observation = {name:string; state:string};
export function preflightObservations(s:SafetyView, t:View, now:number):Observation[] {
  const available=t.connected && s.reachable===true && s.session>0 && s.source!=='OFFLINE' && s.connection==='INFORMATION';
  const live=(key:string,ttl:number)=>{
    const f=t.fields[key], age=now-(f?.timestamp_ms ?? 0);
    return !!f && f.status==='LIVE' && (f.value!==null || f.text!==null) && age>=0 && age<=ttl;
  };
  const heartbeat=available && live('armed',5000);
  const value=(v:string,valid:boolean)=>available && valid && v!=='UNKNOWN' ? v : '미확인';
  return [
    {name:'FC 연결 / HEARTBEAT',state:heartbeat?`${s.source} · 수신 중 · 출처 미인증`:'미확인'},
    {name:'기체 식별',state:heartbeat?`${s.source} · 세션에서 식별됨 · 인증되지 않음`:'미확인'},
    {name:'ARM 상태',state:value(s.armed,live('armed',5000))},
    {name:'센서 상태',state:value(s.sensors,['sensors_present','sensors_enabled','sensors_health'].every(k=>live(k,10000)))},
    {name:'배터리',state:value(s.battery,live('battery',10000))},
    {name:'GPS',state:value(s.gps,live('gps_fix',5000) && live('satellites',5000))},
    {name:'기체 위치',state:'별도 위치 확인 필요'},
    {name:'연결 품질',state:'지연·손실률 미측정'},
  ];
}
