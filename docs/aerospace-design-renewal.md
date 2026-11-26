# Aerospace 디자인 리뉴얼

## 기준과 참고

디자인 변경 전 로컬 커밋 `0a5e494`에 Tauri 설정, Cargo.lock과 관련 기존 변경을 보존했다. Figma Soft Neutral 원본과 로컬 기술·디자인 연구를 확인했다. 연구의 미연결/unknown 구분, 상시 상태, 명령 잠금 및 색상과 문구 병기 원칙을 유지한다. 원본 참고 자료와 Stage 0 로컬 조사 기록은 커밋하지 않았다.

공개 자료: [Crew Dragon Displays](https://www.shanemielke.com/work/spacex/crew-dragon-displays/)는 유인 우주선 디스플레이 프로젝트, [SpaceX Webcast](https://www.shanemielke.com/work/spacex/webcast/)는 공개 중계 그래픽 프로젝트다. 명확한 상태 계층과 절제된 데이터 표현을 참고해 독립적인 FireScene 화면을 구성했다. [Andrew Fitzpatrick 포트폴리오](https://andrewjfitzpatrick.com/portfolio-spacex.html)는 접근 시 401이 반환되어 내용을 확인하지 못했다. 비공개 화면이나 SpaceX 브랜드는 사용하지 않았다.

## 변경

기본 Aerospace Dark와 Soft Neutral 계열 Day를 CSS 의미 토큰으로 관리한다. 배경, 표면, 텍스트, 강조, 경고, 위험, 비활성, 선, 간격, 반경과 시스템 글꼴을 공유한다. 상단에 핵심 상태를 고정하고 지도 중심 종합 관제에 비행 데이터, 임무 및 이벤트의 빈 상태를 배치했다. 비행 제어에는 자세와 고도·상승/하강 속도 영역을 마련하되 값이나 자세를 만들어 표시하지 않는다. 9개 메뉴, 세부 패널, 미연결 상태 및 명령 잠금을 유지한다. 다른 작업 공간은 개발 예정으로 유지한다. 테마 선택은 현재 세션 상태이며 앱 재시작 시 Dark로 돌아간다. 실제 연결, 시뮬레이션, stale 판정과 경고 로직은 구현되지 않았다.

## 검증

- `npm run typecheck`, `npm run build`, `node tests/offline.test.mjs`: 통과.
- `cargo tauri build --no-bundle`: 성공, 릴리스 실행 파일 생성. 패키지 재생성은 수행하지 않았다.
- 릴리스 실행 파일은 GUI 환경에서 시작해 8초 동안 즉시 종료되지 않았다. Tauri 창의 실제 표시와 상호작용은 직접 확인하지 못했다.
- 로컬 Chrome headless 1440×900: Dark 종합 관제 및 Day 비행 제어 렌더링 확인. Day 전환과 비행 제어 이동, 자세 미수신 및 명령 잠금 문구를 확인했다. 운영체제 WebView, 전체 메뉴와 창 크기별 수동 검증은 남았다.

복구 시 현재 변경을 보존한 뒤 `git switch --detach 0a5e494`로 디자인 이전 상태를 별도로 확인할 수 있다. 기존 브랜치를 이동하거나 덮어쓰지 않는다.
