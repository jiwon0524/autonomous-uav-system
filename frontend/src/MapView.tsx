import { useEffect, useRef, useState } from 'react';
import Map from 'ol/Map';
import View from 'ol/View';
import TileLayer from 'ol/layer/Tile';
import XYZ from 'ol/source/XYZ';
import { ScaleLine, defaults as controls } from 'ol/control';
import { fromLonLat, toLonLat } from 'ol/proj';
import 'ol/ol.css';
import { useTelemetry } from './Telemetry';
const keyName = 'firescene.vworld.key', centerName = 'firescene.map.center';
const defaultCenter: [number, number] = [126.978, 37.5665];
function initialCenter(): [number, number] {
  try { const value: unknown = JSON.parse(localStorage.getItem(centerName) || 'null'); if (Array.isArray(value) && value.length === 2 && value.every(Number.isFinite) && Math.abs(value[0]) <= 180 && Math.abs(value[1]) <= 85) return value as [number, number]; } catch { /* use default */ }
  return defaultCenter;
}
export function MapSettings() {
  const [key, setKey] = useState(() => localStorage.getItem(keyName) || '');
  const [center, setCenter] = useState(() => initialCenter().join(', '));
  const [message, setMessage] = useState('');
  function save() { const parts = center.split(',').map(x => Number(x.trim())); if (parts.length !== 2 || !parts.every(Number.isFinite) || Math.abs(parts[0]) > 180 || Math.abs(parts[1]) > 85) { setMessage('중심 좌표는 경도, 위도 순서로 입력하세요.'); return; } localStorage.setItem(keyName, key.trim()); localStorage.setItem(centerName, JSON.stringify(parts)); setMessage('로컬 설정에 저장했습니다. 인증 성공 여부는 확인되지 않았습니다.'); }
  return <section className="connection-panel"><div className="section-heading"><h2>지도 설정</h2></div><label>VWorld API 키<input type="password" autoComplete="off" value={key} onChange={e=>setKey(e.target.value)}/></label><label>초기 중심 (경도, 위도)<input value={center} onChange={e=>setCenter(e.target.value)}/></label><button className="connection-button" onClick={save}>지도 설정 저장</button><p>{message || '키는 이 기기의 웹뷰 로컬 저장소에 저장되며 완전한 비밀 저장소가 아닙니다.'}</p><p>VWorld에서 API 키를 발급받고 사용 도메인 및 데스크톱 앱 허용 여부를 확인하세요. 인터넷 연결이 필요합니다.</p></section>;
}
export function MapView({ source }: { source: string }) {
  const target = useRef<HTMLDivElement>(null); const mapRef = useRef<Map | null>(null);
  const [layer, setLayer] = useState<'Base'|'Satellite'>('Base'); const [status, setStatus] = useState('지도 준비'); const [coords, setCoords] = useState(initialCenter); const [follow, setFollow] = useState(false);
  const key = localStorage.getItem(keyName) || ''; const { fields } = useTelemetry();
  const lat = fields.gps_lat, lon = fields.gps_lon;
  const live = lat?.status === 'LIVE' && lon?.status === 'LIVE' && lat.value !== null && lon.value !== null && lat.timestamp_ms === lon.timestamp_ms && lat.system_id === lon.system_id && lat.component_id === lon.component_id;
  useEffect(() => {
    if (!target.current || !key) return;
    let failed = false;
    const source = new XYZ({ url: `https://api.vworld.kr/req/wmts/1.0.0/${encodeURIComponent(key)}/${layer}/{z}/{y}/{x}.${layer === 'Base' ? 'png' : 'jpeg'}`, attributions: '© VWorld', maxZoom: 19 });
    source.on('tileloadstart', () => { if (!failed) setStatus('지도 불러오는 중'); });
    source.on('tileloadend', () => { if (!failed) setStatus('지도 타일 수신'); });
    source.on('tileloaderror', () => { failed = true; setStatus('지도 타일 오류 · 연결 또는 인증과 도메인 설정 확인'); });
    const map = new Map({ target: target.current, layers: [new TileLayer({ source })], view: new View({ center: fromLonLat(initialCenter()), zoom: 15 }), controls: controls().extend([new ScaleLine()]) }); mapRef.current = map;
    map.on('moveend', () => { const c = map.getView().getCenter(); if (c) setCoords(toLonLat(c) as [number, number]); });
    const observer = new ResizeObserver(() => map.updateSize()); observer.observe(target.current);
    return () => { observer.disconnect(); map.setTarget(undefined); map.dispose(); mapRef.current = null; };
  }, [key, layer]);
  useEffect(() => { if (follow && live) mapRef.current?.getView().setCenter(fromLonLat([lon.value!, lat.value!])); }, [follow, live, lat?.value, lon?.value]);
  return <section className="map-area map-real" aria-label="지도 작업 공간"><div className="map-toolbar"><b>관제 지도</b><button onClick={()=>setLayer('Base')} aria-pressed={layer==='Base'}>기본</button><button onClick={()=>setLayer('Satellite')} aria-pressed={layer==='Satellite'}>위성</button><button onClick={()=>{mapRef.current?.getView().setCenter(fromLonLat(initialCenter())); mapRef.current?.getView().setZoom(15); setFollow(false);}}>초기화</button><button disabled={!live} onClick={()=>{if(live) mapRef.current?.getView().setCenter(fromLonLat([lon.value!,lat.value!]));}}>GPS 원시 위치</button><label><input type="checkbox" checked={follow} onChange={e=>setFollow(e.target.checked)}/> GPS 추적</label></div><div className="map-canvas" ref={target}>{!key && <div className="map-empty"><strong>VWorld API 키가 설정되지 않았습니다</strong><span>시스템 설정에서 키를 입력하세요. 인터넷 연결이 필요합니다.</span></div>}</div><div className="map-info"><span>{key ? status : '지도 미연결'}</span><span>중심 {coords[1].toFixed(5)}, {coords[0].toFixed(5)}</span><span>{live ? `GPS 원시 위치 · ${source === 'SIMULATION' ? '시뮬레이션' : '하드웨어 링크'} · ${new Date(lat.timestamp_ms).toLocaleTimeString()} · 출처 미검증` : 'GPS 원시 위치 미수신 또는 오래됨'}</span><span>지도 출처: VWorld</span></div></section>;
}
