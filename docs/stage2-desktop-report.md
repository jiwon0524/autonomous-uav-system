# Stage 2 데스크톱 통합 진행 보고

Stage 2는 **미완료**다. 독립 창 실행, Tauri 프로덕션 빌드, 설치 패키지 생성·설치 및 GUI 확인은 성공하지 못했다. 이 문서와 데스크톱 설정은 준비 작업이며 실행 가능한 앱으로 검증된 결과물이 아니다.

## Stage 1 GUI 재검증

- `npm run dev -- --port 5173`: `listen EPERM 127.0.0.1:5173`. 앱 화면을 띄우기 전에 로컬 포트 바인딩이 제한됐다.
- Chrome headless: 앞선 시도에서 `setsockopt: Operation not permitted`; 이번 작업에서도 같은 제한 환경이다.
- `DISPLAY=:1`, `XDG_SESSION_TYPE=x11`, X11 소켓은 보이지만 `xdpyinfo`는 디스플레이를 열지 못했다. 따라서 디스플레이 서버가 사용자 세션에서 정상인지, 인증 문제인지, 샌드박스 제한인지는 확정하지 못한다.
- 코드 타입 검사와 정적 렌더링 테스트는 통과했다. 실제 GUI, Figma 시각 비교, 메뉴 클릭, 창 크기별 레이아웃, 콘솔 오류 및 스크린샷은 미검증이다. 로컬 소켓 오류만으로 애플리케이션 전체가 정상이라고 결론 내릴 수 없다.

## 기술 선택

Tauri 2를 우선 선택했다. 기존 React 빌드 산출물을 앱에 내장할 수 있고, 운영체제 웹뷰와 네이티브 창을 사용해 별도의 브라우저 주소 입력 없이 실행하는 구조를 목표로 한다. Ubuntu는 WebKitGTK 4.1 개발 라이브러리, Windows는 WebView2가 필요하다. Tauri는 Linux DEB/AppImage와 Windows 설치 프로그램을 지원한다. 설치 크기는 아직 측정하지 않았다. Electron은 Chromium과 Node 런타임을 포함하는 대안이지만 이 단계에서는 기존 UI 재사용과 시스템 웹뷰 방식으로 Tauri를 우선한다. 2D/3D 지도는 플랫폼별 WebGL·GPU·WebView 호환성과 오프라인 타일 정책을 실제 데이터로 검증해야 한다.

공식 근거: [사전 요구사항](https://v2.tauri.app/start/prerequisites/), [프론트엔드 및 내장 설정](https://v2.tauri.app/reference/config/), [배포](https://v2.tauri.app/distribute/), [Windows WebView2 설치 방식](https://v2.tauri.app/distribute/windows-installer/), [외부 바이너리](https://v2.tauri.app/develop/sidecar/), [보안 capability](https://v2.tauri.app/security/capabilities/).

## 추가한 구조와 보안

- `src-tauri/Cargo.toml`, `build.rs`, `src/main.rs`: Tauri 2 Rust 진입점 및 빌드 의존성 초안.
- `src-tauri/tauri.conf.json`: 이름, 버전 0.1.0, 1920×1080 초기 창, 크기 조절과 운영체제 기본 창, 로컬 프론트엔드 경로, CSP 및 번들 설정 초안. 최소 창 크기는 850×620이다. 전체 화면 등 운영체제 동작은 실제 검증 전이다.
- `src-tauri/icons/`: 제공된 Figma favicon을 크기별 PNG와 Windows ICO로 변환했다.
- `frontend/src/App.tsx`: 기존 화면에 package 버전을 표시한다. 미연결 데이터와 메뉴 로직은 유지한다.
- 플러그인, 임의 shell 실행, 파일/시리얼 권한, 외부 URL 및 비행 명령 API를 추가하지 않았다. CSP와 capability는 실제 빌드 및 플랫폼별 테스트로 확인해야 한다. 버전 값은 프론트엔드 package, Cargo 및 Tauri 설정에서 일치하도록 유지해야 한다.

Python/MAVSDK server를 포함하지 않았다. 추후 각 운영체제용 고정 바이너리를 sidecar로 묶는 방식이 가능하지만, 프로세스 시작·정상 종료·장애 복구와 명령 차단은 아직 구현·검증되지 않았다. Connection Manager, Serial Discovery, Telemetry, Safety Supervisor, Flight Command Service, Event Logger, Settings 및 Raspberry Pi는 후속 설계 대상으로 남긴다. ROS2는 필수 의존성이 아니다.

## 환경 및 실제 검증

Ubuntu 24.04, Node 22.23.4, Cargo/Rust 1.100.0이 있다. `pkg-config`에서 `webkit2gtk-4.1`, `gtk+-3.0`, `libsoup-3.0` 개발 패키지를 찾지 못했다. Tauri CLI와 Cargo 레지스트리 캐시는 없다. 새 시스템 패키지나 Tauri 의존성은 설치하지 않았다.

- `npm install --save-dev @tauri-apps/cli@^2 --fetch-retries=0 --fetch-timeout=10000`: npm DNS `EAI_AGAIN`으로 실패, package 변경 없음.
- `cargo search tauri --limit 1`: crates.io DNS 조회 실패.
- `cargo check --offline --manifest-path src-tauri/Cargo.toml`: 캐시에 `tauri`가 없어 실패.
- `cargo tauri build --bundles deb`: Tauri CLI가 없어 실패. DEB/AppImage는 생성되지 않았다. Windows MSI/EXE 역시 빌드하지 않았다.
- `python3 -m json.tool src-tauri/tauri.conf.json`, `cargo metadata --offline --no-deps`, `rustfmt --check` 및 아이콘 형식 검사: 통과. 이는 Tauri 설정 스키마·Rust 컴파일·보안 동작을 검증한 것이 아니다. Cargo.lock도 생성하지 못했으며 의존성 버전은 아직 잠기지 않았다.
- `cd frontend && npm run typecheck && npm run build && node tests/offline.test.mjs`: 통과. 정적 렌더링 테스트만 수행했다.

## 이어서 검증하는 방법

네트워크와 관리자 권한이 있는 일반 Ubuntu 환경에서 [Tauri 사전 요구사항](https://v2.tauri.app/start/prerequisites/)의 Debian/Ubuntu 개발 패키지를 설치한다. 예시:

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
cargo install tauri-cli --version '^2' --locked
cd /home/jiwon/autonomous-uav-system/frontend
npm ci
cd ..
cargo tauri dev
cargo tauri build --bundles deb
```

위 명령은 아직 이 환경에서 성공을 확인하지 못했다. 최초 빌드 후 Cargo.lock을 검토·보존하고 버전과 보안 설정을 고정해야 한다. 빌드에 성공하더라도 DEB 설치 여부, 설치 후 개발 서버 없이 실행되는지, USB 드라이버/권한은 각각 따로 시험해야 한다. 사용자 시스템에 패키지를 설치하지 않았다.

Windows 11 x64는 네이티브 빌드 및 테스트가 필요하다. WebView2 오프라인 설치 방식 설정은 초안이며 실제 설치와 크기를 확인해야 한다. UI의 1280×800, 1440×900, 1920×1080, 2560×1440, 전체 화면, 9개 메뉴 및 패널의 실제 상호작용도 미검증이다.

## 다음 단계

Stage 2 완료 전 실제 창과 설치 패키지 검증을 끝내야 한다. 이후 Stage 3에서는 공식 MAVSDK/PX4 문서를 검증하고 별도의 읽기 전용 연결·상태 설계, USB 탐지 및 플랫폼 권한, 명령 차단을 준비한다. 실기 시험은 프로펠러 제거 등 별도 승인된 안전 절차에서 진행한다. 이번 단계에서 드론 통신, 비행 및 파라미터 변경은 수행하지 않았다. 기존 main·보관 브랜치와 bundle을 보존하고 원격 push는 하지 않았다. 커밋 정보는 `git log -1`로 확인한다.

데스크톱 초안과 관련 README·ignore 수정은 실행 검증 전이므로 커밋하지 않고 작업 트리에 남겼다. 프론트엔드 버전 표시와 이 보고서만 로컬 커밋한다.
