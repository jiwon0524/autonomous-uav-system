import { formatDateTime as date } from '../utils/format';
import { useState } from 'react';
import { ArrowLeft, Camera, Layers, Radar, Maximize2 } from 'lucide-react';
import { Panel } from '../components/Panel';
import { sensorLabels, sensorStatusLabels } from './model';
import type { SensorId, SensorStatus as Status, SensorSnapshot } from './model';
import { useSensors } from './store';
export function SensorConnection({status}: {status:Status}) {return <span className={`sensor-connection sensor-${status.toLowerCase()}`}><i className="dot"/>{sensorStatusLabels[status]}</span>;}
function EmptyView({id}: {id:SensorId}){const Icon=id==='rgb'?Camera:id==='depth'?Layers:Radar;return <div className="sensor-visualization"><Icon size={32}/>{id==='lidar' && <span>2D LiDAR</span>}<strong>데이터 없음</strong></div>;}
// Visualization slots accept a typed snapshot. Future render adapters own decoding, not UI lifecycle.
export function RgbView({data}: {data:SensorSnapshot['rgb']}){return <div data-source-reference={data.source?.reference ?? undefined}><EmptyView id="rgb"/></div>;}
export function DepthView({data}: {data:SensorSnapshot['depth']}){return <div data-source-reference={data.source?.reference ?? undefined}><EmptyView id="depth"/></div>;}
export function LidarView({data}: {data:SensorSnapshot['lidar']}){return <div data-source-reference={data.source?.reference ?? undefined}><EmptyView id="lidar"/></div>;}
const number=(value:number|null)=>value===null?'--':String(value);
const range=(value:{min:number;max:number}|null)=>value?`${value.min}–${value.max} m`:'--';

export function SensorDetail({id,sensors}: {id:SensorId;sensors:SensorSnapshot}){
 const data=sensors[id];
 const rows=[['상태',sensorStatusLabels[data.status]],['데이터 수신',data.receiving===null?'--':data.receiving?'수신 중':'수신 없음'],['마지막 수신',date(data.timestamp)]];
 if(id==='rgb')rows.push(['해상도',sensors.rgb.resolution?`${sensors.rgb.resolution.width} × ${sensors.rgb.resolution.height}`:'--'],['FPS',number(sensors.rgb.fps)]);
 if(id==='depth')rows.push(['대상 ID',sensors.depth.targetDistance?.targetId ?? '--'],['대상 거리',sensors.depth.targetDistance?`${sensors.depth.targetDistance.distanceMeters} m`:'--'],['최소 감지 거리',sensors.depth.minimumDistance===null?'--':`${sensors.depth.minimumDistance} m`],['Depth 범위',range(sensors.depth.depthRange)]);
 if(id==='lidar')rows.push(['최소 거리',sensors.lidar.minimumDistance===null?'--':`${sensors.lidar.minimumDistance} m`],['스캔 범위',range(sensors.lidar.scanRange)],['스캔 포인트',sensors.lidar.scanPoints===null?'--':String(sensors.lidar.scanPoints.length)]);
 return <dl className="sensor-detail-data">{rows.map(([label,value])=><div key={label}><dt>{label}</dt><dd>{value}</dd></div>)}</dl>;
}
export function SensorCard({id,sensors,onExpand,expanded}: {id:SensorId;sensors:SensorSnapshot;onExpand:()=>void;expanded:boolean}){
 const Icon=id==='rgb'?Camera:id==='depth'?Layers:Radar;
 return <Panel title={sensorLabels[id]} icon={Icon} extra={!expanded && <button aria-label={`${sensorLabels[id]} 확대`} onClick={onExpand}><Maximize2 size={16}/></button>}><div className="sensor-view-slot" role={!expanded?'button':undefined} tabIndex={!expanded?0:undefined} aria-label={!expanded?`${sensorLabels[id]} 데이터 보기`:undefined} onClick={()=>{if(!expanded)onExpand();}} onKeyDown={event=>{if(!expanded && (event.key==='Enter'||event.key===' ')){event.preventDefault();onExpand();}}}>{id==='rgb'?<RgbView data={sensors.rgb}/>:id==='depth'?<DepthView data={sensors.depth}/>:<LidarView data={sensors.lidar}/>}</div><div className="monitor-connection"><SensorConnection status={sensors[id].status}/></div>{expanded && <SensorDetail id={id} sensors={sensors}/>}</Panel>;
}
export function SensorMonitor(){const {sensors}=useSensors();const [focus,setFocus]=useState<SensorId|null>(null);return <div className="monitor-shell"><header className="monitor-header"><div><h1>센서 모니터{focus?` · ${sensorLabels[focus]}`:''}</h1><p>센서 진단 전용 · 실제 센서 연결 대기 중</p></div><span className="monitor-vehicle">UAV-01</span>{focus && <button onClick={()=>setFocus(null)}><ArrowLeft size={15}/>전체 센서</button>}</header><main className={`monitor-grid ${focus?'focused':''}`}>{(['rgb','depth','lidar'] as const).filter(id=>!focus||id===focus).map(id=><SensorCard key={id} id={id} sensors={sensors} expanded={focus===id} onExpand={()=>setFocus(id)}/>)}</main></div>;}
