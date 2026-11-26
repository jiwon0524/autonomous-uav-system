# Phase 5 검증 보고 (진행 중)

2026-11-26. 브랜치 `phase5-safety-manual-control`은 `a347168`에서 생성했다. Phase 4 잔여 항목을 `phase6-pending-validation.md`에 이관했으며 기존 18개 판정은 변경하지 않았다.

안전 모니터의 battery, GPS fix, 위성, 센서 비트, roll/pitch 각각에 대해 미수신·INVALID·STALE이 NORMAL이 되지 않는 단위 시험 1개 PASS. 비행 전 점검 화면은 오프라인 상태·수동 확인·화면 이탈 후 초기화·명령 잠금을 실제 Tauri GUI에서 PASS했다 (`~/FireScene-evidence/phase5/preflight/final-results.json`). 프런트엔드 TypeScript, 18개 시험 파일 및 생산 빌드 PASS. 최초 프런트엔드 회귀는 모델과 컴포넌트의 대소문자 충돌로 실패했고 파일명을 분리해 재실행했다. 개발 전용 메모리 가상 모델의 입력 범위, 500ms 시간 초과, 세션 교체, 무효 입력 및 중립/잠금 시험 2개 PASS. 아직 IPC·GUI·실시간 권한에는 연결하지 않았으므로 가상 제어 종단 간 검증은 아니다. Rust 기본 58 PASS 및 simulation 61 PASS, 각각 1 ignored, 권한 시험 각 1 PASS. 안전 이벤트 확장·SITL 등은 아직 시험하지 않았다. PX4/Gazebo 바이너리 존재만 확인했다. 네트워크 네임스페이스 생성 사전 점검은 샌드박스 안과 권한 상승 실행 모두 EPERM으로 실패해 격리된 SITL을 시작하지 않았다. SITL 실행 성공이 아니다.

| 기준 | 판정 |
|---|---|
| 1 안전 아키텍처 정의 | PASS |
| 2 제어 권한 상태 관리 | PASS |
| 3 실제·시뮬레이션·재생 분리 | PASS |
| 4 입력 관리자 | PASS |
| 5 반대 방향 입력 처리 | PARTIAL |
| 6 키 해제 및 입력 초기화 | PASS |
| 7 포커스 상실 처리 | PASS |
| 8 입력 시간 초과 | PASS |
| 9 가상 제어 기능 | PASS |
| 10 가상 제어 안전 잠금 | PASS |
| 11 비행 전 점검 화면 | PASS |
| 12 센서 및 연결 안전 판단 | PASS |
| 13 비상 상황 모의 시험 | PASS |
| 14 안전 관리 UI | PASS |
| 15 안전 이벤트 기록 | PARTIAL |
| 16 실제 Tauri GUI 시험 | PASS |
| 17 기존 기능 회귀 시험 | PASS |
| 18 생산 및 개발 빌드 격리 | PASS |
| 19 실제 비행 명령 송신 차단 | PASS |
| 20 SITL 검증 범위 및 한계 기록 | PASS |

P0 발견 없음. P1: 가상 권한의 연결·재생·GUI 연동, 종단 간 입력 무효화·격리 및 불변 조건 시험 미구현/미검증. P2: Phase 4 이관 문서 참조. Phase 5 완료나 Phase 6 진입을 판정하지 않는다.

## 가상·연결 경합 보강 (진행 중)

개발 simulation 빌드에서 가상 세션 획득과 직렬·UDP 연결 시작을 공통 mutex로 보호한다. 가상 활성 중 각 연결 시작 거부, 기존 하드웨어 연결 중 가상 획득 거부, 동시 루프백 연결/가상 획득 10회에서 하나만 성공함을 단위 시험 2개로 확인했다. 가상 모델의 단조 순번은 지연·중복 입력이 중립을 덮지 못하게 한다. 추가로 상태 폴링 없이 입력이 500ms 뒤 도착하면 만료된 권한을 되살리는 문제를 수정했다. 모델 단위 시험 4개 PASS. 연결 잠금은 세션별 토큰을 사용하며 이전 세션의 늦은 해제가 새 세션 잠금을 풀지 못하는 시험도 포함한다. 연결 상호 배제는 아직 IPC에서 호출되지 않아 완성된 종단 간 보호가 아니다. 재생 차단, 이벤트 기록 및 가상 GUI는 미구현이다.

## 통합 복구 검증 (진행 중)

2026-11-26 중단된 작업 트리를 복구해 simulation 컴파일, 가상 제어/이벤트 Rust 5개, TypeScript, 프런트엔드 18개 시험과 생산 Vite 빌드를 확인했다. 실제 Tauri `--virtual-only`에서 다섯 모의 위험 버튼별 잠금·세션 종료와 각 세션의 `simulated_warning`, `safety_lock`, `simulated_warning_ended`를 확인했다 (`/tmp/firescene-phase5-recovery-virtual/`). 경고 종료는 모의 시험 종료를 뜻한다. 이 시험은 위험 주입 당시 활성 입력이나 지연 입력·복구 후 자동 재개 방지를 확인하지 않는다. 실제 Tauri `--replay-only`는 합성 기록 생성·재생 및 재생 중 가상 시작 거부를 확인했다 (`/tmp/firescene-phase5-recovery-replay/`). 샌드박스 내 Rust 전체 시험의 루프백 bind는 EPERM이었으며 권한 상승 재시험으로 구분한다.

P1: 백엔드 입력 만료는 요청 처리 시 평가되고 잠긴 가상 세션의 권한 해제는 프런트엔드의 종료 요청에 의존한다. 프런트엔드 중단 시 독립적인 권한 회수 요구가 충족되었다고 볼 수 없다. 위험별 활성 입력·지연 입력, 기록 실패, 포커스/최소화, 다른 창 IPC, 생산 IPC 거부 및 최종 전체 회귀는 미완료다. 위 20개 기준의 기존 판정은 유지하며 Phase 5 완료 및 Phase 6 진입을 승인하지 않는다.

루프백 접근이 가능한 환경에서 Rust 전체 재시험은 기본 58 PASS, simulation 67 PASS, 각각 1 ignored 및 Tauri 권한 시험 1 PASS였다. GUI 결과는 `~/FireScene-evidence/phase5/integration/firescene-phase5-recovery-virtual/`와 `firescene-phase5-recovery-replay/`에 보존했다.

## Watchdog 중간 검증

개발 simulation 빌드에서 50ms 감시 및 500ms 단조 입력 임대 기반 권한 회수를 추가했다. 웹뷰 이벤트 루프를 1.8초 정지한 실제 Tauri 시험에서 상태 조회 없이 0.505초에 권한 해제, 이전 토큰 입력 거부와 재생 권한 재획득을 확인했다 (`/tmp/firescene-phase5-watchdog-retry3/results.json`). 기존 IPC의 동기 이벤트 기록 및 창 상태 조회가 runtime mutex를 보유하므로 장시간 정지 시 watchdog 차단 가능성은 아직 해소되지 않았다. P1은 열린 상태이며 이 결과를 최종 안전 승인으로 해석하지 않는다.

입력 IPC의 창 상태 조회는 이후 runtime mutex 밖으로 이동했으며 가상 Rust 시험 6개 및 실제 Tauri 비응답 시험을 재실행해 PASS했다 (권한 회수 관측 0.545초). 동기 이벤트 기록의 mutex 점유 공백은 남는다.

## 이벤트 I/O 분리 작업 중

이벤트를 128개 유한 큐에 비차단 전달하고 별도 작업자가 파일 I/O를 수행하도록 변경했다. 큐 포화 또는 작업자 오류는 영구 실패로 표시하며 신규 시작을 거부하고 활성 입력을 잠근다. 가상 관련 Rust 8개 통과. 정지된 작업자와 포화 큐에서 권한 회수 함수를 확인했으나 실제 watchdog 스레드의 장시간 정지 결합 시험 및 최종 회귀가 남아 P1 해결은 아직 판정하지 않는다.

기록 작업자를 500ms 이상 정지시키고 실제와 같은 watchdog tick을 독립 실행한 단위 시험에서 I/O 재개 전 중립화·권한 회수 및 재생 권한 재획득 PASS. 가상 관련 Rust 10개 PASS. 실제 Tauri 비응답 시험은 이벤트 I/O 분리 후 0.527초에 회수 관측 PASS (`~/FireScene-evidence/phase5/finalization/firescene-phase5-io-isolation-gui/`). 이후 경고 원인 필드 변경에 대한 GUI 재시험과 실제 Tauri 기록 정지 결합 시험은 남았다. 이는 하드 실시간 보장이 아니다.

각 위험 주입 전 전진 속도 활성, 위험 후 백엔드 OFFLINE·속도 중립 및 이전 토큰 입력 거부를 실제 Tauri에서 확인했다. 다섯 모의 경고의 `reason`을 구분한다 (`~/FireScene-evidence/phase5/finalization/firescene-phase5-active-hazards-final/`). 위험 복구를 별도로 주입하지는 않았다. 변경 후 Rust 기본 58, simulation 72 및 각 권한 시험 1 PASS (각 1 ignored), 프런트엔드 18개와 TypeScript 포함 생산 빌드 PASS. 실제 기록·재생 GUI PASS (`firescene-phase5-io-replay/`). 아직 production release 실제 IPC, 다른 창 IPC 및 전체 GUI 회귀를 검증 중이다.

최신 전체 GUI 회귀에서 첫 실행, 9개 메뉴, 테마, 6개 크기와 합성 기록·재생은 통과했다. 개요 1200x800, 축소 개요 overflow, 오프라인 연습, 지도 키 없는 검색, 임무 지도 캔버스는 실패했다. 오프라인 연습 단독 재시험은 PASS이나 전체 실행 실패 원인은 확정하지 않았다. 지도 검색에는 키가 제공되지 않았다. 실패 원본과 재시험은 `~/FireScene-evidence/phase5/finalization/firescene-phase5-io-regression/` 및 `firescene-phase5-offline-retest/`에 보존했다. 일반 및 simulation release 빌드는 성공했으나 WebDriver의 비정상 origin 때문에 정상 `connection_status`까지 실패하여 가상 IPC 거부 시험은 NOT TESTED다. 이벤트 I/O와 runtime mutex의 직접 결합은 제거했으며 정지 주입 단위 시험은 통과했지만 실제 Tauri에서 기록 작업자 정지 결합 검증은 남는다. 안전 관련 중대한 미검증 공백으로 Phase 5 기준선 확정 및 Phase 6 진입을 보류한다.

실제 Tauri 격리 프로필에서 기록 worker가 FIFO 열기에서 정지했음을 `/proc`으로 확인하고 웹뷰를 1.5초 정지했다. FIFO를 풀기 전에 이전 토큰 거부와 재생 권한 획득, 이후 OFFLINE·중립 속도를 확인했다 (`~/FireScene-evidence/phase5/finalization/firescene-phase5-tauri-journal-stall-verified/results.json`). 상태 IPC 이전 재생 권한 획득을 확인하므로 상태 조회가 만료 처리를 유발한 결과가 아니다. 1.535초는 시험 관측 시점이며 최대 응답 시간 보장이 아니다. 기록 오류·포화·사용자 종료 경합의 실제 Tauri 시험은 미완료다.

일반 및 simulation release를 `tauri/custom-protocol`로 빌드한 후 패키지 앱 origin의 실제 IPC로 `connection_status` 성공을 양성 대조군으로 확인했다. 두 빌드 모두 가상 IPC 8개가 각각 `Command ... not found`로 거부됐다 (`~/FireScene-evidence/phase5/finalization/firescene-phase5-release-normal-8ipc/results.json`, `firescene-phase5-release-simulation-8ipc/results.json`). 이전 실패는 custom protocol 없이 개발 URL을 사용하는 빌드의 비정상 origin이었다. CSP나 ACL을 완화하지 않았다.

실제 Tauri 격리 프로필에서 이벤트 파일 열기 실패를 재현했다. 이전 입력 거부, 재생 권한 획득, OFFLINE·중립 속도 및 기록 오류를 이유로 신규 시작 거부 PASS (`~/FireScene-evidence/phase5/finalization/firescene-phase5-tauri-journal-failure/results.json`). 이는 실제 디스크 write 실패·큐 포화·종료 경합을 별도로 증명하지 않는다.

키를 격리 프로필에 제공한 전체 재시험에서 지도 검색 및 임무 작성·복제 PASS; 개요 1200×800, 전체 실행 중 오프라인 연습, 축소 개요 overflow 3개는 재현됐다. 오프라인 단독은 PASS이며 원인은 확정하지 않았다 (`~/FireScene-evidence/phase5/finalization/firescene-phase5-keyed-regression/`). 실제 Tauri 격리 프로필에서 `/dev/full`을 통한 쓰기 실패 시 입력 거부·권한 회수·중립 및 기록 오류에 따른 새 시작 거부 PASS (`firescene-phase5-tauri-write-failure/`). 실제 Tauri 포화·종료 경합은 아직 시험하지 않았다.

유휴와 가상 세션 활성 각각 2초 표본 3회의 중앙값은 main CPU 6.0%/3.5%, main RSS 240.54/240.23MiB, WebKit CPU 12.0%/10.0%, WebKit RSS 440.69/458.14MiB였다 (`firescene-phase5-virtual-perf/`). 짧은 단일 환경 측정으로 RSS에는 공유 페이지 중복이 가능하다. GPU, GUI 입력 지연 및 정상·기록 정지 시 정확한 권한 회수 시간의 3회 측정은 미완료다.

### 20개 기준의 현재 근거

당시 20개 판정은 모두 PARTIAL이었다. 아래 최신 개별 평가가 이를 대체한다. 1은 안전 아키텍처 문서와 동작 경계, 2는 토큰과 watchdog, 3은 연결·재생 경합 시험을 근거로 하되 다른 창과 전체 경합이 남는다. 4~8은 모델·가상 GUI 입력과 제한시간 시험을 근거로 하되 포커스·최소화의 직접 IPC 검증이 남는다. 9~10은 실제 Tauri 가상 세션·정지된 기록 worker·쓰기 실패 시험을 근거로 하되 포화·종료 경합이 남는다. 11은 비행 전 점검 GUI, 12는 안전 모니터 단위 시험, 13은 활성 입력 상태의 5개 모의 위험 시험을 근거로 하되 위험 복구 시험이 남는다. 14~17은 안전 UI 및 GUI 회귀를 근거로 하되 위 3개 GUI 실패가 남는다. 18은 두 release의 정상 IPC 대조군과 가상 IPC 8개 미등록 시험을 근거로 하되 개발 구성과 나머지 격리 조합이 남는다. 19는 기존 읽기 전용 설계 및 잠금 검증을 근거로 하되 외부 기체 환경은 미검증이다. 20은 네트워크 격리 거부를 기록했으며 SITL 자체는 NOT TESTED다. 상세 결과와 증거 경로는 위 절과 handoff에 있다.

최종 자동화 재실행에서 프런트엔드 18 PASS, TypeScript 포함 생산 Vite 빌드 PASS, Rust 기본 58 및 simulation 72 PASS, 각 1 ignored 및 Tauri 권한 시험 각 1 PASS였다. 세 GUI 실패와 위 안전 경합 검증 공백은 해결되지 않았다.

128개 이벤트 큐 포화와 기록 작업자 정지 상태에서 watchdog tick과 사용자 종료 경로를 20회 경합시킨 Rust 단위 시험은 중립화와 이전 토큰의 새 권한 해제 거부를 확인했다. 실제 Tauri 경합 검증은 별도로 남는다.

주 창의 직접 시작과 활성 입력을 양성 대조로 확인한 후 X11 보조 창으로 포커스를 이동했다. 실제 Tauri의 주 창 직접 시작·입력 거부와 OFFLINE·중립화를 확인했다 (`~/FireScene-evidence/phase5/finalization/firescene-phase5-focus-ipc-retry/results.json`). 다른 Tauri 창의 ACL 및 최소화 상태 시험은 아니다.

다섯 모의 위험 각각에 대해 모델 단위로 안전 데이터 복구 후에도 잠금 유지·이전 입력 거부 및 명시적 새 모델 세션만 허용함을 확인했다. 이 시험은 내부 모의 데이터를 직접 정상화하므로 실제 Tauri IPC 기반 위험 복구 증거는 아니다.

### 2026-11-26 잔여 시험 후 개별 판정

| 번호 | 판정 | 근거와 남은 범위 |
|---|---|---|
| 1 | PASS | 안전 아키텍처 및 읽기 전용·가상·재생 경계를 문서화했다. |
| 2 | PARTIAL | 토큰·watchdog과 포화 큐 종료 경합 단위 시험 통과; Tauri 경합 남음. |
| 3 | PARTIAL | 연결·재생 상호 배제 단위 및 실제 IPC 시험 통과; 전체 경합 남음. |
| 4 | PARTIAL | 프런트엔드 입력 단위 및 가상 GUI 통과; 다른 창 IPC 남음. |
| 5 | PARTIAL | 반대 방향 입력 단위 및 오프라인 GUI 시험 통과; 가상 입력의 물리 키보드 확인 남음. |
| 6 | PARTIAL | 키 해제·중립 GUI 통과; 전체 GUI의 오프라인 연습 실패 미해결. |
| 7 | PARTIAL | 실제 주 창 포커스 상실 IPC 통과; 최소화 및 다른 Tauri 창은 미검증. |
| 8 | PASS | 500ms 모델 만료, UI 비응답 및 기록 작업자 정지 중 독립 회수 확인. 하드 실시간 보장은 아님. |
| 9 | PARTIAL | 개발 가상 IPC·GUI 및 활성 입력 위험 시험 통과; 전체 경합 검증 남음. |
| 10 | PARTIAL | 5개 위험 잠금과 복구 모델 시험 통과; Tauri 위험 복구 및 종료 경합 남음. |
| 11 | PARTIAL | 오프라인 읽기 전용 점검 GUI 통과; 실제 기체 입력은 미검증. |
| 12 | PARTIAL | 누락·오류·지연 센서 안전 판단 단위 시험 통과; 실제 데이터 미검증. |
| 13 | PARTIAL | 5개 활성 입력 위험 GUI 및 내부 복구 단위 시험 통과; Tauri 복구 남음. |
| 14 | PARTIAL | 안전 UI·모의 경고 GUI 확인; 전체 GUI 세 실패 남음. |
| 15 | PARTIAL | 분리 기록·오류 및 가상 JSONL 확인; 실제 Tauri 포화·종료 경합 남음. |
| 16 | PARTIAL | 실제 Tauri 9메뉴·6크기·가상·재생 실행; 세 실패 남음. |
| 17 | PARTIAL | 자동화 및 주요 GUI 회귀 통과; 전체 실행의 세 실패 남음. |
| 18 | PARTIAL | 두 release의 유효 origin에서 정상 IPC와 가상 IPC 8개 미등록 확인; 다른 창 ACL 남음. |
| 19 | PARTIAL | 읽기 전용 구조·기존 명령 잠금 증거; 실제 FC는 미검증. |
| 20 | PASS | SITL 범위·격리 실패 및 NOT TESTED 한계를 기록했다. SITL 실행 PASS가 아니다. |

현재 P0 발견 없음. P1은 다른 Tauri 창·최소화·위험 복구 및 기록 포화/종료 경합의 안전 검증 공백이다. P2는 전체 GUI의 세 실패와 성능 반복 측정 미완료다. Phase 5 기준선 및 Phase 6 진입은 보류한다.

추가 단위 시험 포함 simulation Rust 전체 재실행은 74 PASS, 1 ignored 및 Tauri 권한 시험 1 PASS였다. 제품 코드 변경이 없어 이전 프런트엔드·기본 Rust·release 증거를 유지한다.

동일 단위 실행에서 입력 갱신부터 watchdog tick의 권한 회수 직후까지 각각 3회: 정상 worker 501.788/502.317/502.581ms(중앙 502.317), 정지 worker 502.045/502.241/502.246ms(중앙 502.241) (`~/FireScene-evidence/phase5/finalization/watchdog-unit-timing.txt`). 이는 운영체제 스케줄링을 포함한 단위 관측이며 실제 Tauri의 반복값이나 시간 상한이 아니다. 실제 Tauri의 활성 입력 IPC 왕복은 1/0/0ms(중앙 0ms, 브라우저 시계 분해능 한계)였고 물리 키부터 화면 표시까지를 측정한 값이 아니다 (`firescene-phase5-input-ipc-timing/results.json`).

반복 측정 시험 추가 후 simulation Rust 전체는 75 PASS, 1 ignored 및 Tauri 권한 시험 1 PASS였다.

실제 Tauri 개발 simulation에서 ACL 없는 `qa-untrusted` 보조 창을 시험 전용 환경 변수로 생성했다. 같은 실행의 주 창 연결 상태 및 가상 시작·종료 양성 대조가 성공했고 보조 창의 가상·재생 관련 IPC 7개는 기존 ACL에 의해 거부됐다 (`~/FireScene-evidence/phase5/finalization/firescene-phase5-tauri-other-window/results.json`). ACL·CSP를 변경하지 않았다. 최소화·복원은 미검증이다.

창 크기 단독 진단에서 1200×800 요청 시 실제 웹뷰 1200×800, workspace client/scroll 673/673px, 지도 720×471px로 통과했다 (`firescene-phase5-layout-diagnostic/`). 전체 GUI 선행 상태를 재현하지 못했으므로 기존 전체 회귀 실패는 여전히 열려 있다.

실제 Tauri 주 창 최소화 시 X11 비표시 상태, 시작·이전 입력 거부 및 복원 후 OFFLINE·중립 PASS (`~/FireScene-evidence/phase5/finalization/firescene-phase5-minimize-ipc-final/`). FIFO에 정지한 기록 작업자와 128개 큐를 실제 IPC로 포화해 기존 토큰 거부·권한 해제·신규 가상 시작 거부 PASS (`firescene-phase5-tauri-queue-race/`); 정지 중 사용자 종료 PASS (`firescene-phase5-tauri-stalled-stop/`). 위험·종료 동시 요청 및 늦은 이전 종료가 새 세션을 침해하지 않음 10회 PASS (`firescene-phase5-tauri-session-races-final/`). 실제 기록 쓰기 실패와 종료의 동시 경합은 미검증이다.

시험 전용 환경 변수로만 활성화되는 simulation debug 모의 데이터 복구 분기를 사용해 실제 Tauri의 다섯 위험별 활성 입력→잠금·권한 회수→정상 데이터→이전 입력 거부·자동 재개 없음→명시적 새 세션 입력을 확인했다 (`firescene-phase5-tauri-fault-recovery/`). 생산 빌드에는 가상 모듈이 없다.

전체 GUI 진단 재실행에서도 세 실패가 재현됐다 (`firescene-phase5-gui-diagnostic-full/`). 개요의 실제 창 1200×800, 지도 720×471px에 workspace client/scroll 673/4081px; 축소 시 실제 910×512, 지도 832×269px에 397/3805px였다. 단독 창 크기 진단은 정상이라 전체 선행 상태의 영향 가능성이 있으나 원인은 아직 확정하지 않았다.

전체 GUI 실패의 원인은 비행 제어 페이지의 `ManualPreview`와 `VirtualControl` 형제 컴포넌트가 같은 React key를 사용해 반복 전환에서 이전 미리보기 DOM이 잔존한 것이었다. 실패 진단에서 여러 오프라인 연습 버튼과 비정상 workspace scroll 높이를 확인했다 (`~/FireScene-evidence/phase5/finalization/firescene-phase5-gui-overflow-trace/`). 두 키를 구분해 수정한 후 키 제공 격리 환경의 기존 전체 순서·기준에서 3개 기존 실패를 포함해 전체 GUI PASS (`firescene-phase5-gui-distinct-keys/`), 실제 가상 GUI 재시험 PASS (`firescene-phase5-virtual-distinct-keys/`). 프런트엔드 18 PASS·TypeScript/Vite PASS, Rust 기본 58·simulation 75 PASS 및 각 권한 시험 1 PASS(각 1 ignored)였다. 이 수정 후 release IPC 재시험이 필요하다.

### 2026-11-26 최종 소프트웨어 기준선 평가

아래 평가는 개발 가상 모델과 현재 환경의 소프트웨어 범위다. 실제 Windows·물리 키보드·FC·격리 SITL 통과를 뜻하지 않는다. 이전 평가 및 실패 기록은 이력으로 보존하며 이 표가 최신이다.

| 기준 | 판정 | 근거와 한계 |
|---|---|---|
| 1 안전 아키텍처 정의 | PASS | `phase5-safety-architecture.md`의 읽기 전용·가상·재생 분리 및 권한 정책. |
| 2 제어 권한 상태 관리 | PASS | 세션별 토큰, watchdog 및 실제 Tauri 위험/종료·이전 종료 경합 10회 (`firescene-phase5-tauri-session-races-final/`). |
| 3 실제·시뮬레이션·재생 분리 | PASS | 연결·재생 경쟁 단위 시험과 실제 Tauri 가상/재생 거부 (`firescene-phase5-io-replay/`). 물리 기체는 외부 검증. |
| 4 입력 관리자 | PASS | 입력 reducer·시퀀스 단위 시험과 실제 가상 GUI (`firescene-phase5-virtual-distinct-keys/`). |
| 5 반대 방향 입력 처리 | PARTIAL | reducer 및 오프라인 GUI는 검증했으나 가상 세션에서 반대 키 동시 입력 자체의 실제 Tauri 종단 간 증거는 없다. |
| 6 키 해제 및 입력 초기화 | PASS | 가상 GUI 키 해제 중립 및 전체 GUI Space·ESC 시험 (`firescene-phase5-gui-distinct-keys/`). |
| 7 포커스 상실 처리 | PASS | 실제 X11 포커스 상실, 최소화·복원 및 직접 IPC 거부 (`firescene-phase5-focus-ipc-retry/`, `firescene-phase5-minimize-ipc-final/`). |
| 8 입력 시간 초과 | PASS | 500ms 단위 및 실제 웹뷰 정지·기록 작업자 정지 watchdog 시험. 하드 실시간 상한 아님. |
| 9 가상 제어 기능 | PASS | 개발 simulation 실제 Tauri 키보드·GUI·IPC 및 메모리 상태 (`firescene-phase5-virtual-distinct-keys/`). |
| 10 가상 제어 안전 잠금 | PASS | 활성 입력 중 5개 위험 및 실제 Tauri 복구 후 이전 입력 거부 (`firescene-phase5-tauri-fault-recovery/`). |
| 11 비행 전 점검 화면 | PASS | 읽기 전용 오프라인 관찰·수동 확인 초기화·명령 잠금 GUI (`~/FireScene-evidence/phase5/preflight/final-results.json`). 실제 FC 항목은 별도. |
| 12 센서 및 연결 안전 판단 | PASS | 누락·INVALID·STALE 단위 시험과 합성 안전 GUI. 실제 데이터는 별도. |
| 13 비상 상황 모의 시험 | PASS | 다섯 위험의 활성 입력·잠금 및 개발 전용 복구 IPC (`firescene-phase5-tauri-fault-recovery/`). 실제 비상 상황 아님. |
| 14 안전 관리 UI | PASS | 실제 Tauri 안전 경고·비행 전 점검 및 전체 GUI (`firescene-phase5-gui-distinct-keys/`). |
| 15 안전 이벤트 기록 | PARTIAL | 분리 JSONL·위험별 원인·열기/쓰기 실패·포화 시험 통과. 모든 이벤트의 장시간 순서·중복/누락 전수는 미검증이다. |
| 16 실제 Tauri GUI 시험 | PASS | 키 제공 격리 프로필의 전체 46개 GUI 검사 PASS (`firescene-phase5-gui-distinct-keys/results.json`). |
| 17 기존 기능 회귀 시험 | PASS | 46개 GUI 검사, 프런트엔드 18개, 기본 Rust 58개 및 simulation 75개 통과. 실제 장치 회귀는 별도. |
| 18 생산 및 개발 빌드 격리 | PASS | 주 창 양성 대조와 다른 Tauri 창 ACL, 일반/simulation release 양성 대조 및 가상 IPC 8개 미등록 (`firescene-phase5-tauri-other-window/`, `firescene-phase5-final-{normal,simulation}-ipc/`). |
| 19 실제 비행 명령 송신 차단 | PASS | 읽기 전용 전송 경로 및 직렬 쓰기 차단 단위 시험, GUI 명령 잠금. 실제 FC 연결은 별도. |
| 20 SITL 검증 범위 및 한계 기록 | PASS | 네트워크 격리 실패와 SITL NOT TESTED를 문서화했다. SITL 실행 PASS가 아니다. |

판정 합계: PASS 18, PARTIAL 2, FAIL 0, NOT TESTED 0. 이는 20개 소프트웨어 기준의 판정 수이며 외부 PX4 SITL·Windows·물리 FC 검증은 별도 NOT TESTED다. P0/P1 발견 없음. P2 검증 잔여는 가상 세션의 반대 키 동시 입력 종단 간 확인과 모든 이벤트의 장시간 순서·중복/누락 확인이다. 현재 환경에서 핵심 소프트웨어 안전 검증은 통과했으므로 Phase 5 소프트웨어 기준선을 확정한다. Phase 6 진입은 가능하되 자동 착수하지 않으며 외부 검증을 이월한다.

추가 실제 Tauri 증거: 큐 포화 후 종료 요청 없이 watchdog이 단독 회수 (`firescene-phase5-tauri-queue-watchdog/`), 격리 `/dev/full` 쓰기 실패와 사용자 종료 동시 요청 시 안전 유지 (`firescene-phase5-tauri-write-stop-race/`). GUI 수정 후 일반/simulation release 실제 IPC 대조군 및 8개 가상 IPC 미등록 재확인 (`firescene-phase5-final-normal-ipc/`, `firescene-phase5-final-simulation-ipc/`).
