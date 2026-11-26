# Phase 6 개별 평가 — 진행 중

2026-11-26. 소프트웨어·합성 GUI와 실제 Windows/FC 시험을 구분한다. 기존 Phase 4의 18개와 Phase 5의 20개 평가를 대체하지 않는다. 증거와 범위는 `phase6-integration-report.md`, `phase6-brand-validation.md`, `phase6-pending-validation.md`, `phase6-installer-design.md` 참조.

| 번호 | 기준 | 판정 | 근거와 잔여 |
|---|---|---|---|
| 1 | Phase 1–5 통합 | PARTIAL | 코드·이력 대조 및 합성 GUI; 외부 장치와 세부 조합 미검증. |
| 2 | Phase 4 텔레메트리 | PARTIAL | 기존 경계·TTL 및 신규 자세/센서 단위 시험; 모든 조합과 실제 데이터 미검증. |
| 3 | 반대 방향 입력 | PASS | 실제 개발 Tauri 가상 세션의 합성 키·백엔드·종료 토큰 거부; 물리 키보드는 별도. |
| 4 | 장시간 이벤트 | PASS | 제한 개발 시험 870세션·618.8초·4,640건 순서/개수 일치 및 오류 거부. 무제한 무결성 보장 아님. |
| 5 | 브랜드 | PASS | 세 개 시각 목업과 선택 방향; Windows 실제 화면 별도. |
| 6 | 그래픽 | PASS | 자체 개념 SVG, 시작 및 온보딩 표시. 설계 도면 아님. |
| 7 | Windows Welcome | PARTIAL | NSIS 코드·하네스 및 목업; 전체 네이티브 빌드/실행 미검증. |
| 8 | Windows Configuration | PARTIAL | 기존 실제 기능·브랜드 헤더; 본문 기본 MUI, Windows 미검증. |
| 9 | Windows Deployment | PARTIAL | 기존 실제 진행과 브랜드 헤더; Windows 미검증. |
| 10 | Windows Completion | PARTIAL | MUI 다크 완료와 읽기 전용 문구; Windows 미검증. |
| 11 | 제거 | PARTIAL | 기존 선택적 데이터 삭제 및 브랜드 헤더; 실제 제거 미검증. |
| 12 | 스플래시 | PARTIAL | Ubuntu 개발 Tauri 영상과 전환; Windows 미검증. |
| 13 | 초기화 상태 | PARTIAL | React 마운트 연동 및 지연 재시도 단위 시험; 실제 오류 주입 미검증. |
| 14 | 온보딩 | PASS | Ubuntu GUI 첫 실행·언어·테마·건너뛰기·재시작 상태 보존; Windows 별도. |
| 15 | OS 아이콘 | PARTIAL | 기존 Windows/Ubuntu 아이콘 리소스 및 패키지 등록; Windows 실제 표시 미검증. |
| 16 | 9개 메뉴·테마 | PASS | 최종 배경색 포함 Ubuntu 개발 Tauri 48개 GUI. |
| 17 | 지도·주소 검색 | PARTIAL | OpenLayers/VWorld 및 키 제공 합성 GUI; 서비스·실환경 조합 전체 미검증. |
| 18 | 임무 | PARTIAL | 격리 생성·복제·재시작 GUI; 모든 지도 오류 조합 미검증. |
| 19 | 직렬·UDP | PARTIAL | 읽기 전용 코드 및 합성 UDP; Windows COM·실제 FC NOT TESTED. |
| 20 | 진단·복구 | PARTIAL | 합성 연결 손실/수동 복구; 실제 장치 NOT TESTED. |
| 21 | 텔레메트리·계기 | PARTIAL | 합성 GUI 및 일부 경계·TTL; 실제 기체 NOT TESTED. |
| 22 | 안전·점검 | PARTIAL | 읽기 전용 안전 및 합성 GUI; 실제 장치 관찰 NOT TESTED. |
| 23 | 카메라·Pi·점군 | PARTIAL | 수신 인터페이스·합성 카메라/점군 GUI; 실제 팀원 장치 미검증. |
| 24 | 기록·재생 | PARTIAL | 합성 GUI·재시작; 실제 기체 데이터 미검증. |
| 25 | 생산 격리 | PASS | 일반/feature release 실제 IPC 읽기 대조 및 가상 8개 미등록; 실기체는 별도. |
| 26 | 전체 GUI | PASS | Ubuntu 개발 Tauri 48/48; Windows는 별도. |
| 27 | 성능·안정성 | PARTIAL | 시작 중앙값 1.373→3.158초 관측; 빌드 조건 차이가 있어 원인 미확정. GPU 미측정. |
| 28 | Ubuntu 설치 | PARTIAL | DEB 빌드 및 추출 앱 IPC; 시스템 설치·최종 보안 확인 미완료. |
| 29 | Windows 설치·업데이트 | NOT TESTED | 하네스는 전체 Windows NSIS 빌드·설치·데이터 보존 증명이 아님. |
| 30 | 배포 보안·안내 | PARTIAL | 생산 IPC 검사, 패키지 민감 정보 점검 진행; 서명·최종 안내·보안 완료 전. |

실제 Windows 설치·DPI·물리 키보드, KOA FC·물리 COM·실제 HEARTBEAT·텔레메트리·연결 복구 및 격리 PX4 SITL은 NOT TESTED다. 미확인 영역에 대해 P0/P1 부재를 보장하지 않는다. 최초 패키지의 홈 경로 노출은 재매핑 빌드에서 검사 대상 원문 0건으로 수정했으나, 동일 버전 업데이트 정책과 Windows 설치는 미검증이다. 제한적 보안 검사만으로 모든 개인정보 부재를 보장하지 않는다.
