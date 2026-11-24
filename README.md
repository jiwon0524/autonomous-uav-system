# FireScene GCS

FireScene GCS는 실제 드론 관제를 위한 데스크톱 앱의 초기 프론트엔드입니다. 현재 화면은 Aerospace Dark를 기본으로 하며 Soft Neutral Day 테마를 지원합니다. 기체 통신, 실제 지도와 비행 명령은 구현되지 않았습니다.

## 개발 환경 및 실행

Node.js와 npm이 필요합니다. 저장소 루트에서:

```bash
cd frontend
npm ci
npm run dev
```

브라우저에서 Vite가 표시하는 로컬 주소를 엽니다. 검증 명령:

```bash
npm run typecheck
npm run build
node tests/offline.test.mjs
```

왼쪽 메뉴에서 9개 작업 공간을 선택할 수 있습니다. 비행 제어는 미수신 상태 화면이며 나머지 기능은 개발 예정입니다. 수신된 기체 데이터가 없으므로 연결 상태, 위치, 모드 및 ARM 상태는 미연결 또는 확인 불가로 표시합니다. 실제 명령이나 기체 연결 기능은 없습니다.

## 구조

- `frontend/`: React/TypeScript 관제 UI 및 초기 상태 검사
- `docs/`: 개발 기록
- `references/`: 로컬 전용 Figma 및 리서치 원본. 공개 및 커밋 전 별도 검토 필요

Tauri 2 데스크톱 셸을 사용하며 독립 통신 계층은 후속 검토 대상입니다. ROS2는 현재 실행에 필요하지 않습니다. 실제 기체 명령은 향후 UI와 백엔드 양쪽에서 차단 정책을 마련하고 검증하기 전까지 구현하지 않습니다.

## 공식 디자인 및 데스크톱

디자인 기준 구현은 `86971e8`이며 Aerospace Dark와 Soft Neutral Day를 사용합니다. 향후 화면은 [디자인 시스템](docs/design-system.md)과 [디자인 확정 기록](docs/design-freeze-v1.md)을 따릅니다. 사용성 또는 안전성 개선은 근거와 검증을 기록해 반영합니다.

저장소 루트에서 `cargo tauri dev`로 개발 창을 실행하고 `cargo tauri build --bundles deb`로 Ubuntu 패키지를 빌드합니다. 빌드와 실제 GUI·설치 검증의 구분은 [디자인 확정 기록](docs/design-freeze-v1.md)을 참고하세요.
