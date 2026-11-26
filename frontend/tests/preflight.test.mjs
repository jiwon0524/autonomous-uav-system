import {strict as assert} from 'node:assert';
import {preflightObservations} from '../src/preflightModel.ts';
const base={reachable:true,session:1,source:'SIMULATION',connection:'INFORMATION',armed:'DISARMED',sensors:'NORMAL',battery:'NORMAL',gps:'NORMAL'};
const now=100000;
const fields=Object.fromEntries(['armed','sensors_present','sensors_enabled','sensors_health','battery','gps_fix','satellites'].map(k=>[k,{status:'LIVE',value:1,text:null,timestamp_ms:now}]));
const telemetry={connected:true,fields};
const result=(s=base,t=telemetry)=>Object.fromEntries(preflightObservations(s,t,now).map(v=>[v.name,v.state]));
assert.equal(result()['ARM 상태'],'DISARMED');
assert.equal(result({...base,gps:'UNKNOWN'})['GPS'],'미확인');
for(const patch of [{reachable:false},{session:0},{source:'OFFLINE'},{connection:'UNKNOWN'}]) {
 const out=result({...base,...patch});
 for(const key of ['FC 연결 / HEARTBEAT','기체 식별','ARM 상태','센서 상태','배터리','GPS']) assert.equal(out[key],'미확인');
}
for(const [key,label,ttl] of [['armed','ARM 상태',5000],['battery','배터리',10000],['gps_fix','GPS',5000],['satellites','GPS',5000],['sensors_health','센서 상태',10000]]) {
 for(const change of [{status:'STALE'},{status:'INVALID'},{timestamp_ms:now-ttl-1},{timestamp_ms:now+1}]) {
  assert.equal(result(base,{...telemetry,fields:{...fields,[key]:{...fields[key],...change}}})[label],'미확인');
 }
}
assert.equal(result()['기체 위치'],'별도 위치 확인 필요');
assert.equal(result()['연결 품질'],'지연·손실률 미측정');
