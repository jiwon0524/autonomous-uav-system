import { ConnectionSettings, phaseText, sourceText, useConnection } from './Connection';
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

function Metric({ label, unit, value = '—' }: { label: string; unit?: string; value?: string }) {
  return <div className="metric"><span>{label}</span><div><strong>{value}</strong>{unit && <small>{unit}</small>}</div></div>;
}

function App() {
  const [page, setPage] = useState(0);
  const control = useConnection();
  const connection = control.connection;
  const connected = connection.phase === 'CONNECTED' && !!connection.vehicle;
  const status = connected ? 'PX4 HEARTBEAT 수신' : phaseText[connection.phase] || '미연결';
  const [details, setDetails] = useState(true);
  const [theme, setTheme] = useState<'dark' | 'day'>('dark');
  return <div className={`app theme-${theme}`}><div className="product">
    <header className="header">
      <div className="product-brand"><Mark/><div><b>FireScene <em>GCS</em></b><span>Ground Control Station · {version}</span></div></div>
      <div className="header-context"><span>작업 공간</span><b>{navigation[page][1]}</b></div>
      <div className="header-status"><Dot tone={connected ? 'good' : 'muted'}/><div><b>{connected ? status : '기체 미연결'}</b><span>{connection.source !== 'OFFLINE' ? sourceText(connection) : '텔레메트리 미연결'}</span></div></div>
      <button className="theme-toggle" onClick={() => setTheme(theme === 'dark' ? 'day' : 'dark')} aria-label={theme === 'dark' ? 'Day 테마로 변경' : 'Dark 테마로 변경'}><Icon name={theme === 'dark' ? 'sun' : 'moon'} size={17}/><span>{theme === 'dark' ? 'Day' : 'Dark'}</span></button>
    </header>
    <div className="status-strip" aria-label="주요 비행 상태"><div><span>연결</span><b>{status}</b></div><div><span>비행 모드</span><b>확인 불가</b></div><div><span>ARM</span><b>확인 불가</b></div><div><span>GPS</span><b>미수신</b></div><div><span>배터리</span><b>—</b></div><div className="strip-lock"><Icon name="lock" size={14}/> 명령 잠김</div></div>
    <div className={`body ${details ? '' : 'panel-closed'}`}>
      <nav className="nav-rail" aria-label="주 메뉴"><div className="nav-label">메뉴</div>{navigation.map(([icon,label],i)=><button key={icon} title={label} aria-label={label} aria-current={page === i ? 'page' : undefined} className={page === i ? 'active' : ''} onClick={()=>setPage(i)}><Icon name={icon}/><span>{label}</span></button>)}<div className="nav-foot">FireScene<br/><span>운용 화면</span></div></nav>
      <main className="workspace">
        {page === 0 ? <><div className="workspace-heading"><div><span className="eyebrow">운용 개요</span><h1>종합 관제</h1><p>기체와 임무의 현재 상태</p></div><span className="source-label">{connected ? '비행 데이터 미수신' : '데이터 소스 없음'}</span></div>
          <section className="map-area" aria-label="지도 작업 공간"><div className="map-top"><div><b>관제 작업 공간</b><span>지도</span></div><span>위치 확인 불가</span></div><div className="map-empty"><Icon name="mapping" size={26}/><strong>지도 연동 준비 중</strong><span>위치 데이터가 수신되지 않았습니다</span></div><div className="map-bottom"><span>기체 위치</span><b>확인 불가</b></div></section>
          <section className="telemetry-section"><div className="section-heading"><div><span className="eyebrow">기체 데이터</span><h2>현재 비행 정보</h2></div><span>수신된 데이터 없음</span></div><div className="metrics"><Metric label="고도" unit="m"/><Metric label="비행 속도" unit="m/s"/><Metric label="기체 방향" unit="°"/><Metric label="배터리" unit="%"/><Metric label="GPS" value="미수신"/></div></section>
          <div className="overview-lower"><section><div className="section-heading"><h2>현재 임무</h2><span>정보 없음</span></div><p>표시할 임무가 없습니다</p><small>임무 연결 후 진행 상태를 확인할 수 있습니다.</small></section><section><div className="section-heading"><h2>최근 이벤트</h2><span>기록 없음</span></div><p>수신된 이벤트가 없습니다</p><small>기체 연결 후 이벤트가 표시됩니다.</small></section></div>
        </> : page === 1 ? <><div className="workspace-heading"><div><span className="eyebrow">기체 운용</span><h1>비행 제어</h1><p>자세 및 비행 상태</p></div><span className="source-label">{connected ? '비행 데이터 미수신' : '데이터 소스 없음'}</span></div><div className="flight-layout"><section className="attitude"><div className="section-heading"><div><span className="eyebrow">자세 정보</span><h2>기체 자세</h2></div><span>데이터 미수신</span></div><div className="attitude-empty"><Icon name="flight" size={30}/><strong>자세 정보 확인 불가</strong><span>자세 데이터가 수신되지 않았습니다</span></div><div className="metrics flight-metrics"><Metric label="Roll" unit="°"/><Metric label="Pitch" unit="°"/><Metric label="Yaw" unit="°"/><Metric label="Heading" unit="°"/></div></section><section className="flight-data"><div className="section-heading"><div><span className="eyebrow">비행 정보</span><h2>기체 상태</h2></div></div><div className="metrics flight-values"><Metric label="고도" unit="m"/><Metric label="상승 / 하강 속도" unit="m/s"/></div><dl className="state-list"><div><dt>비행 준비 상태</dt><dd>확인 불가</dd></div><div><dt>명령 상태</dt><dd>잠김</dd></div><div><dt>비행 모드</dt><dd>확인 불가</dd></div></dl><div className="command-lock"><Icon name="lock" size={16}/><div><b>비행 명령 잠김</b><span>모든 기체 명령을 사용할 수 없습니다</span></div></div></section></div></> : page === 8 ? <ConnectionSettings control={control}/> : <div className="placeholder"><Icon name={navigation[page][0]} size={28}/><span className="eyebrow">작업 공간</span><h1>{navigation[page][1]}</h1><p>개발 예정</p><small>{descriptions[page]}</small></div>}
      </main>
      {details && <aside className="inspector" aria-label="기체 정보"><div className="inspector-head"><div><span className="eyebrow">기체 정보</span><h2>기체 상태</h2><span>{connected ? 'PX4 HEARTBEAT 확인' : '연결된 기체 없음'}</span></div><button onClick={()=>setDetails(false)} aria-label="기체 정보 패널 닫기">×</button></div><div className="inspector-section"><div className="section-heading"><h3>연결 상태</h3><span className="state-badge"><Dot tone={connected ? 'good' : 'muted'}/>{status}</span></div><p>{connected ? 'HEARTBEAT 수신 중 · 송신자 인증되지 않음' : '기체에서 수신된 데이터가 없습니다.'}</p></div><div className="inspector-section"><div className="section-heading"><h3>비행 준비 상태</h3><span>확인 불가</span></div><dl className="state-list">{[['비행 데이터','미수신'],['비행 모드','확인 불가'],['ARM 상태','확인 불가'],['기체 위치','확인 불가'],['GPS','미수신'],['배터리','—']].map(([label,value])=><div key={label}><dt>{label}</dt><dd>{value}</dd></div>)}</dl></div><div className="inspector-section"><div className="section-heading"><h3>안전 경고</h3><span>확인 불가</span></div><p>수신된 안전 데이터가 없어 안전 상태를 확인할 수 없습니다.</p></div><div className="command-lock"><Icon name="lock" size={16}/><div><b>비행 명령 잠김</b><span>모든 기체 명령을 사용할 수 없습니다</span></div></div><p className="development-note">읽기 전용 연결 · 비행 명령 비활성</p></aside>}
      {!details && <button className="reopen-panel" onClick={()=>setDetails(true)}><Icon name="panel" size={17}/><span>기체 정보</span></button>}
    </div>
  </div></div>;
}
export default App;
