# Stage 1 — Figma 기반 관제 UI

## 범위 및 정리

`dev/firescene-gcs`에서 작업했다. `main`, `archive/pre-firescene`은 기존 `dc72c5e`를 유지하고 기존 Git bundle을 `git bundle verify`로 확인했다. 개발 브랜치에서 추적되던 `dashboard/` 전체, `dashboard_bridge/` 전체, `config/README.md`, `experiments/README.md`, `src/README.md`를 제거했다. 삭제 파일의 전체 목록은 Git 변경 내역에서 확인할 수 있고 보관 브랜치 및 bundle에서 복구 가능하다. 기존 dashboard는 React/Vite와 HTTP `/api/status` 프록시에 의존했고 bridge는 ROS2 `rclpy` 및 `px4_msgs`에 의존한다. 실제 MAVLink 구현은 없었다. 단조 시계와 수신 시간 초과를 이용한 상태 판단 아이디어는 후속 연결 계층에서 검토한다. 기존 알고리즘·Docker 파일은 조사한 로컬 기준 커밋에 없었다. 무시된 기존 `dashboard/node_modules`는 삭제하지 않았다.

`.gitignore`, `docs/README.md`를 유지하고 README를 갱신했다. Stage 0 보고서 및 references는 로컬에 보존한다. references는 `.git/info/exclude`에 의해 로컬에서 제외되며 원본 자료를 커밋하지 않는다. 다른 독립 프로젝트를 변경하지 않았다.

## 구현

Figma ZIP의 React `App.tsx` 아이콘/브랜드와 Soft Neutral 색상·CSS·구조를 재사용하고, 화면 비교 메뉴와 샘플 시나리오를 제거했다. 다른 시안은 원본 Figma ZIP에 보존했다. 9개 메뉴는 React 상태로 선택되며 종합 관제를 제외한 메뉴는 개발 예정 화면이다. 고도·속도·배터리는 `—`, GPS는 미수신, 위치·모드·ARM은 확인 불가, 텔레메트리는 미연결로 표시한다. 지도는 실제 지도가 아니므로 빈 작업 공간과 개발 예정 문구만 표시한다. 기체 명령은 구현되어 있지 않으며 잠금 상태가 표시된다. 서버, 시리얼 및 명령 경로 자체가 없다. 향후 실제 백엔드를 추가할 때 별도의 명령 차단과 검증이 필요하다.

Figma의 Google Fonts 요청을 제거하여 오프라인 시스템 글꼴로 대체했다. 외부 글꼴을 배포하지 않으므로 글꼴 라이선스를 추가하지 않았다. 시스템 폰트에 따라 원본 타이포그래피와 차이가 있다. Figma Make 전용 Vite 설정을 채택하지 않았으며, 기존 대시보드에서 사용한 로컬 캐시의 React/Vite 의존성을 사용했다. 백엔드, 설정 및 Tauri의 빈 폴더는 만들지 않았다.

## 검증

- `npm ci --offline --no-audit --no-fund`: esbuild 설치 스크립트의 `spawnSync`가 `EPERM`으로 실패. `npm ci --offline --ignore-scripts --no-audit --no-fund`로 캐시에서 설치 후 빌드 성공. 일반 환경에서는 `npm ci` 사용.
- `npm run typecheck`: 통과.
- `npm run build`: 통과, Vite 7.3.7 프로덕션 산출물 생성.
- `node tests/offline.test.mjs`: 서버 렌더링 검사 통과. 9개 메뉴 라벨, 미연결 상태 및 가상 값 부재 확인. 메뉴 클릭과 실제 브라우저 상호작용 검사는 아니다.
- `npm run dev -- --port 5173`: 환경의 로컬 포트 바인딩 제한으로 `listen EPERM` 실패.
- Chrome headless: `setsockopt: Operation not permitted`로 시작 실패. GUI 화면, 스크린샷, 메뉴 클릭과 시각적 일치도는 미검증이다.
- 실제 기체 연결 및 비행 명령은 실행하지 않았다.

## 현재 상태와 Stage 2

소스 정리, 초기 화면, 타입·빌드는 준비되었다. 실행과 GUI 검증이 차단되어 Stage 1의 실행 및 전체 메뉴 동작 완료 기준은 아직 검증되지 않았다. 일반 Ubuntu 세션에서 `cd frontend && npm ci && npm run dev`를 실행해 9개 메뉴, 패널 열기/닫기, 1280×800 및 오프라인 상태를 직접 확인해야 한다. Stage 2에서는 공식 문서와 라이선스를 확인한 뒤 Tauri 2, 통신 프로세스와 운영체제별 패키징을 설계하고 연결 확인 및 telemetry를 단계별 검증한다. 설치 파일, USB 자동 탐지, 연결, 지도, 카메라, 3D, 임무, 자율비행 및 명령은 미구현이다. GitHub 원격 조회 및 push는 하지 않았다.

로컬 커밋은 이 보고서와 함께 생성되며 `git log -1`로 확인한다. 부모 커밋은 `dc72c5e`다.
