# Phase 6 통합 점검 — 초기 체크포인트

## 2026-11-26 기능별 대조 (최신)

다음은 현재 트리의 구현 위치와 이력 대조다. 코드 존재는 물리 기체나 Windows 시험 성공을 뜻하지 않는다. 기존 Phase 4의 18개, Phase 5의 20개 판정을 수정하지 않는다.

| 영역 | 코드와 이력 | 검증 및 범위 |
|---|---|---|
| 설치 | `src-tauri/tauri.conf.json`, `installer/firescene.nsi`; `79fb17a`, `d3ea440`, `60b4cb3` | NSIS 페이지 하네스만 컴파일. 전체 Windows 설치·업데이트는 NOT TESTED. DEB 설정 존재. |
| 9개 메뉴·테마 | `frontend/src/App.tsx`, `index.css`; `aa4ce97`, `f382731` | 최종 배경색 포함 격리 GUI 48/48; 실제 Windows는 별도. |
| 지도·검색·임무 | `MapView.tsx`, `locationSearch.ts`, `Missions.tsx`, `src-tauri/src/map.rs`, `missions.rs`; `a15c945`, `1b22aa5` | OpenLayers/VWorld 및 격리 검색·합성 임무 GUI; MapLibre는 개발 실험. 외부 API 가용성 보장 아님. |
| 연결·COM·UDP | `connection.rs`, `Connection.tsx`; `586084f`, `188bbe4`, `6877f21` | 읽기 전용 직렬·UDP 및 진단 구현, 합성 연결 GUI. 물리 COM·KOA FC 미검증. |
| 텔레메트리·계기 | `telemetry.rs`, `Telemetry.tsx`; `7677eec`, `49bdf11`, `a347168` | 지원 범위와 누락은 `phase4-telemetry-spec.md`; 합성 GUI와 일부 경계 단위 시험. 전체 조합·실제 데이터 미검증. |
| 안전·점검 | `safety.rs`, `Safety.tsx`, `Preflight.tsx`; `71aa5f7`, `ffd2501` | 읽기 전용 관찰과 수동 확인. 실제 비행 준비 판정 아님. |
| Pi·카메라·탐지·점군 | `companion.rs`, `camera.rs`, `map3d.rs`, 대응 프런트엔드; `45cf830`, `a9e5e07`, `5dc7416` | 수신·표시 인터페이스 및 합성 fixture. 팀원 알고리즘, SLAM·실제 3D 재구성 구현을 주장하지 않는다. |
| 기록·재생 | `recording.rs`, `Recordings.tsx`; `e394239` | 격리 합성 GUI 기록·재생과 재시작 보존 PASS. 실제 기체 기록은 미검증. |
| 가상 제어·명령 잠금 | `virtual_control.rs`, `main.rs`, `VirtualControl.tsx`; `08385c6`, `8afb43e`, `684b035` | 개발 simulation 한정 가상 제어, 500ms 임대·50ms watchdog·128 큐; 두 release 정상 IPC 대조 및 가상 IPC 8개 미등록. 실제 명령 송신은 범위 밖. |

기존 최초 실행 상태 키·テーマ·임무·기록 저장 구조는 브랜드 변경에서 변경하지 않았다. 사용자 데이터의 실제 Windows 업데이트 보존은 여전히 외부 시험이다.

2026-11-26. 기준선은 `phase5-safety-manual-control`의 `684b035`이며 새 로컬 브랜치 `phase6-integrated-release`에서 작업한다. 이 문서는 초기 조사 기록이며 Phase 6 완료 판정이 아니다.

## 확인한 근거

- Phase 2–5 설계·검증 문서와 Git 이력에 단계별 구현이 기록되어 있다. 특히 `aa4ce97`, `3bf9abd`, `a347168`, `ffd2501`, `684b035`는 각 단계의 이력 근거다. 이력의 존재만으로 개별 기능의 현재 동작을 증명하지 않는다.
- `src-tauri/tauri.conf.json`은 기존 식별자 `org.firescene.gcs`, 현재 사용자 NSIS 설치, 한국어·영어, WebView2 다운로드 부트스트래퍼, 아이콘과 헤더·사이드바 이미지를 설정한다. 사용자 지정 NSIS 템플릿은 설정되어 있지 않다. 앱 버전은 0.1.0이다.
- `frontend/src/FirstRun.tsx`와 `setupPreferences.ts`에는 언어·테마 선택, USB·지도·읽기 전용 안내, 건너뛰기, Tab 순환과 Escape 및 완료 상태 키가 있다. 기존 완료 키는 `firescene.setup.complete.v1`이다. 나머지 앱의 언어는 현재 한국어다.
- Phase 4 텔레메트리의 지원 범위 및 검증 공백은 `phase4-telemetry-spec.md`에 정리되어 있다. STATUSTEXT 다중 청크 재조립은 지원하지 않는다. 긴 상태 문구가 분리되어 문맥을 잃을 수 있으므로 안전 메시지 해석에 주의가 필요하며, 변경 전 실제 제품 영향과 위험을 평가해야 한다.
- Phase 5 최종 보고서는 소프트웨어 기준 PASS 18, PARTIAL 2를 기록한다. 두 잔여는 실제 개발 Tauri 가상 세션의 반대 키 동시 입력과 장시간 이벤트 순서·중복·누락이다. 기존 결과는 실제 FC 비행 검증이 아니다.
- Windows 워크플로는 수동 실행, Windows 2022 네이티브 NSIS 빌드 및 체크섬 아티팩트를 정의한다. 워크플로 정의는 이번 브랜치의 실행 성공이나 실제 설치 화면 검증을 뜻하지 않는다.

## 이번 체크포인트 검증

기준선에서 프런트엔드 시험 18개 파일 PASS, TypeScript 검사 PASS. 제품 코드 변경은 없다. 기존 Phase 4의 18개 및 Phase 5의 20개 판정은 변경하지 않았다.

## 남은 작업

요청된 세부 기능별 코드·이력 대조, Phase 4·5 잔여 시험, 디자인 조사·구현, 최종 회귀 및 패키징은 미완료다. 기존 로컬 태그 목록에 `v0.1.0-hardware-preview.1`은 없으며 공개 Release 확인은 별도로 필요하다. 실제 Windows, KOA FC, 물리 COM·키보드, 실제 텔레메트리 및 격리 PX4 SITL은 NOT TESTED다. 미추적 `docs/stage0-cleanup-report.md`는 보존한다.
