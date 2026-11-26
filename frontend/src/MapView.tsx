import { memo, useEffect, useRef, useState } from 'react';
import Map from 'ol/Map';
import View from 'ol/View';
import TileLayer from 'ol/layer/Tile';
import XYZ from 'ol/source/XYZ';
import { ScaleLine, defaults as controls } from 'ol/control';
import { fromLonLat, toLonLat } from 'ol/proj';
import 'ol/ol.css';
import Feature from 'ol/Feature';
import VectorLayer from 'ol/layer/Vector';
import VectorSource from 'ol/source/Vector';
import Point from 'ol/geom/Point';
import GeoJSON from 'ol/format/GeoJSON';
import { incidentArea } from './mapGeometry';
import { alternateQuery, createSearchGate, mergeResults, normalizeQuery, rank, searchError, type SearchItem, type SearchKind, type SearchResponse } from './locationSearch';
import { getCenter } from 'ol/extent';
import LineString from 'ol/geom/LineString';
import { Circle as CircleStyle, Fill, Stroke, Style } from 'ol/style';
import { useGpsTelemetry } from './Telemetry';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
const keyName = 'firescene.vworld.key', centerName = 'firescene.map.center';
const defaultCenter: [number, number] = [126.978, 37.5665];
function initialCenter(): [number, number] {
  try { const value: unknown = JSON.parse(localStorage.getItem(centerName) || 'null'); if (Array.isArray(value) && value.length === 2 && value.every(Number.isFinite) && Math.abs(value[0]) <= 180 && Math.abs(value[1]) <= 85) return value as [number, number]; } catch { /* use default */ }
  return defaultCenter;
}
export function MapSettings() {
  const [key, setKey] = useState('');
  useEffect(() => { if (isTauri()) void invoke<string>('load_map_key').then(v => setKey(v || localStorage.getItem(keyName) || '')).catch(() => setMessage('지도 설정 읽기 실패')); else setKey(localStorage.getItem(keyName) || ''); }, []);
  const [center, setCenter] = useState(() => initialCenter().join(', '));
  const [message, setMessage] = useState('');
  async function save() { const parts = center.split(',').map(x => Number(x.trim())); if (parts.length !== 2 || !parts.every(Number.isFinite) || Math.abs(parts[0]) > 180 || Math.abs(parts[1]) > 85) { setMessage('중심 좌표는 경도, 위도 순서로 입력하세요.'); return; } try { if (isTauri()) { await invoke('save_map_key', {key:key.trim()}); localStorage.removeItem(keyName); } else localStorage.setItem(keyName, key.trim()); localStorage.setItem(centerName, JSON.stringify(parts)); setMessage('로컬 설정에 저장했습니다. 인증 성공 여부는 확인되지 않았습니다.'); } catch { setMessage('지도 설정 저장에 실패했습니다.'); } }
  return <section className="connection-panel"><div className="section-heading"><h2>지도 설정</h2></div><label>VWorld API 키<input type="password" autoComplete="off" value={key} onChange={e=>setKey(e.target.value)}/></label><label>초기 중심 (경도, 위도)<input value={center} onChange={e=>setCenter(e.target.value)}/></label><button className="connection-button" onClick={()=>void save()}>지도 설정 저장</button><p>{message || '키는 이 기기의 로컬 설정에 저장되며 완전한 비밀 저장소가 아닙니다.'}</p><p>VWorld에서 API 키를 발급받고 사용 도메인 및 데스크톱 앱 허용 여부를 확인하세요. 인터넷 연결이 필요합니다.</p></section>;
}
export type MapSelection = {name:string; address:string; id:string; center:number[]; geometry:object|null; radius:number};
export const MapView = memo(function MapView({ source, initialSelection, onSelection, radiusOverride, compact = false }: { source: string; initialSelection?: MapSelection | null; onSelection?: (selection:MapSelection|null)=>void; radiusOverride?: number; compact?: boolean }) {
  const target = useRef<HTMLDivElement>(null); const mapRef = useRef<Map | null>(null); const savedView = useRef<{center:number[];zoom:number}|undefined>(undefined);
  const buildingFeatures = useRef(new VectorSource()); const areaFeatures = useRef(new VectorSource()); const buildingRequest = useRef(0); const lastBuildingRequest = useRef(0);
  const [building,setBuilding] = useState<Omit<MapSelection,'radius'>|null>(()=>initialSelection ? {name:initialSelection.name,address:initialSelection.address,id:initialSelection.id,center:initialSelection.center,geometry:initialSelection.geometry} : null); const [buildingStatus,setBuildingStatus] = useState(initialSelection ? '저장된 현장 위치' : '지도에서 건물을 선택하세요'); const [radius,setRadius] = useState(initialSelection?.radius ?? 50);
  const features = useRef(new VectorSource()); const searchFeatures = useRef(new VectorSource()); const searchGate = useRef(createSearchGate()); const lastRequest = useRef(0);
  const [infoOpen,setInfoOpen] = useState(false);
  useEffect(()=>{if(building)setInfoOpen(true);},[building]);
  const [query,setQuery] = useState(''); const [searching,setSearching] = useState(false); const [results,setResults] = useState<SearchItem[]>([]); const [searchStatus,setSearchStatus] = useState(''); const track = useRef<number[][]>([]); const trackIdentity = useRef(''); const lastTimestamp = useRef(0); const lastLive = useRef(false);
  const [layer, setLayer] = useState<'Base'|'Satellite'>('Base'); const [status, setStatus] = useState('지도 준비'); const [coords, setCoords] = useState(initialCenter); const [follow, setFollow] = useState(false);
  const [key,setKey] = useState(''); const fields = useGpsTelemetry();
  const [map3dEnabled,setMap3dEnabled] = useState(false); const [map3dOpen,setMap3dOpen] = useState(false); const [map3dStatus,setMap3dStatus] = useState('3D 실험 기능 비활성');
  useEffect(() => { let alive=true; let generation=0; let unlisten: (()=>void)|undefined; if (isTauri()) void (async()=>{ try { const enabled=await invoke<boolean>('map3d_available'); if (!enabled || !alive) return; const stop=await listen('map3d-closed',()=>{ if (!alive) return; generation++; setMap3dOpen(false); setMap3dStatus('3D 창 종료 · 2D 복원'); }); if (!alive) { stop(); return; } unlisten=stop; const currentGeneration=generation; const opened=await invoke<boolean>('map3d_is_open'); if (!alive) return; if (generation === currentGeneration) { setMap3dOpen(opened); setMap3dStatus(opened ? '3D 창 열림 · 로딩 및 권한 미검증' : '3D 실험 기능 · 검증 필요'); } setMap3dEnabled(true); } catch { if(alive) setMap3dStatus('3D 상태 확인 실패'); } })(); return ()=>{ alive=false; unlisten?.(); }; }, []);
  useEffect(() => { if (isTauri()) void invoke<string>('load_map_key').then(v=>setKey(v || localStorage.getItem(keyName) || '')).catch(()=>setStatus('지도 설정 읽기 실패')); else setKey(localStorage.getItem(keyName) || ''); }, []);
  const lat = fields.gps_lat, lon = fields.gps_lon;
  const live = lat?.status === 'LIVE' && lon?.status === 'LIVE' && lat.value !== null && lon.value !== null && Math.abs(lat.timestamp_ms - lon.timestamp_ms) < 100 && lat.system_id === lon.system_id && lat.component_id === lon.component_id;
  useEffect(() => {
    if (!target.current || !key || map3dOpen) return;
    let failed = false;
    const source = new XYZ({ url: `https://api.vworld.kr/req/wmts/1.0.0/${encodeURIComponent(key)}/${layer}/{z}/{y}/{x}.${layer === 'Base' ? 'png' : 'jpeg'}`, attributions: '© VWorld', maxZoom: 19 });
    source.on('tileloadstart', () => { if (!failed) setStatus('지도 불러오는 중'); });
    source.on('tileloadend', () => { if (!failed) setStatus('지도 타일 수신'); });
    source.on('tileloaderror', () => { failed = true; setStatus('지도 타일 오류 · 연결 또는 인증과 도메인 설정 확인'); });
    const map = new Map({ target: target.current, layers: [new TileLayer({ source }), new VectorLayer({ source: features.current }), new VectorLayer({ source: searchFeatures.current }), new VectorLayer({ source: areaFeatures.current }), new VectorLayer({ source: buildingFeatures.current })], view: new View({ center: savedView.current?.center || fromLonLat(initialSelection?.center || initialCenter()), zoom: savedView.current?.zoom ?? 15 }), controls: controls().extend([new ScaleLine()]) }); mapRef.current = map;
    map.on('singleclick', async event => {
      if (!isTauri()) { setBuildingStatus('건물 조회는 Tauri 앱에서 사용할 수 있습니다'); return; }
      const id = ++buildingRequest.current; setBuilding(null); buildingFeatures.current.clear(); areaFeatures.current.clear(); if (Date.now() - lastBuildingRequest.current < 1000) { setBuildingStatus('잠시 후 다시 선택하세요'); return; } lastBuildingRequest.current = Date.now(); const coord = toLonLat(event.coordinate); setBuildingStatus('건물 조회 중');
      try {
        const response = await invoke<{status?:string; error?:{code?:string}; result?:{featureCollection?:object}}>('buildings_at',{key,lon:coord[0],lat:coord[1]});
        if (buildingRequest.current !== id) return;
        if (response.status !== 'OK' || !response.result?.featureCollection) { setBuildingStatus(response.status === 'NOT_FOUND' ? '이 위치에 제공되는 건물 정보가 없습니다' : '건물 조회 또는 인증 오류'); return; }
        const candidates = new GeoJSON().readFeatures(response.result.featureCollection,{dataProjection:'EPSG:4326',featureProjection:'EPSG:3857'});
        const selected = candidates.find(f=>f.getGeometry()?.intersectsCoordinate(event.coordinate));
        if (!selected || !selected.getGeometry()) { setBuildingStatus('선택 위치의 건물 정보가 없습니다'); return; }
        buildingFeatures.current.clear(); selected.setStyle(new Style({stroke:new Stroke({color:'#ffba69',width:3}),fill:new Fill({color:'rgba(255,186,105,0.3)'})})); buildingFeatures.current.addFeature(selected);
        const prop = selected.getProperties(); const text = (v:unknown) => typeof v === 'string' || typeof v === 'number' ? String(v) : '';
        setBuilding({name:text(prop.buld_nm) || '건물명 미제공',address:[prop.sido,prop.sigungu,prop.rd_nm,prop.buld_no].map(text).filter(Boolean).join(' ') || '주소 미제공',id:text(prop.bd_mgt_sn),center:toLonLat(getCenter(selected.getGeometry()!.getExtent())),geometry:new GeoJSON().writeGeometryObject(selected.getGeometry()!,{featureProjection:'EPSG:3857',dataProjection:'EPSG:4326'})});
        setBuildingStatus('VWorld 도로명주소 건물 · 실제 제공 속성');
      } catch { if (buildingRequest.current === id) setBuildingStatus('건물 조회 네트워크 오류'); }
    });
    map.on('moveend', () => { const c = map.getView().getCenter(); if (c) setCoords(toLonLat(c) as [number, number]); });
    const observer = new ResizeObserver(() => map.updateSize()); observer.observe(target.current);
    return () => { const center = map.getView().getCenter(); const zoom = map.getView().getZoom(); if (center && zoom !== undefined) savedView.current = {center,zoom}; buildingRequest.current++; observer.disconnect(); map.setTarget(undefined); map.dispose(); mapRef.current = null; };
  }, [key, layer, map3dOpen]);
  useEffect(() => {
    const vector = features.current;
    if (!live) { if (lastLive.current) { track.current = []; vector.clear(); } lastLive.current = false; return; }
    const identity = `${source}:${lat.system_id}:${lat.component_id}`;
    if (identity !== trackIdentity.current) { trackIdentity.current = identity; track.current = []; lastTimestamp.current = 0; }
    lastLive.current = true;
    const position = fromLonLat([lon.value!, lat.value!]);
    if (lat.timestamp_ms !== lastTimestamp.current) { lastTimestamp.current = lat.timestamp_ms; track.current.push(position); if (track.current.length > 500) track.current.shift(); }
    vector.clear();
    if (track.current.length > 1) { const line = new Feature(new LineString(track.current)); line.setStyle(new Style({stroke:new Stroke({color:'#e3b977',width:3})})); vector.addFeature(line); }
    const marker = new Feature(new Point(position)); marker.setStyle(new Style({image:new CircleStyle({radius:7,fill:new Fill({color:'#e3b977'}),stroke:new Stroke({color:'#171b19',width:2})})})); vector.addFeature(marker);
    if (follow) mapRef.current?.getView().setCenter(position);
  }, [follow, live, lat?.value, lon?.value, lat?.timestamp_ms, lat?.system_id, lat?.component_id, source]);
  useEffect(() => {
    if (!initialSelection) return;
    try { if (initialSelection.geometry) { const geometry=new GeoJSON().readGeometry(initialSelection.geometry,{dataProjection:'EPSG:4326',featureProjection:'EPSG:3857'}); if (!geometry) throw new Error('geometry'); const feature=new Feature(geometry); feature.setStyle(new Style({stroke:new Stroke({color:'#ffba69',width:3}),fill:new Fill({color:'rgba(255,186,105,0.3)'})})); buildingFeatures.current.addFeature(feature); } else { const marker=new Feature(new Point(fromLonLat(initialSelection.center))); marker.setStyle(new Style({image:new CircleStyle({radius:7,fill:new Fill({color:'#73bbec'}),stroke:new Stroke({color:'#171b19',width:2})})})); searchFeatures.current.addFeature(marker); } } catch { setBuildingStatus('저장된 윤곽을 표시할 수 없습니다'); }
  }, []);
  useEffect(() => {
    areaFeatures.current.clear();
    if (!building) return;
    const polygon = incidentArea(building.center, radius); if (!polygon) return;
    const feature = new Feature(polygon); feature.setStyle(new Style({stroke:new Stroke({color:'#e3b977',width:2,lineDash:[6,5]}),fill:new Fill({color:'rgba(227,185,119,0.12)'})})); areaFeatures.current.addFeature(feature);
  }, [building,radius]);
  useEffect(() => { if (radiusOverride !== undefined && Number.isFinite(radiusOverride)) setRadius(radiusOverride); }, [radiusOverride]);
  useEffect(() => { onSelection?.(building ? {...building,radius} : null); }, [building,radius]);
  async function open3d() {
    if (!map3dEnabled || map3dOpen) return;
    const center = building?.center || coords;
    setMap3dOpen(true); setMap3dStatus('3D 창 여는 중 · 로딩 및 권한 미검증');
    try { await invoke('open_map3d', {lon:center[0],lat:center[1],radius:building && radius >= 1 && radius <= 2000 ? radius : null}); }
    catch { setMap3dOpen(false); setMap3dStatus('3D 창을 열지 못했습니다 · HTTPS 설정 확인'); }
  }
  useEffect(()=>()=>{searchGate.current.cancel();}, []);
  function cancelSearch() { searchGate.current.cancel(); setSearching(false); setResults([]); setSearchStatus(''); }
  async function search() {
    if (searching) return;
    const normalized=normalizeQuery(query);
    if (!key) { setSearchStatus('지도 API 키가 필요합니다'); return; }
    if (!isTauri()) { setSearchStatus('검색은 Tauri 앱에서 사용할 수 있습니다'); return; }
    if (normalized.length < 2 || normalized.length > 100) { setSearchStatus('검색어는 2~100자로 입력하세요'); return; }
    if (Date.now()-lastRequest.current<1000) { setSearchStatus('잠시 후 다시 검색하세요'); return; }
    const id=searchGate.current.start(); lastRequest.current=Date.now(); setResults([]); setSearching(true); setSearchStatus('검색 중');
    const run=async (q:string, kinds:SearchKind[], candidate=false) => Promise.all(kinds.map(async kind=>{
      try { const response=await invoke<SearchResponse>('search_places',{key,query:q,kind});
        if(response.status==='NOT_FOUND') return {items:[] as SearchItem[],error:''};
        if(response.status!=='OK') return {items:[] as SearchItem[],error:searchError(response.error?.code)};
        return {items:(response.result?.items||[]).map(item=>({...item,kind,candidate})),error:''};
      } catch(e) { const msg=String(e); return {items:[] as SearchItem[],error:/429/.test(msg)?searchError('429'):/40[13]/.test(msg)?searchError('403'):'네트워크 또는 검색 서버 오류'}; }
    }));
    const primary=await run(normalized,['road','parcel','place']); if(!searchGate.current.isCurrent(id)) return;
    let all=primary.flatMap(r=>r.items); const errors=primary.map(r=>r.error).filter(Boolean);
    const alternate=alternateQuery(normalized);
    if(alternate && alternate!==normalized && !all.some(i=>rank(i,normalized)<=1) && errors.length===0) {
      const more=await run(alternate,['place'],true); if(!searchGate.current.isCurrent(id)) return;
      all=all.concat(more.flatMap(r=>r.items)); errors.push(...more.map(r=>r.error).filter(Boolean));
    }
    const merged=mergeResults(all,normalized); setResults(merged); setSearching(false);
    setSearchStatus(errors.length ? `${merged.length ? `${merged.length}개 결과 · 일부 검색 실패` : '검색 실패'} · ${errors[0]}` : merged.length ? `${merged.length}개 결과` : '검색 결과가 없습니다');
  }
  function selectResult(item: SearchItem) {
    const position = fromLonLat([Number(item.point.x),Number(item.point.y)]); searchFeatures.current.clear();
    const marker = new Feature(new Point(position)); marker.setStyle(new Style({image:new CircleStyle({radius:7,fill:new Fill({color:'#73bbec'}),stroke:new Stroke({color:'#171b19',width:2})})})); searchFeatures.current.addFeature(marker);
    setSearchStatus('검색 위치로 이동 · 건물 선택은 지도에서 별도로 하세요');
    setFollow(false); mapRef.current?.getView().animate({center:position,zoom:17,duration:250});
  }
  return <section className="map-area map-real" aria-label="지도 작업 공간"><div className="map-toolbar"><b>관제 지도</b><button disabled={!map3dEnabled || map3dOpen} onClick={()=>void open3d()} title={map3dStatus}>3D 실험</button><button disabled={map3dOpen} onClick={()=>setLayer('Base')} aria-pressed={layer==='Base'}>기본</button><button disabled={map3dOpen} onClick={()=>setLayer('Satellite')} aria-pressed={layer==='Satellite'}>위성</button><button disabled={map3dOpen} onClick={()=>{mapRef.current?.getView().setCenter(fromLonLat(initialCenter())); mapRef.current?.getView().setZoom(15); setFollow(false);}}>초기화</button><button disabled={!live || map3dOpen} onClick={()=>{if(live) mapRef.current?.getView().setCenter(fromLonLat([lon.value!,lat.value!]));}}>GPS 원시 위치</button><label><input type="checkbox" disabled={map3dOpen} checked={follow} onChange={e=>setFollow(e.target.checked)}/> GPS 추적</label></div><div className="map-search"><input value={query} onChange={e=>{cancelSearch(); setQuery(e.target.value);}} onKeyDown={e=>{if(e.key==='Enter') void search(); if(e.key==='Escape') cancelSearch();}} placeholder="주소, 건물명, 장소명을 입력하세요." aria-label="위치 검색"/><button disabled={searching} onClick={()=>void search()}>검색</button><button onClick={()=>{cancelSearch();setQuery('');searchFeatures.current.clear();}} aria-label="검색어 삭제">지우기</button>{searching && <button onClick={cancelSearch}>검색 취소</button>}<span role="status">{searchStatus}</span>{results.length > 0 && <div className="map-results">{results.map((item,i)=><button key={i} onClick={()=>{selectResult(item); setResults([]);}}><b>{item.title || item.address?.bldnm || '이름 미제공'}</b> · {item.kind==='place'?'장소':item.kind==='road'?'도로명 주소':'지번 주소'}{item.candidate?' · 대체 검색 후보':rank(item,normalizeQuery(query))>=3?' · 유사 검색 후보':''}{item.address?.road && <span> · 도로명: {item.address.road}</span>}{item.address?.parcel && <span> · 지번: {item.address.parcel}</span>}</button>)}</div>}</div><div className="map-canvas" ref={target}>{!key && <div className="map-empty"><strong>VWorld API 키가 설정되지 않았습니다</strong><span>시스템 설정에서 키를 입력하세요. 인터넷 연결이 필요합니다.</span></div>}</div>{compact ? <details className="map-info-disclosure" open={infoOpen} onToggle={e=>setInfoOpen(e.currentTarget.open)}><summary>지도 상태 및 현장 도구</summary><div className="map-info"><span>{buildingStatus}{building ? ` · ${building.name} · ${building.address} · ${building.id}` : ''}</span>{building && <><button onClick={()=>setRadius(50)}>50 m</button><button onClick={()=>setRadius(100)}>100 m</button><label>반경 m <input type="number" min="1" max="2000" value={radius} onChange={e=>setRadius(Number(e.target.value))}/></label><button onClick={()=>mapRef.current?.getView().setCenter(fromLonLat(building.center))}>건물 중심</button><button onClick={()=>{setBuilding(null); buildingFeatures.current.clear(); areaFeatures.current.clear();}}>현장 초기화</button><span>조회 범위 · 비행 안전거리 아님</span></>}<span>{map3dStatus}</span><span>{key ? status : '지도 미연결'}</span><span>중심 {coords[1].toFixed(5)}, {coords[0].toFixed(5)}</span><span>{live ? `GPS 원시 위치 · 경로 최대 500점 · ${source === 'SIMULATION' ? '시뮬레이션' : source === 'QGC_FORWARD' ? 'QGC 전달 (선택)' : source === 'ROUTER_FORWARD' ? 'Router 전달 (선택)' : '직접 직렬'} · ${new Date(lat.timestamp_ms).toLocaleTimeString()} · 출처 미검증` : 'GPS 원시 위치 미수신 또는 오래됨'}</span><span>지도 출처: VWorld</span></div></details> : <div className="map-info"><span>{buildingStatus}{building ? ` · ${building.name} · ${building.address} · ${building.id}` : ''}</span>{building && <><button onClick={()=>setRadius(50)}>50 m</button><button onClick={()=>setRadius(100)}>100 m</button><label>반경 m <input type="number" min="1" max="2000" value={radius} onChange={e=>setRadius(Number(e.target.value))}/></label><button onClick={()=>mapRef.current?.getView().setCenter(fromLonLat(building.center))}>건물 중심</button><button onClick={()=>{setBuilding(null); buildingFeatures.current.clear(); areaFeatures.current.clear();}}>현장 초기화</button><span>조회 범위 · 비행 안전거리 아님</span></>}<span>{map3dStatus}</span><span>{key ? status : '지도 미연결'}</span><span>중심 {coords[1].toFixed(5)}, {coords[0].toFixed(5)}</span><span>{live ? `GPS 원시 위치 · 경로 최대 500점 · ${source === 'SIMULATION' ? '시뮬레이션' : source === 'QGC_FORWARD' ? 'QGC 전달 (선택)' : source === 'ROUTER_FORWARD' ? 'Router 전달 (선택)' : '직접 직렬'} · ${new Date(lat.timestamp_ms).toLocaleTimeString()} · 출처 미검증` : 'GPS 원시 위치 미수신 또는 오래됨'}</span><span>지도 출처: VWorld</span></div>}</section>;
});
