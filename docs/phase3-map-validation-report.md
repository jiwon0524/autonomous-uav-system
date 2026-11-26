# Phase 3 검증 — 진행 기록

이번 단계는 Phase 3 완료가 아니다. TypeScript, 프런트엔드 테스트 및 빌드와 Rust 기본·simulation 및 Tauri 권한 테스트를 실행했다. 지도 실제 인증, 하드웨어, Windows DPI, 지도 엔진 실험 및 성능 비교는 미검증이다. 기존 3,749,493 bytes 설치 파일은 하드웨어 진단 빌드의 기준이며 이번 코드로 Windows 설치 파일을 생성하지 않았다. 프런트엔드 빌드의 최대 JS 청크는 664.65 kB, gzip 198.93 kB이고 500 kB 경고가 남는다. 별도 GUI 결과는 완료 후 추가한다.

GUI 검증: Xvfb와 실제 Tauri/WebKitGTK, 격리 프로필에서 910×512, 1093×614, 1366×768, 1440×900, 1920×1080, 2560×1440 매트릭스 및 메뉴·테마·집중 보기가 통과했다. 키를 사용한 별도 실행에서 장소 검색, 임무 초안·복제, 합성 기체, 안전 및 기록·재생이 통과했다. 초기 오프라인 입력 연습은 대기 시간 초과였으나 xdotool로 테스트 창 포커스를 명시한 재실행에서 통과했다. 레이어 자동화의 기존 첫 번째 비활성 버튼 선택자가 추가된 도구 버튼과 충돌해 수정했으며 재실행에서 통과했다. 합성 telemetry는 실제 KOA FC 성공이 아니다. 증거 경로: `~/FireScene-evidence/phase3/`.

최종 포커스 지정 GUI 실행은 모든 기존 시나리오를 통과했다. 여섯 크기 매트릭스는 별도 실행에서 통과했다. 새 검색 위치 적용 버튼 및 방향 삼각형 자체를 실제 기체로 검증한 것은 아니며, 방향 유효성 판단은 단위 테스트로 확인했다. GUI 도구는 테스트용 도구이며 배포물에 포함하지 않는다.

2026-11-26 추가: 실제 타일 평면·Globe는 격리 Chrome SwiftShader에서만 확인했다. Tauri 통합, 실제 기체·임무 마커, 엔진 간 상태 이전, 다섯 지역 품질과 동조건 3회 성능 비교는 미실시다. 초기 Dark 시각 검사에서 도로 표식 대비 결함이 발견돼 수정했고 Chrome SwiftShader에서 다시 확인했다. 다른 지역과 Tauri 검증은 남았다. 따라서 기본 OpenLayers가 유일한 안정 후보이며 24개 완료 기준 전체 PASS로 평가할 수 없다. 기존 24개 항목의 번호별 원문 목록은 현재 저장소에서 확인되지 않아 임의로 재구성하지 않았다.

## 2026-11-26 사용자 지정 신규 24개 기준 (진행 중)

이 목록은 과거 24개 기준의 복원이 아니라 이번 요청의 신규 기준이다. 실제 기체·Windows GPU·웹서비스 인증은 별도 외부 시험이다.

| 번호 | 기준 | 판정 | 근거 또는 남은 범위 |
|---|---|---|---|
| 1 | 기존 연구 및 설계 반영 | PARTIAL | 기본 화면과 실험 경계 반영, 비교 미완료 |
| 2 | 통합 장소 검색 | PASS | 기존 VWorld 실제 Tauri 검색 시험 |
| 3 | 검색 결과 선택과 임무 저장 분리 | PASS | Tauri 검색 후 별도 현장 선택 및 Global 표시 확인 |
| 4 | 건물 조회 실패 시 기존 임무 보호 | PARTIAL | 코드 수정, 오류 주입 필요 |
| 5 | Tactical Map 기본 조작 | PARTIAL | 구현, 신규 버튼 개별 검증 필요 |
| 6 | 기체 위치 및 방향 표시 | PARTIAL | 합성 기체의 Global 방향 GUI 확인, 실제 기체 미검증 |
| 7 | 이동 궤적 및 데이터 최신성 | PARTIAL | 기존 구현, 교차 엔진 상태 유지 미검증 |
| 8 | 임무 건물 및 관측 범위 표시 | PARTIAL | OpenLayers 구현, Global 외곽선·범위 전달 추가 후 검증 필요 |
| 9 | Satellite View | PARTIAL | 기존 전환 GUI 통과, 통합 상태 이관 미검증 |
| 10 | 지도 공통 상태 및 좌표 검증 | PARTIAL | 기존 WGS84 데이터 사용, 공통 카메라·궤적 어댑터 미완성 |
| 11 | 실제 한국 벡터 타일 렌더링 | PARTIAL | Chrome과 Tauri WebKitGTK에서 실제 타일 관찰, 품질·오류 검증 필요 |
| 12 | 부산 5개 지역 지도 품질 비교 | NOT TESTED | 동조건 비교 및 화면 필요 |
| 13 | 한글 라벨 가독성 | PARTIAL | Chrome 일부 부산 화면, 5개 지역 미비 |
| 14 | FireScene Dark 지도 스타일 | PARTIAL | Tauri 개발용 Dark 적용과 라벨 대비 보정, 다지역 검증 미비 |
| 15 | FireScene Day 지도 스타일 | NOT TESTED | 제공자 기본 스타일 |
| 16 | 실제 Global Globe 렌더링 | PARTIAL | Chrome 및 Tauri WebKitGTK 실제 타일 확인, 통합·Windows 검증 필요 |
| 17 | Global ↔ Tactical 상태 이관 | PARTIAL | 기본 지도를 유지하며 overlay 복귀, 상태 전반 미검증 |
| 18 | 실제 Tauri/WebKitGTK 호환성 | PARTIAL | 타일 안정화 후 지구본·평면·복귀 및 현장 마커 확인, 반복·오류 검증 필요 |
| 19 | 지도 오류 격리 및 기본 지도 복귀 | PARTIAL | 합성 context-lost 이벤트 후 경고·복귀 통과, 타일·glyph·실제 GPU 손실 미검증 |
| 20 | 지도 공급자 이용 조건 및 출처 표시 | PARTIAL | OpenFreeMap 공식 자료 예비 검토, 배포 조건 종합 필요 |
| 21 | Prior 3D 구현 가능성과 실제 검증 범위 | PARTIAL | 기존 실험만, 인증·실제 모델 미검증 |
| 22 | 지도 엔진 성능 비교 | NOT TESTED | 동일 조건 반복 측정 없음 |
| 23 | 전체 GUI 및 자동화 회귀 시험 | PARTIAL | 기존 Phase 3 GUI 자료 보존, 여섯 크기와 기존 GUI 시나리오 통과, 새 기능 전체 범위는 미완료 |
| 24 | 하드웨어 진단 기능과 비행 명령 잠금 유지 | PARTIAL | 코드 보존, 실기체·Windows 미검증 |

GPU 조사: 커널 7.0.0-43-generic, PCI 10de:2f04, nvidia 커널 모듈 580.178.04 로드. 이 실행 환경에서 `nvidia-smi`는 드라이버 통신 실패, `/dev/dri`와 일반 `/dev/nvidia*` 노드는 확인되지 않았다. Xvfb의 소프트웨어 렌더링과 물리 GPU 가속을 동일시하지 않는다. 시스템 변경은 하지 않았다.

Tauri 개발 모드 GUI: 실제 검색 결과를 현장으로 선택하고 Global에서 선택 마커를 확인했다. 기체가 없는 경우 기체 마커가 없는 화면을 확보했다. `~/FireScene-evidence/phase3/tauri-mission-global/global-experiment.png`. 여섯 창 크기 전체 매트릭스와 기존 전체 시나리오는 `tauri-globe-matrix`에서 통과했다. 관측 범위·건물 외곽선의 실제 시각 확인 및 오류 주입은 미완료다.

Tauri 후속 시험: 실제 검색 결과의 명시적 현장 선택 → Global 위치 표시 → 현장 확대 → 평면/지구본 → 기존 지도 복귀 후 현장 보존을 검증했다. 화면 `~/FireScene-evidence/phase3/tauri-mission-roundtrip/global-mission-zoom.png`. 저장된 건물 외곽선·범위의 실제 데이터와 기체 방향·궤적은 이 시험에 포함되지 않는다. 배포 후보는 A (OpenLayers 기본 + 개발 실험)이며 새 설치 파일과 Windows 크기는 측정하지 않았다.

## 2026-11-26 후속 구현·시험

개발 전용 Global은 Tactical의 현재 세션 궤적을 WGS84로 전달하며 같은 기체의 최신 heading이 있을 때 방향 삼각형을 표시한다. GPS가 만료·연결 해제되면 현재 기체 표시와 궤적을 비운다. Xvfb 실제 Tauri에서 합성 telemetry 방향 삼각형 및 simulation 출처 표시를 확인했다. 증거: `~/FireScene-evidence/phase3/finalization/global-simulation-heading.png`. 실제 KOA FC는 사용하지 않았다. 여섯 창 크기 및 기존 GUI 매트릭스 통과. Dark 지도 글자 외곽선 대비 수정과 context-lost 이벤트 주입 결과는 후속 시험 결과에 따라 추가한다. 이 변경은 생산 지도 엔진을 바꾸지 않는다.

24개 기준의 기존 항목 및 판정표는 위에 그대로 남아 있으며 실험 기능의 외부 검증과 5개 지역 비교·Day 스타일·동조건 성능 비교가 미완료이므로 Phase 3 전체 완료로 판정하지 않는다. 배포 후보는 A다. 외부 시험과 개발 빌드 주의사항은 `phase3-windows-followup.md`를 참조한다.

최종 GUI 보강: `finalization-fault/results.json`에서 검색·Global 복귀, 합성 기체 방향, 합성 context-lost 경고·복귀, 기록·안전 시나리오가 통과했다. Dark 보정 화면은 `finalization-fault/global-experiment.png`다. 합성 DOM 이벤트는 실제 GPU 손실을 재현하지 않는다. 프런트엔드 15개, Rust 기본 49개·simulation 50개와 각 권한 테스트 통과, 생산 빌드 주요 JS 664.87 kB (gzip 199.05 kB). 각 Rust 구성의 벤치마크 1개는 ignored다.

잔여 분류: P0 확인 없음. P1 범위는 개발 Global의 미완성 모드 이관·오류 및 타일 품질 검증으로, 생산 빌드에서 제외해 기본 관제 후보를 보호한다. P2는 Dark 지도 라벨·작은 창 추가 시각 조정과 기존 번들 크기 경고다. 현재 환경에서도 부산 5개 지역 비교, Day 전용 스타일, 성능 반복 측정과 실제 외곽선·범위 GUI 검증이 남았으므로 개발 전체 완료를 주장하지 않는다. Windows/WebView2·실기체·GPU 가속·Prior 3D 인증은 외부 환경 항목이다.

최종 재실행 `finalization-verified/results.json`에서도 실제 Tauri context-lost 합성 주입과 복귀, 합성 기체 방향, 기록 및 안전 시나리오가 통과했다. 지도 회전과 기체 방향의 정량 비교, 이동 궤적 및 실제 외곽선·범위 시각 증거는 아직 없다. Dark의 지구본 축척 및 래스터 기반 부분에서는 일부 지명 대비가 부족해 스타일 완성으로 보지 않는다. 여섯 크기 매트릭스는 이전 `finalization` 실행에서 수행했으며 마지막 회전 정렬 보정 이후 전체 매트릭스를 반복하지 않았다.
