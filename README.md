# FireScene GCS

FireScene GCS는 실제 드론 관제를 위한 데스크톱 앱의 초기 프론트엔드입니다. 현재 화면은 Figma Soft Neutral 시안을 기반으로 한 미연결 상태이며 통신, 지도와 비행 명령은 구현되지 않았습니다.

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

왼쪽 메뉴에서 9개 작업 공간을 선택할 수 있습니다. 종합 관제 외 메뉴는 개발 예정 화면입니다. 수신된 기체 데이터가 없으므로 연결 상태, 위치, 모드 및 ARM 상태는 미연결 또는 확인 불가로 표시합니다. 실제 명령이나 기체 연결 기능은 없습니다.

## 구조

- `frontend/`: React/TypeScript 관제 UI 및 초기 상태 검사
- `docs/`: 개발 기록
- `references/`: 로컬 전용 Figma 및 리서치 원본. 공개 및 커밋 전 별도 검토 필요

향후 Tauri 2 데스크톱 셸과 독립 통신 계층을 검토합니다. ROS2는 현재 실행에 필요하지 않습니다. 실제 기체 명령은 향후 UI와 백엔드 양쪽에서 차단 정책을 마련하고 검증하기 전까지 구현하지 않습니다.

## 데스크톱 통합 준비 (검증 전)

`src-tauri/`에 Tauri 2 창과 배포 설정 초안을 추가했습니다. 아직 Tauri 의존성 설치, 데스크톱 빌드 및 창 실행을 검증하지 못했으므로 설치형 제품으로 취급하지 마세요. 개발용 Ubuntu 시스템 패키지 및 빌드 방법은 [Stage 2 보고서](docs/stage2-desktop-report.md)를 참고하세요. 프론트엔드 개발 서버 실행과 설치된 앱 실행은 별개입니다.
