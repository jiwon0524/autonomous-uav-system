# FireScene GCS 디자인 확정 v1.0

공식 디자인 기준 구현은 `86971e8`이다. Aerospace Dark 기본, Soft Neutral Day 보조 테마 및 [디자인 시스템](design-system.md)의 토큰, 글꼴, 아이콘, 간격, 상태 표현, 지도 중심 관제와 9개 메뉴를 후속 개발의 기준으로 한다. SpaceX 공개 Crew Dragon과 웹캐스트 디자인, QGroundControl의 비행 관제 개념을 참고하되 소방 드론 기능과 브랜드는 FireScene 요구에 맞게 독립적으로 개발한다. 안전성 또는 사용성 개선은 근거와 검증을 기록해 반영할 수 있다.

## 2026-11-24 검증

- 기준점 `86971e8` 대비 UI와 Tauri 소스 변경 없음. 디자인 확정 작업에서 화면을 변경하지 않았다.
- TypeScript 검사, 프론트엔드 빌드, `node tests/offline.test.mjs` 통과. 정적 렌더링 테스트는 9개 메뉴 라벨, 미연결·확인 불가 및 명령 잠금과 대표 예시 값 부재를 확인한다.
- Dark 기본값, Day 전환 코드와 두 테마 토큰을 소스에서 확인했다. 이전 리뉴얼 검증에서 Chrome headless의 Day 전환 및 비행 제어 이동을 확인했다. 이번 검증에서 전체 메뉴의 실제 클릭과 운영체제 WebView는 시험하지 않았다.
- `cargo tauri build --bundles deb` 성공: 릴리스 실행 파일 및 `src-tauri/target/release/bundle/deb/FireScene GCS_0.1.0_amd64.deb` 생성. 패키지 크기 2,983,730 bytes, SHA-256 `02c54c135531cad003feb59f30dec794469fba8a5ea0f345e460b9f1e1b928e0`. `dpkg-deb --info`로 amd64, 버전 0.1.0 확인.
- 이번 작업에서 실제 Tauri GUI 표시·상호작용과 DEB 설치·설치 후 실행은 검증하지 않았다. 이전 리뉴얼의 headless 렌더링 및 실행 파일 시작 검증은 [기록](aerospace-design-renewal.md)을 참고한다.

이 문서 커밋에 로컬 `design-v1.0` 태그를 지정한다. 기존 커밋과 `main`, `archive/pre-firescene` 브랜치는 변경하지 않으며 원격에 push하지 않는다. 개인 참고 자료와 미추적 Stage 0 조사 기록은 커밋하지 않는다.
