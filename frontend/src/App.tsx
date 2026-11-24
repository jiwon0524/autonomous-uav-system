import { useState, type ReactNode } from 'react';
import { version } from '../package.json';

const navigation = [
  ['overview', '종합 관제'], ['flight', '비행 제어'], ['mission', '임무 계획'],
  ['auto', '자율비행'], ['mapping', '3D 매핑'], ['camera', '카메라'],
  ['safety', '안전 관리'], ['logs', '비행 기록'], ['settings', '시스템 설정'],
] as const;
function Icon({ name, size = 18 }: { name: string; size?: number }) {
  const paths: Record<string, ReactNode> = {
    overview: <><path d="M4 5h16v14H4z"/><path d="M8 9h8M8 13h5"/></>,
    flight: <path d="m12 2 3 8 7 2-7 2-3 8-3-8-7-2 7-2 3-8Z"/>,
    mission: <><circle cx="6" cy="18" r="2"/><circle cx="18" cy="6" r="2"/><path d="M8 18c6 0 4-10 8-11"/></>,
    auto: <><rect x="4" y="5" width="16" height="14" rx="3"/><path d="M9 10h.01M15 10h.01M9 15h6M12 2v3"/></>,
    mapping: <><path d="m3 6 6-3 6 3 6-3v15l-6 3-6-3-6 3V6Z"/><path d="M9 3v15M15 6v15"/></>,
    camera: <><path d="M4 8h4l2-2h4l2 2h4v11H4z"/><circle cx="12" cy="13" r="3"/></>,
    safety: <><path d="M12 3 4 6v5c0 5 3.5 8.5 8 10 4.5-1.5 8-5 8-10V6l-8-3Z"/><path d="m9 12 2 2 4-4"/></>,
    logs: <><path d="M5 3h14v18H5z"/><path d="M8 8h8M8 12h8M8 16h5"/></>,
    settings: <><circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M19 5l-2 2M7 17l-2 2"/></>,
    link: <><path d="m8.5 15.5-2 2a3 3 0 0 1-4-4l3.5-3.5a3 3 0 0 1 4 0M15.5 8.5l2-2a3 3 0 0 1 4 4L18 14a3 3 0 0 1-4 0M8 16l8-8"/></>,
    layers: <><path d="m12 3-9 5 9 5 9-5-9-5Z"/><path d="m3 12 9 5 9-5M3 16l9 5 9-5"/></>,
    search: <><circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/></>,
    alert: <><path d="M12 3 2 21h20L12 3Z"/><path d="M12 9v5M12 18h.01"/></>,
    chevron: <path d="m9 18 6-6-6-6"/>,
    panel: <><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M15 4v16"/></>,
    lock: <><rect x="5" y="10" width="14" height="11" rx="2"/><path d="M8 10V7a4 4 0 0 1 8 0v3"/></>,
    sun: <><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2"/></>,
    moon: <path d="M20 15.5A8 8 0 0 1 8.5 4 8.5 8.5 0 1 0 20 15.5Z"/>,
    more: <><circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/></>,
    locate: <><circle cx="12" cy="12" r="3"/><circle cx="12" cy="12" r="8"/><path d="M12 2V0M12 24v-2M2 12H0M24 12h-2"/></>,
  };
  return <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" aria-hidden>{paths[name] || paths.overview}</svg>;
}

function Mark() {
  return <div className="mark"><span /><i /></div>;
}

function Dot({ tone = 'muted' }: { tone?: 'good' | 'warn' | 'danger' | 'muted' }) {
  return <span className={`dot ${tone}`} />;
}


const descriptions = [
  '', '기체 상태와 비행 제어 기능을 준비하고 있습니다.', '임무 계획 기능을 준비하고 있습니다.',
  '자율비행 기능을 준비하고 있습니다.', '3D 매핑 연동을 준비하고 있습니다.', '카메라 연동을 준비하고 있습니다.',
  '안전 관리 기능을 준비하고 있습니다.', '비행 기록 기능을 준비하고 있습니다.', '연결 및 프로그램 설정을 준비하고 있습니다.',
];

function Telemetry() {
  return <div className="telemetry">{[['고도', '—'], ['속도', '—'], ['배터리', '—'], ['GPS', '미수신']].map(([label, value]) =>
    <div key={label}><span>{label}</span><p><b>{value}</b></p></div>)}</div>;
}

function App() {
  const [page, setPage] = useState(0);
  const [details, setDetails] = useState(true);
  return <div className="app"><div className="product soft-neutral">
    <header className="soft-header">
      <div className="product-brand"><Mark/><b>FireScene</b><span>GCS {version}</span></div>
      <div className="workspace-name"><b>{navigation[page][1]}</b><span>/</span><span>GCS</span></div>
      <div className="status-pill" style={{marginLeft:'auto'}}><Dot/><div><b>기체 미연결</b><span>텔레메트리 미연결</span></div></div>
    </header>
    <div className={`soft-body ${details ? '' : 'panel-closed'}`}>
      <nav className="nav-rail" aria-label="주 메뉴">{navigation.map(([icon,label],i)=><button key={icon} title={label} aria-label={label} aria-current={page === i ? 'page' : undefined} className={page === i ? 'active' : ''} onClick={()=>setPage(i)}><Icon name={icon}/></button>)}</nav>
      <main className="soft-map-wrap">
        <div className="map map-soft">
          {page === 0 ? <>
            <span className="map-heading">관제 작업 공간</span>
            <div className="safety-notice"><Icon name="link" size={17}/><div><b>기체가 연결되지 않았습니다</b><span>기체 위치와 텔레메트리를 확인할 수 없습니다.</span></div></div>
            <div className="map-unavailable"><strong>지도 연동 준비 중</strong><span>기체 위치: 확인 불가</span></div>
            <div className="floating-flight"><div className="flight-heading"><span>현재 비행 정보</span><button aria-label="기체 정보 패널 열기" onClick={()=>setDetails(true)}><Icon name="panel" size={16}/></button></div><Telemetry/></div>
          </> : <div className="placeholder"><Icon name={navigation[page][0]} size={28}/><h1>{navigation[page][1]}</h1><p>개발 예정</p><p>{descriptions[page]}</p></div>}
        </div>
      </main>
      {details && <aside className="soft-inspector open" aria-label="기체 정보">
        <div className="inspector-head"><div><h1>기체 상태</h1><span>연결된 기체 없음</span></div><button onClick={()=>setDetails(false)} aria-label="기체 정보 패널 닫기">×</button></div>
        <div className="tabs"><span className="active">요약</span></div>
        <div className="section-title"><div><b>비행 준비 상태</b><span>기체 연결 후 확인 가능</span></div><strong>확인 불가</strong></div>
        <dl className="state-list">{[['텔레메트리','미연결'],['비행 모드','확인 불가'],['ARM 상태','확인 불가'],['기체 위치','확인 불가'],['GPS','미수신'],['배터리','—']].map(([label,value])=><div key={label}><dt>{label}</dt><dd>{value}</dd></div>)}</dl>
        <div className="command-lock"><Icon name="lock" size={15}/><div><b>비행 명령 잠김</b><span>모든 기체 명령을 사용할 수 없습니다</span></div></div>
        <p className="development-note">기체 연결 기능 개발 예정</p>
      </aside>}
      {!details && <button className="reopen-panel" onClick={()=>setDetails(true)}><Icon name="panel" size={17}/><span>기체 정보</span></button>}
    </div>
  </div></div>;
}
export default App;
