import { useEffect, useRef, useState } from 'react';
import { finishFirstRun, type GuideLanguage, type Theme } from './setupPreferences';
const text = {
 ko: {eyebrow:'시작하기', title:'FireScene GCS에 오신 것을 환영합니다', intro:'현장을 살피고 기체 상태를 확인하는 읽기 전용 관제 시스템입니다.', language:'안내 언어', theme:'화면 테마', guideTitle:'연결 전에 확인하세요', vehicle:'KOA FC 연결: 프로펠러를 제거하고 USB로 연결한 뒤 시스템 설정에서 포트와 통신 속도를 선택하세요.', map:'지도: 시스템 설정에서 VWorld 키를 입력할 수 있습니다. 키가 없어도 기체 연결을 시도할 수 있습니다.', safety:'실제 비행 명령은 잠겨 있습니다.', skip:'건너뛰기', start:'종합 관제 시작', dark:'Dark', day:'Day', note:'프로그램의 나머지 화면은 현재 한국어로 제공됩니다.'},
 en: {eyebrow:'GET STARTED', title:'Welcome to FireScene GCS', intro:'A read-only ground station for incident awareness and aircraft telemetry.', language:'Guide language', theme:'Appearance', guideTitle:'Before connecting', vehicle:'KOA FC: remove the propellers, connect via USB, then select the port and baud rate in System Settings.', map:'Map: enter a VWorld key in System Settings. You can still attempt an aircraft connection without a key.', safety:'Flight commands are locked.', skip:'Skip setup', start:'Open mission control', dark:'Dark', day:'Day', note:'The rest of the application is currently available in Korean.'}
} as const;
export function FirstRun({theme,onTheme,onClose}:{theme:Theme;onTheme:(v:Theme)=>void;onClose:()=>void}) {
 const [language,setLanguage]=useState<GuideLanguage>('ko'); const t=text[language];
 const dialog=useRef<HTMLElement>(null);
 useEffect(()=>{dialog.current?.querySelector<HTMLButtonElement>('button')?.focus();},[]);
 const finish=()=>{finishFirstRun(language,theme);onClose();};
 const trap=(e:React.KeyboardEvent)=>{if(e.key==='Escape'){e.preventDefault();finish();return;}if(e.key!=='Tab')return;const buttons=dialog.current?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)');if(!buttons?.length)return;const first=buttons[0],last=buttons[buttons.length-1];if(e.shiftKey&&document.activeElement===first){e.preventDefault();last.focus();}else if(!e.shiftKey&&document.activeElement===last){e.preventDefault();first.focus();}};
 return <div className="first-run-overlay"><section ref={dialog} onKeyDown={trap} className={`first-run theme-${theme}`} role="dialog" aria-modal="true" aria-labelledby="first-run-title" lang={language}>
  <header className="first-run-brand"><img src="/firescene-mark.png" alt=""/><div><strong>FireScene GCS</strong><span>AEROSPACE MISSION CONTROL · 0.1.0 HARDWARE PREVIEW</span></div></header>
  <main className="first-run-content"><span className="first-run-eyebrow">{t.eyebrow}</span><h1 id="first-run-title">{t.title}</h1><p>{t.intro}</p>
   <div className="first-run-options"><fieldset><legend>{t.language}</legend><button type="button" aria-pressed={language==='ko'} onClick={()=>setLanguage('ko')}>한국어</button><button type="button" aria-pressed={language==='en'} onClick={()=>setLanguage('en')}>English</button></fieldset><fieldset><legend>{t.theme}</legend><button type="button" aria-pressed={theme==='dark'} onClick={()=>onTheme('dark')}>{t.dark}</button><button type="button" aria-pressed={theme==='day'} onClick={()=>onTheme('day')}>{t.day}</button></fieldset></div>
   <p className="first-run-note">{t.note}</p><div className="first-run-guide"><h2>{t.guideTitle}</h2><p>{t.vehicle}</p><p>{t.map}</p><strong>{t.safety}</strong></div>
  </main><footer><button type="button" onClick={finish}>{t.skip}</button><button type="button" className="first-run-primary" onClick={finish}>{t.start}</button></footer>
 </section></div>;
}
