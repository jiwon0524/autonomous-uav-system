# Phase 6 시작 경험 — 진행 중

초기 HTML에 로컬 CSS·SVG 스플래시를 표시하고 React 관제 트리가 마운트되면 즉시 제거한다. 별도 창이나 새 IPC 권한이 없다. 로딩 문구는 인터페이스 로딩만 나타내며 연결·비행 준비 또는 가짜 퍼센트를 표시하지 않는다. 8초 후에도 화면이 남아 있으면 지연과 재시도를 안내한다. 이 타이머는 정상 시작을 지연하지 않는다. 모션 감소 설정에서는 애니메이션을 끈다. 지도 키·네트워크를 기다리지 않는다.

프런트엔드 단위 시험은 지연 안내와 재시도, 이미 해제된 경우의 콜백을 확인한다. 격리 Ubuntu Tauri WebKitGTK 개발 실행은 실제 관제 트리가 마운트된 뒤 오버레이가 제거되고 최초 실행 화면에 진입함을 확인했다. 오프라인 입력 연습도 통과했다. 최초 실행 실제 화면과 결과는 `~/FireScene-evidence/phase6/branding/first-run.png`, `results.json`에 있다. 실제 스플래시 표시 순간·전환 영상, 실패 주입, Windows WebView2 및 재시작 보존의 GUI 검증은 아직 없다. 첫 렌더 전 OS 창의 색상과 표시 시간도 미측정이다.

후속 실제 Xvfb/Tauri 녹화는 `~/FireScene-evidence/phase6/branding/firescene-phase6-launch.mp4`, 프레임은 `firescene-phase6-splash.png`다. 기존 첫 녹화에서 흰 창이 보여 Tauri 창·웹뷰 배경을 `#0b0f12`로 지정했고 재녹화에서는 흰 프레임이 보이지 않았다. 이는 Linux 개발 환경의 관찰이며 Windows 결과가 아니다. 녹화는 30fps 표본이므로 매우 짧은 깜빡임을 배제하지 못한다. 재시험에서 마운트 후 해제 및 오프라인 입력 PASS. 아래 GUI 회귀에서 온보딩 완료 상태 재시작도 확인했다. 실패 주입과 Windows는 여전히 미검증이다.
