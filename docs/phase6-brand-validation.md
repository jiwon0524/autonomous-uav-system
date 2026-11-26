# Phase 6 디자인 회귀 — 중간 결과

2026-11-26. 격리 Ubuntu Xvfb/WebKitGTK 개발 Tauri에서 시뮬레이션 빌드, 지도 키 격리 프로필 및 합성 카메라·점군을 사용한 전체 GUI 재시험은 48/48 PASS다. 최초 실행·영어/한국어·Dark/Day·9 메뉴·6 크기·합성 연결·안전·기록/재생·재시작 데이터 보존이 포함된다. 증거 `~/FireScene-evidence/phase6/gui/firescene-phase6-full-gui-simulation/`. 먼저 실행한 시험은 개발 바이너리에 simulation 기능이 없어 시작 버튼을 찾지 못했고 이에 의존한 9개가 실패했다 (`firescene-phase6-full-gui/`, 39/48). 기능을 명시해 다시 빌드한 뒤 같은 시험을 통과했다. 두 결과의 파일에서 지도 키 원문은 발견되지 않았다.

후속 변경은 Tauri 초기 창 배경색뿐이며 변경 후 실제 시작 녹화와 오버레이 해제·오프라인 입력 시험이 통과했다 (`firescene-phase6-background-gui/`). 전체 48개는 배경색 변경 전 실행이다. 실제 Windows, 생산 release IPC 재검증, 스플래시 실패 주입 및 물리 장치는 별도 미검증이다.

이번 GUI의 2초 표본 3회 유휴 중앙값: main RSS 241.46 MiB·CPU 약 2.0%, WebKit RSS 503.50 MiB·CPU 약 3.5%. 공유 페이지가 RSS에 중복될 수 있다. Phase 5의 유휴 중앙값 main 240.54 MiB·6.0%, WebKit 440.69 MiB·12.0%는 다른 실행의 짧은 표본이므로 성능 차이로 단정할 수 없다. 시작 시간, GPU 및 Windows 성능은 미측정이다. 로딩 CSS 2,054 bytes, JS 535 bytes, SVG 1,526 bytes이며 설치 파일 크기는 아직 측정하지 않았다.

## 최종 배경색 및 설치기 변경 후 재시험

소스 체크포인트 `60b4cb354bed4c3cb973cc082caf23446c8c35c9`에서 `cargo build --locked --features simulation --examples --bin firescene-gcs`로 개발 바이너리 구성 후, 기존 여섯 크기·합성 fixture·격리 지도 키 조건의 전체 GUI는 48/48 PASS (`~/FireScene-evidence/phase6/gui/firescene-phase6-installer-final-gui/`). 결과 폴더에서 지도 키 원문 0건. 프런트엔드 19개 시험 파일, TypeScript/Vite, 기본 및 simulation Tauri 권한 시험 PASS. 이 시험은 실제 Windows·물리 기체를 다루지 않는다.

동일 소스의 `tauri/custom-protocol` 일반 release와 `tauri/custom-protocol,simulation` release를 각각 빌드하여 실제 Tauri 앱 origin에서 정상 `connection_status` 성공 및 가상 IPC 8개 미등록 거부를 확인했다 (`firescene-phase6-{normal,simulation}-release-ipc/`). 새 창을 추가하지 않았으며 CSP·ACL은 변경하지 않았다. Windows 설치 프로그램의 컴파일·실행 결과는 아니다.

이번 재시험의 2초×3회 유휴 중앙값은 main RSS 242.44 MiB·CPU 약 2.0%, WebKit RSS 509.39 MiB·CPU 약 3.5%다. 기존 Phase 5 수치와 서로 다른 짧은 실행 표본이고 RSS 공유 페이지가 중복될 수 있으므로 원인 또는 회귀라고 단정하지 않는다. 시작·메인 준비 시간 및 설치 파일 크기의 전후 반복 측정은 여전히 미완료다.
