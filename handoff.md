# 작업 인계

## 2026-11-26 Phase 6 0.1.1 로컬 마감 진행 (최신)

앱·Cargo·프런트엔드·Tauri 및 Windows 시험 아티팩트 이름을 0.1.1로 맞추고 진단 정보의 고정 버전을 패키지 버전으로 교체했다. 공개 Preview 1 설정의 0.1.0, 식별자 및 일반 사용자 설치 정책을 읽기 전용으로 대조했다. 태그 제안 `v0.1.1-hardware-preview.2`는 미생성. 실제 Windows 업데이트·데이터 보존은 NOT TESTED.

실제 개발 Tauri 내구 870세션·618.8초·4,640건 예상 순서/개수 PASS; 0.1.1 `/dev/full` 실패 후 권한 회수·신규 시작 거부 및 화면 버전 PASS. 새 합성 UDP 테스트는 연결 종료 후 이전 데이터 제거와 이전 포트·다른 기체 차단, 새 0 수용 PASS. 0.1.1 프런트엔드 19개, TypeScript/Vite, 기본 Rust 60 및 simulation 77 PASS, 각각 1 ignored 및 권한 시험 PASS. 현재 시점에는 0.1.1 DEB 빌드·생산 IPC·시작 단계 계측이 아직 남았다.

공개 `origin/main..HEAD` 이력의 현재 홈 경로 원문과 개인 이메일 형식 작성자·커미터 주소를 발견했다. 현재 handoff의 경로는 가렸으나 과거 커밋을 변경하지 않았다. 사용자의 지시대로 원격 push는 보안 보류이며 별도 정리 브랜치 선택을 사용자에게 요청한 상태다. 원격 작업·태그·Release는 수행하지 않았다.

## 2026-11-26 Phase 6 시작 성능 비교 (최신)

Phase 5 `684b035`를 임시 `git archive`에서 빌드하고 현재 `2372bb6` DEB 추출 실행 파일과 격리 Ubuntu Xvfb/WebDriver에서 번갈아 3회 측정했다. 세션 요청→주 메뉴 감지 중앙값 각각 1.373초와 3.158초. WebDriver·웹뷰 시간을 포함하고 baseline은 `--no-bundle`, 현재는 DEB 패치 실행 파일이라 동일 조건이 완전하지 않다. 1.785초 관측 차이의 원인은 미확정이며 P2 조사 대상이다. `docs/phase6-performance-report.md`, `~/FireScene-evidence/phase6/performance/` 참조. 실제 Windows 성능은 미측정.

기존 30개 기준의 개별 중간 평가는 `docs/phase6-validation-report.md`, 제한적 배포 보안·버전/승인 절차는 `docs/phase6-release-checklist.md`다. 장시간 감사 전수 및 텔레메트리 조합·보안 검사, Windows 설치 검증과 동일 버전 업데이트 정책은 미완료다. 사용자 승인 전 원격 작업 금지.

## 2026-11-26 Phase 6 패키지 및 평가 체크포인트 (최신)

소스 `2372bb6`에서 Tauri CLI 2.12.6의 앱/프런트 경로를 명시하고 Rust 홈 경로 재매핑과 release 심볼 제거로 DEB를 생성했다. `dist/phase6-local-validation/FireScene-GCS-Phase6-remapped-amd64.deb`, 5,134,216 bytes, SHA256 `fb23e3c1bccf9b8389e8886874eb1049f1314964fea458cf3328223b8fbf9b42`; 앱/DEB 내부 버전 0.1.0. 추출 실행의 읽기 `connection_status` 및 가상 IPC 8개 미등록 PASS. 추출 파일에서 지도 키·현재 홈 경로·QA 오류 주입 표식 원문 0건, stripped 확인; 제한적 검사이고 전체 보안 보증은 아니다. 증거 `~/FireScene-evidence/phase6/packaging/`. 이전 비재매핑 DEB는 홈 경로·심볼 포함으로 배포 금지. 사용자 환경에는 설치하지 않았다.

추가 자세 경계 시험 포함 기본 Rust 59 PASS, simulation 76 PASS, 각 1 ignored 및 권한 시험 PASS. `docs/phase6-validation-report.md`는 30개 기준 개별 중간 판정, `docs/phase6-release-checklist.md`와 Windows/KOA 시험 계획 참조. 공개 Preview 1 존재·버전 0.1.0을 읽기 전용 확인했고 현 후보도 0.1.0이므로 업데이트 정책 미확정이다. Windows 테스트 워크플로는 명시적 앱 경로와 구분된 파일명을 사용하나 실행하지 않았다. 최종 디자인의 Windows 확인, Windows 전체 NSIS 빌드·DPI·업데이트/데이터 보존, 장시간 이벤트 추가 검증·텔레메트리 전수·시작 전후 반복 성능·보안 검토는 미완료다. 아직 최종 배포 후보 아님. 원격 push/Actions/태그/공개는 승인 전 금지.

## 2026-11-26 Phase 6 로컬 후보 작업 중 (최신)

`docs/phase6-integration-report.md` 기능별 코드·이력 대조 및 `docs/phase6-pending-validation.md` 추가 시험 참조. 실제 개발 Tauri 가상 반대 키/부분 해제/토큰 거부 PASS; 90세션 63.9초 480 이벤트의 정상·잠금·watchdog 순서/개수 PASS. 긴 시간 전수는 여전히 PARTIAL. 자세 경계와 센서 0/TTL 단위 시험 추가 10/10 PASS. 직전 Rust 전체 기본 58·simulation 75 PASS, 각 1 ignored 및 권한 시험 PASS(샌드박스 루프백 거부 후 권한 상승 재실행).

Tauri CLI가 저장소 루트에서 다른 실험 프로젝트를 선택할 수 있어 Windows 워크플로에 `TAURI_APP_PATH`/`TAURI_FRONTEND_PATH`를 명시하고 시험 아티팩트 이름을 Preview 1과 구분했다. 명시 경로 Ubuntu DEB 빌드 및 추출 실행 IPC 양성/8개 거부 PASS. 첫 로컬 DEB는 0.1.0, 5,755,692 bytes, SHA256 `a3c8494308e32dde0d7f173de470f1d8cdebfeee30642490ed86f642e867ecea`; 그러나 실행 파일에 로컬 홈 경로와 심볼이 있어 배포 후보가 아니다. 경로 재매핑과 심볼 제거를 통한 재빌드 및 재검사가 다음 작업이다. 공개 Preview 1 Release 존재 확인; 동일 0.1.0 업데이트 문제와 공개 버전 승인이 남는다. Windows 실제 설치·FC·SITL 미검증, 원격 작업 금지.

## 2026-11-26 Phase 6 최종 배경색 포함 회귀 (최신)

소스 `60b4cb354bed4c3cb973cc082caf23446c8c35c9`의 simulation 개발 바이너리를 명시 빌드하여 최종 배경색 포함 기존 GUI 48/48 PASS. 지도 키 원문은 증거에 없음을 확인했다. 프런트엔드 19개 시험, TypeScript/Vite 및 기본·simulation Tauri 권한 시험 PASS. 일반 및 simulation 지정 release의 실제 앱 origin에서 읽기 `connection_status` 양성 대조와 가상 IPC 8개 미등록 PASS. 증거는 `~/FireScene-evidence/phase6/gui/firescene-phase6-installer-final-gui/`와 `firescene-phase6-{normal,simulation}-release-ipc/`. 설치기 5페이지 목업 및 NSIS 하네스 결과는 branding 폴더. 자세한 조건은 `docs/phase6-brand-validation.md`.

Windows NSIS 전체 컴파일·실행, 모든 페이지의 전용 다크 디자인 및 DPI, 업데이트/데이터 보존은 여전히 미검증이다. 시작 오류 실제 주입, 시작 성능 반복 측정, Phase 4/5 잔여·30개 기준·패키징·보안이 남는다. 최종 배포 후보 아님. 다음은 설치기 Windows 후보 검증 준비를 보완하고 Stage A/B 안전 잔여를 수행한다. 원격 push/Actions/태그/공개는 별도 승인 전 금지.

## 2026-11-26 Phase 6 설치기 페이지 개선 (최신)

`src-tauri/installer/firescene.nsi`에 기본 설치 로직을 보존하면서 MUI 헤더·완료 배경, 페이지별 한국어/영어 제목, 읽기 전용 버전 완료 안내를 추가했다. Welcome의 언어 상수가 정의 전 참조되어 발생한 NSIS 경고를 수정했다. 임시 폴더에만 압축 해제한 Ubuntu NSIS 3.09에서 `scripts/check-phase6-installer.py --makensis ...`의 실제 Welcome 및 MUI 페이지 하네스가 `-WX` PASS; 전체 Tauri 렌더링·Windows 실행은 NOT TESTED다. 다섯 구현 기반 **목업**은 `docs/phase6-concepts/installer-pages/`, 제한은 `docs/phase6-installer-design.md`. 기본 MUI 설정·진행·제거 본문은 네이티브 밝은 컨트롤이며 전체 전용 다크 디자인은 아니다. 다음: 최종 코드 기준 프런트엔드·권한·전체 GUI·두 production release IPC, 증거 및 성능 검증. 기존 미추적 Stage 0 보존, 원격 및 배포 금지.

## 2026-11-26 Phase 6 GUI 및 시작 녹화 체크포인트 (최신)

`docs/phase6-brand-validation.md` 참조. 최초 전체 GUI는 simulation 없는 바이너리 사용으로 39/48; simulation 명시 빌드 후 48/48 PASS. 두 원본은 `~/FireScene-evidence/phase6/gui/`에 보존했고 키 원문 미포함을 검사했다. 이후 초기 흰 창을 실제 Xvfb 녹화에서 발견해 Tauri 창·웹뷰 배경을 지정했다. 재녹화에서 흰 프레임은 보이지 않았고 오버레이 해제·오프라인 입력 재시험 PASS. 전후 영상은 `~/FireScene-evidence/phase6/branding/`. 전체 48개는 배경색 변경 전이다. Windows NSIS 사용자 지정 템플릿 컴파일·실행과 나머지 설치 페이지 디자인, 실패 주입, 생산 release IPC, Phase 4/5 잔여와 30개 기준은 미완료. 원격 작업 및 배포 후보 생성 금지.

## 2026-11-26 Phase 6 시작·설치 디자인 체크포인트 (최신)

세 설치/시작 목업 SVG 및 Chromium PNG는 `docs/phase6-concepts/`, 사본은 `~/FireScene-evidence/phase6/branding/`에 있다. A의 여백·청색에 B의 도시/주황, C의 원형 좌표를 결합했다. 실제 앱은 초기 HTML·로컬 CSS/JS로 로딩을 표시하고 React 마운트 시 지연 없이 제거하며, 8초 지연 시 재시도 안내와 모션 감소를 지원한다. 별도 창·IPC 없음. 프런트엔드 19개 시험 파일, TypeScript/Vite 빌드 및 Tauri 권한 시험 PASS. 격리 개발 Tauri에서 스플래시 제거와 오프라인 입력 PASS, 최초 실행 캡처 `~/FireScene-evidence/phase6/branding/first-run.png`. 스플래시 표시 순간·전환 영상, 재시작 보존, 전체 GUI 및 release IPC는 미검증이다.

Tauri bundler 2.10.6 템플릿에 사용자 지정 Welcome 페이지를 추가하고 `tauri.conf.json`에 연결했다. 나머지 원본 설치 로직을 유지했으나 **Windows NSIS 컴파일과 실행은 미검증**이며 설정·진행·완료·제거 전용 디자인도 미완료다. 최종 설치 파일을 생성하지 말고 Windows 시험 전 템플릿의 컴파일·업데이트·다국어·DPI를 확인할 것. `docs/phase6-installer-design.md`, `docs/phase6-launch-experience.md` 참조. Stage A/B 잔여와 30개 기준·성능·전체 안전 회귀는 계속 미완료. 미추적 Stage 0 보존, 원격 작업 금지.

## 2026-11-26 Phase 6 브랜드 구현 중간 체크포인트 (최신)

`frontend/public/firescene-airframe.svg` 자체 개념 도면을 제작하고 `FirstRun.tsx`와 `index.css`에 적용했다. 기존 완료 키·언어·테마·포커스 정책은 변경하지 않았다. `docs/phase6-brand-design.md`에 세 방향 비교와 출처를 기록했으나 세 실제 시안, 스플래시·설치기 적용 및 실제 렌더링은 미완료다. Rust telemetry 단위 9 PASS, 프런트엔드 18개 시험 파일 PASS, TypeScript/Vite 빌드 PASS; 메인 청크 669.83 kB 경고 유지. 기존 시험의 재실행이며 Phase 4 전수나 Phase 5 두 잔여를 완료한 것이 아니다. 다음 작업: 통합 대조와 잔여 Tauri 시험, 시안·설치 템플릿 분석, 실제 초기화 기반 스플래시 및 GUI 검증. 외부 환경·원격 작업 제한은 아래 최신 Phase 6 체크포인트와 동일하다.

## 2026-11-26 Phase 6 초기 체크포인트 (최신)

기준 `684b035`에서 로컬 `phase6-integrated-release` 브랜치를 생성했다. 초기 조사와 미완료 범위는 `docs/phase6-integration-report.md` 참조. 프런트엔드 시험 18개 파일 및 TypeScript 검사 PASS. 제품 변경, Phase 4·5 잔여 시험, 디자인 및 최종 회귀는 아직 수행하지 않았다. 기존 미추적 Stage 0 보고서는 보존했다. 다음 단계는 세부 통합 대조와 Phase 4 텔레메트리·Phase 5 두 잔여 검증이다. 요청된 증거 경로는 작업 공간 밖이므로 쓰기 권한 확보 전까지 기존 증거를 수정하지 않는다. 원격 push, Actions, 태그 및 Release는 사용자 승인 전 수행하지 않는다.

## 현재 상태

2026-11-26 Phase 5 통합 진행 중. 브랜치 `phase5-safety-manual-control`, 최신 체크포인트 `08385c6`. 기존 미추적 `docs/stage0-cleanup-report.md` 유지. `docs/phase5-validation-report.md`, `docs/phase5-safety-architecture.md`, `docs/phase5-simulation-control.md`를 참조한다.

가상 획득과 직렬·전달/합성 UDP 시작을 개발 simulation 빌드에서 공통 mutex로 직렬화하고 연결이 있는 동안 가상 획득을 거부한다. 세션 토큰으로 이전 세션 해제가 새 잠금을 풀지 못하게 했다. 병렬 경쟁 10회 및 각 경로·오래된 해제 시험 2개 PASS. 모델의 입력 순번으로 지연/중복 입력을 거부하며 상태 폴링 없이 500ms 만료 후 입력이 와도 잠금 처리한다. 모델 4개 단위 시험 PASS. 변경 중간 Rust 전체 회귀 기본 58·simulation 64 및 권한 시험 각 1 PASS; 이후 토큰 및 만료 보호 변경은 해당 단위 시험만 재실행했다. 아직 IPC에 가상 권한 획득/해제를 연결하지 않아 컴파일 시 dead_code 경고가 있고, 이 체크포인트를 종단 간 제어 기능으로 간주하지 않는다.

다음 작업: backend가 보유하는 재생 상태와 가상 세션의 원자적 생명주기를 설계하고, 오래된 IPC·창 포커스·메뉴 전환·연결 경합을 막는 기능을 구현한 후에만 development-only IPC를 등록한다. 생산 빌드 제외, 실제 통신·제어 경로 불가를 검증한다. 이어 키보드·GUI, 안전 판단, JSONL 이벤트 호환과 전체 GUI·성능 회귀가 남는다. 실제 SITL의 네트워크 격리는 양쪽 unshare 실행에서 EPERM이었으며 실행하지 않았다. Phase 5 완료/Phase 6 진입 미판정. 외부 증거는 원래 호스트 `~/FireScene-evidence/phase5/`에 있으며 다른 장치로 자동 이전되지 않는다. 원격 push 및 Release 금지.

```context-handoff-state
{
  "schema_version": 1,
  "updated_at": "2026-11-26T06:52:50.073554+00:00",
  "project_root_hint": "<PROJECT_ROOT>",
  "host": "<HOST>",
  "git": {
    "head": "08385c677d3333a7e017f2c20dfe79593019b2b1",
    "branch": "phase5-safety-manual-control",
    "status": [
      {
        "status": "??",
        "path": "docs/stage0-cleanup-report.md"
      }
    ]
  },
  "evidence": [
    {
      "path": "docs/phase5-validation-report.md",
      "sha256": "289999b85d9c4ab5b724211b219b90feafe5e021527d1cf023a627cc54a27f51"
    }
  ],
  "processes": {
    "visibility": "VISIBLE_SCOPE",
    "scope": "Same-user Linux processes whose CWD is inside this project; excludes inspector ancestors. No arguments/environment collected. Remote/container jobs and jobs with external CWD need separate inspection.",
    "tasks": [],
    "unreadable_same_user_processes": 0
  }
}
```

## 2026-11-26 통합 복구 재개

HEAD `08385c6`; 중단된 IPC/GUI/이벤트 변경은 작업 트리에 존재했다. 기존 인계의 미구현 설명은 현재 코드보다 오래되었다. `docs/stage0-cleanup-report.md`는 미추적 상태로 보존한다. 기존 GUI 도구는 `/tmp/firescene-qa-tools`에 있다. 시험은 격리 프로필과 Xvfb `:91`을 사용했다.

현재 코드에서 simulation `cargo check`, 가상 모델/이벤트 관련 Rust 5개, TypeScript 검사, 프런트엔드 18개 시험 및 생산 Vite 빌드 PASS. 최초 샌드박스 Rust 전체 시험은 루프백 bind EPERM으로 실패했으며 권한 상승 재실행 결과로 판정해야 한다. 실제 Tauri `--virtual-only`는 다섯 모의 위험 버튼에서 잠금과 세션 종료, JSONL의 경고/잠금/종료 각각 5개를 확인하여 PASS (`/tmp/firescene-phase5-recovery-virtual/`). 이는 활성 입력·지연 입력·복구 후 자동 재개 방지 전체를 증명하지 않는다. `--replay-only`는 합성 기록·재생과 재생 중 가상 시작 거부 PASS (`/tmp/firescene-phase5-recovery-replay/`).

안전 미해결: 입력 만료는 `virtual_input` 또는 `virtual_status` 실행 시에만 평가되고 권한 해제는 `virtual_stop`에 의존한다. 프런트엔드가 응답하지 않으면 독립적인 백엔드 감시/권한 회수가 없으므로 요구된 안전 생명주기는 미검증이며 P1로 취급한다. 모의 경고 JSONL은 위험 종류를 포함하지 않아 개별 종류를 기록만으로 구분할 수 없다. 파일 쓰기 실패, 허용되지 않은 창, 최소화/포커스, 각 위험 중 활성·지연 입력, 생산 IPC 거부, 전체 GUI 회귀 및 CPU/메모리 시험이 남았다. SITL은 격리 불가로 NOT TESTED. Phase 5 기준선 확정 및 Phase 6 진입 불가.

권한 상승 후 Rust 전체 회귀: 기본 58 PASS, simulation 67 PASS, 각각 1 ignored 및 Tauri 권한 시험 1 PASS. GUI 증거는 `~/FireScene-evidence/phase5/integration/firescene-phase5-recovery-{virtual,replay}/`에 복사했다.

## 2026-11-26 Watchdog 작업 중간 체크포인트

기준 `8afb43e`에서 개발 simulation 전용 백엔드 스레드가 50ms 주기로 500ms 단조 입력 임대와 포커스 상실 이벤트를 검사하고 동일 runtime mutex에서 입력 중립화, 토큰 무효화 및 연결 잠금을 해제하도록 변경했다. GUI 스레드에 동기 질의하지 않는다. 기존 가상 시험 6개 PASS. 실제 Tauri 웹뷰 이벤트 루프를 1.8초 정지시킨 시험에서 상태 조회 없이 0.505초에 권한 해제 이벤트가 기록되고 이전 토큰 입력 거부 및 재생 권한 재획득 PASS (`/tmp/firescene-phase5-watchdog-retry3/`). 최초 두 실행은 시험의 IPC 가로채기가 실제 프런트 호출을 막지 못해 실패했으며 증거를 유지한다. 위험 잠금을 시간 초과로 잘못 기록하는 경합을 수정했다.

P1 미해결: 기존 IPC와 watchdog이 이벤트 기록 및 일부 창 상태 조회를 runtime mutex 안에서 수행한다. I/O나 GUI 질의가 장시간 막히면 감시가 mutex를 획득하지 못할 수 있으므로 독립 동작이 모든 요청 정지 상황에서 보장된다고 판정하지 않는다. 기록 I/O를 안전한 비차단 경계로 분리하고 실패·경합을 검증해야 한다. 구조화 위험 원인, 활성 입력 5위험, 포커스/최소화/창 종료, 허용되지 않은 창, release IPC 거부, 전체 GUI·성능 및 최종 회귀가 남았다. Phase 5 완료/Phase 6 진입 불가. 기존 20개 판정 유지.

후속 변경: 입력 IPC의 창 상태 조회를 runtime mutex 밖으로 옮겼다. 이 변경 후 가상 Rust 6개 및 실제 Tauri 비응답 시험 PASS, 권한 회수 관측 0.545초 (`/tmp/firescene-phase5-watchdog-final/`). 이벤트 기록이 mutex 안에 남아 있는 P1은 여전히 열려 있다.

## 2026-11-26 이벤트 I/O 분리 진행

`e49b600` 이후 가상 이벤트 기록을 128개 유한 큐와 별도 작업자로 분리했다. runtime mutex 안에서는 파일 경로 조회·열기·쓰기 대신 비차단 `try_send`만 실행한다. 큐 포화·작업자 쓰기 오류·panic은 실패 상태로 고정되고 다음 입력을 잠그며 watchdog이 권한을 회수한다. 비동기 초기 파일 열기 검사 또는 이전 이벤트가 미완료인 동안 새 세션 시작을 거부한다. enqueue 성공은 디스크 저장 확인이 아니다. 관련 Rust 시험 8개 PASS. 정지된 작업자·큐 포화 상태의 직접 권한 회수와 오류/panic은 단위 검증했으나 실제 watchdog 스레드와 장시간 I/O 정지 조합, 실제 GUI 및 나머지 최종 시험은 아직 미완료다. P1 해결을 최종 판정하지 않는다.

기록 작업자가 500ms 이상 멈춘 동안 실제 감시와 같은 `watchdog_tick`을 50ms 주기로 독립 실행하는 시험에서 중립화·권한 회수·재생 권한 재획득을 파일 I/O 재개 전 확인했다. 큐 포화, 쓰기 오류·panic, 디렉터리 오류를 포함한 가상 관련 Rust 10개 PASS. 실제 Tauri GUI 비응답 시험은 I/O 분리 후 0.527초 관측 PASS (`~/FireScene-evidence/phase5/finalization/firescene-phase5-io-isolation-gui/`). 운영체제 스케줄링 상한을 보장하지 않는다. 이후 모의 경고에 `reason` 필드를 추가했으며 해당 변경의 GUI 재시험은 아직 하지 않았다. 작업자 완전 정지 중 실제 Tauri 결합 시험, 쓰기 실패 경합, 활성 입력 위험별 시험 및 release IPC 등이 남는다.

활성 전진 속도를 확인한 후 다섯 모의 위험을 각각 주입하는 실제 Tauri GUI 시험 PASS. 각 세션의 백엔드 OFFLINE·중립 속도 및 이전 토큰 입력 거부와 5가지 구조화 `reason`을 확인 (`~/FireScene-evidence/phase5/finalization/firescene-phase5-active-hazards-final/`). 위험 복구 자체는 주입하지 않아 복구 후 자동 재개는 별도 미검증이다. 최신 Rust 기본 58·simulation 72 PASS, 각각 1 ignored 및 권한 시험 1 PASS; 프런트엔드 시험 18개와 TypeScript 포함 생산 Vite 빌드 PASS. 최신 `--replay-only` PASS (`firescene-phase5-io-replay/`). 전체 GUI 회귀 실행 중. 다른 창 IPC, 생산 release 실제 IPC 및 I/O 정지 실제 Tauri 결합 시험 등은 미완료.

최신 전체 GUI 회귀 (`~/FireScene-evidence/phase5/finalization/firescene-phase5-io-regression/`): 첫 실행, 9메뉴, Dark/Day, 여섯 크기, 가상 외 일반 기록·재생 등 통과. 5개 실패: 개요 1200x800 맞춤, 오프라인 연습, 키 미제공 지도 검색, 임무 지도 캔버스, 축소 개요 overflow. 오프라인 연습은 독립 재시험 PASS (`firescene-phase5-offline-retest/`); 전체 시험 순서/포커스 영향 가능성일 뿐 원인 확정 아님. 나머지 미해결. 일반/simulation release 빌드 성공. 실제 IPC 시도는 WebDriver 대상이 정상 앱 origin이 아니어서 정상 connection_status도 `Origin header is not a valid URL`로 실패하므로 생산 IPC 차단은 NOT TESTED (`firescene-phase5-release-ipc-*/`). P1의 파일 I/O와 mutex 직접 결합은 제거했고 주입 단위 시험은 통과했으나 실제 Tauri 기록 중단 결합 시험과 release 격리는 미검증이다. Phase 5 완료/Phase 6 진입 불가.

## 2026-11-26 실제 Tauri 기록 정지 결합 검증

격리 시험 프로필의 이벤트 파일을 FIFO로 교체하고 기록 worker의 `/proc/.../wchan`이 `wait_for_partner`임을 확인했다. 전진 입력 활성 후 웹뷰 이벤트 루프를 1.5초 멈추고 FIFO 독자가 없는 상태에서 이전 토큰 입력 거부 및 재생 권한 획득을 먼저 직접 IPC로 확인했다. 이후 상태는 OFFLINE·속도 중립이었다. PASS (`~/FireScene-evidence/phase5/finalization/firescene-phase5-tauri-journal-stall-verified/results.json`). 1.535초는 관측 시점이지 권한 회수 정확한 소요 시간 또는 상한이 아니다. 실제 Tauri 쓰기 오류·포화·종료 경합, 다른 창·위험 복구, 정상 origin release IPC, GUI 결함 및 성능 측정은 남았다.

일반 및 simulation release를 `tauri/custom-protocol`로 빌드해 유효한 패키지 앱 origin에서 실제 IPC를 재검증했다. 각각 `connection_status` 성공과 가상 IPC 8개 모두 `Command ... not found` 확인 PASS (`~/FireScene-evidence/phase5/finalization/firescene-phase5-release-{normal,simulation}-8ipc/results.json`). 이전 일반 cargo release는 custom protocol 없이 개발 URL을 사용해 origin 대조군이 실패한 시험이었다. CSP/ACL을 변경하지 않았다. release의 다른 기능 및 실기체 통신은 이 시험의 범위가 아니다.

실제 Tauri 격리 프로필에서 가상 입력 중 로그 파일 경로를 디렉터리로 바꿔 이벤트 파일 열기를 실패시켰다. 기존 토큰 입력 거부, 권한 해제 후 재생 권한 획득, OFFLINE·중립 속도 및 기록 오류를 이유로 신규 가상 시작 거부 PASS (`~/FireScene-evidence/phase5/finalization/firescene-phase5-tauri-journal-failure/results.json`). 실제 디스크 write 실패, 포화 및 종료 경합은 여전히 별도 미검증이다.

키를 격리 프로필에 제공한 동일 전체 GUI 재시험 (`~/FireScene-evidence/phase5/finalization/firescene-phase5-keyed-regression/`): 실제 지도 검색 및 임무 작성·복제 PASS, 이전 두 실패는 키 없는 환경과 관련 있으나 캔버스 실패의 정확한 원인은 별도 확정하지 않았다. 개요 맞춤, 전체 실행 중 오프라인 연습, 축소 개요 overflow 3개는 반복 FAIL. 오프라인 단독은 기존 PASS. 키는 결과 파일에 포함되지 않음을 검사했다. 실제 Tauri `/dev/full` 쓰기 실패에서도 이전 입력 거부·권한 회수·중립 및 기록 오류에 따른 신규 시작 거부 PASS (`firescene-phase5-tauri-write-failure/`).

동일 GUI 조건 2초×3회 중앙값: 유휴 main CPU 6.0%, RSS 240.54MiB, WebKit CPU 12.0%, RSS 440.69MiB; 가상 세션 활성 중 main 3.5%, 240.23MiB, WebKit 10.0%, 458.14MiB (`firescene-phase5-virtual-perf/`). 합산 RSS는 공유 페이지 중복 가능, 짧은 표본이며 성능 보장이 아니다. GPU·실제 반응 지연·정상 및 기록 정지 중 정확한 권한 회수 3회 측정은 미완료. 다른 창 IPC, 포커스/최소화, 위험 복구, 기록 포화 및 종료 경합의 실제 Tauri 시험과 GUI 실패 해결이 남아 Phase 5 기준선/Phase 6 진입 불가.

최종 자동화 재실행: 프런트엔드 18 PASS, TypeScript 포함 Vite 생산 빌드 PASS; Rust 기본 58 PASS 및 simulation 72 PASS, 각 1 ignored와 Tauri 권한 시험 1 PASS. GUI의 위 3개 실패와 안전 IPC/경합 미검증은 여전히 남는다.

## 2026-11-26 잔여 안전 시험 진행

기록 작업자 정지 및 128개 큐 실제 포화 상태에서 동일 runtime/connection 관리자에 watchdog tick과 사용자 종료 경로를 20회 동시 실행하는 Rust 단위 시험 PASS. 매회 중립화·기존 토큰 무효·새 가상 권한에 대한 오래된 해제 거부를 확인했다. 이는 실제 Tauri의 경합 시험이나 기록 오류 상태에서 신규 시작 허용을 뜻하지 않는다. 실제 다른 창·포커스·최소화, 위험 복구, GUI 실패 3개 및 반복 성능 측정이 남는다. Phase 5 완료 및 Phase 6 진입 불가.

실제 Tauri 주 창에서 직접 `virtual_start`/활성 입력 성공을 양성 대조로 확인한 후 별도 X11 창으로 포커스를 옮겨 주 창의 직접 시작·입력 거부, 토큰 회수와 중립 속도를 확인했다 (`~/FireScene-evidence/phase5/finalization/firescene-phase5-focus-ipc-retry/results.json`). 최초 시험은 xmessage의 X11 창에 PID 속성이 없어 보조 창 식별에 실패했고 이름으로 수정해 재시험했다. 이 X11 보조 창은 다른 Tauri 창이 아니며 다른 Tauri 창의 ACL 및 최소화 검증은 남는다.

다섯 모의 위험을 각각 활성 모델에 주입하고 잠금 후 시험 코드에서 모의 안전 데이터를 정상화하는 Rust 단위 시험 PASS. 정상 데이터가 되어도 기존 모델 권한은 Locked이고 이전 순번/세션 입력을 거부하며 명시적 새 모델 세션만 수용했다. 시험은 내부 모델·모의 관리자만 대상으로 하고 기존 IPC나 실제 Tauri에서 위험 복구를 주입한 증거가 아니다. 실제 Tauri 위험 복구 및 위험·종료 경합은 남는다.

20개 기준을 개별 평가로 갱신했다. 문서화 자체인 안전 아키텍처(1)와 SITL 범위·한계 기록(20), 모델·실제 Tauri watchdog으로 확인한 입력 시간 초과(8)는 PASS; 나머지는 각 행의 미검증 범위로 PARTIAL이다. SITL 실행은 여전히 NOT TESTED다. 전체 기준선과 Phase 6 진입은 보류한다.

추가 단위 시험 포함 simulation Rust 전체 재실행은 74 PASS, 1 ignored 및 권한 시험 1 PASS. 제품 코드 변경은 없었고 기존 프런트엔드·기본 Rust·release 증거는 유지한다.

반복 시간 측정: 동일 Rust 단위 실행의 실제 watchdog tick 직후 토큰 회수 관측은 정상 기록 501.788/502.317/502.581ms(중앙 502.317), 기록 worker 정지 502.045/502.241/502.246ms(중앙 502.241), 각 3회 (`~/FireScene-evidence/phase5/finalization/watchdog-unit-timing.txt`). 실제 Tauri나 하드 실시간 상한이 아니다. 실제 Tauri의 메모리 입력 IPC 왕복은 1/0/0ms(중앙 0ms, 브라우저 시계 분해능 한계) (`firescene-phase5-input-ipc-timing/results.json`); 물리 키부터 화면 표시까지의 반응 지연은 아니다. 제품 코드 변경은 없다.

반복 측정 시험 추가 후 simulation Rust 전체는 75 PASS, 1 ignored 및 권한 시험 1 PASS였다.

## 2026-11-26 잔여 Tauri 검증 재개

`FIRESCENE_QA_UNTRUSTED_WINDOW=1`일 때만 simulation debug 빌드에서 권한이 없는 보조 Tauri 창을 생성하는 격리 시험 fixture를 추가했다. ACL·CSP·origin 설정은 변경하지 않았다. 같은 실행에서 주 창의 `connection_status`, `virtual_start`/`virtual_stop` 양성 대조가 성공했고, `qa-untrusted` 창의 시작·입력·위험·잠금·종료·재생 획득/해제 IPC 7개가 실제 Tauri ACL에서 거부됐다 (`~/FireScene-evidence/phase5/finalization/firescene-phase5-tauri-other-window/results.json`). 주 창 최소화 및 복원은 미검증이다.

개요 1200×800 단독 진단에서 2560×1440→1100×760→최대화→1200×800 순서 후 WebDriver와 웹뷰가 실제 1200×800, workspace client/scroll 673/673px, 지도 720×471px로 일치해 통과했다 (`firescene-phase5-layout-diagnostic/`). 전체 시험의 앞선 상태를 재현하지 않았으므로 기존 세 GUI 실패를 해결했다고 판단하지 않는다. 위험 복구·큐 포화 및 실제 Tauri 종료 경합은 남는다.

주 창을 실제 X11 최소화해 비표시 상태를 확인한 뒤 직접 시작·이전 입력 거부, 복원 후 OFFLINE·중립을 확인했다 (`~/FireScene-evidence/phase5/finalization/firescene-phase5-minimize-ipc-final/results.json`). 실제 Tauri FIFO 정지 중 129회 리셋으로 128개 큐 포화를 확인하고 종료 요청 후 구 토큰 거부·재생 권한 획득·신규 가상 시작 거부 PASS (`firescene-phase5-tauri-queue-race/`). FIFO 정지 중 사용자 종료는 별도 PASS (`firescene-phase5-tauri-stalled-stop/`). 배터리 위험/종료 동시 IPC 및 구 종료 요청과 새 세션 보호 10회 PASS (`firescene-phase5-tauri-session-races-final/`). 실제 기록 쓰기 실패와 종료의 동시 경합은 아직 별도 미검증이다.

`FIRESCENE_QA_RECOVERY=1`로만 활성화되는 simulation debug의 기존 가상 위험 IPC 시험 분기를 추가했다. 권한이 OFFLINE인 경우에만 모의 데이터를 정상화한다. 실제 Tauri IPC에서 5개 위험 각각의 활성 입력→위험 잠금→권한 회수→정상화→이전 순번·지연 입력 거부→자동 재개 없음→명시적 새 세션 입력 PASS (`firescene-phase5-tauri-fault-recovery/`). 실제 기체와 무관하며 생산 빌드에는 해당 모듈이 없다.

전체 GUI 재실행 (`firescene-phase5-gui-diagnostic-full/`)에서 기존 3개 실패 재현. 개요 실패 당시 실제 웹뷰 1200×800, 지도 720×471px, workspace client 673px/scroll 4081px; 단독 시험은 673/673px였다. 축소 실패 시 웹뷰 910×512, workspace client 397px/scroll 3805px, 지도 832×269px였다. 선행 UI 상태에 따른 원인은 미확정이며 기준을 완화하지 않았다.

전체 GUI 실패 원인 추적: `firescene-phase5-gui-overflow-trace/`에서 개요 화면 workspace에 이전 오프라인 조종 미리보기 버튼이 다수 남아 스크롤 높이가 증가했고, 오프라인 시험이 잘못된 잔존 버튼을 선택했다. `App.tsx`의 인접 ManualPreview/VirtualControl이 동일 React key를 공유하던 문제를 컴포넌트별 고유 키로 수정했다. 키 제공 격리 프로필에서 기존 전체 순서·기준 그대로 세 실패 포함 전체 GUI PASS (`firescene-phase5-gui-distinct-keys/`); 키가 증거 파일에 없음을 검사했다. 수정 후 실제 가상 제어 GUI PASS (`firescene-phase5-virtual-distinct-keys/`), 프런트엔드 18개 및 TypeScript·Vite PASS, Rust 기본 58·simulation 75 PASS, 각 1 ignored 및 권한 시험 1 PASS. GUI/복구 fixture 변경 후 두 release 재검증 및 남은 실제 기록 실패/종료 동시 경합 확인이 필요하다.

최종 평가: 현재 환경 소프트웨어 핵심 안전 검증은 통과했고 P0/P1 발견 없음. 20개 개별 기준 PASS 18, PARTIAL 2(가상 세션 반대 키 동시 입력 실제 Tauri 종단 간, 이벤트 장시간 순서·중복/누락 전수). GUI 수정 후 전체 46개 GUI 검사 PASS, 프런트엔드 18·Rust 기본 58·simulation 75 및 각 권한 시험 PASS, 두 release 정상 IPC 양성 대조 및 가상 IPC 8개 미등록 재확인. Phase 5 소프트웨어 기준선 확정, Phase 6 진입 가능하나 자동 착수 금지. PX4 SITL·Windows·물리 키보드/COM·KOA FC·실제 텔레메트리 및 하드웨어 복구는 별도 NOT TESTED. 정지 큐에서 종료 요청 없이 watchdog 단독 회수 및 `/dev/full` 쓰기 실패/사용자 종료 동시 요청도 실제 Tauri PASS (`firescene-phase5-tauri-queue-watchdog/`, `firescene-phase5-tauri-write-stop-race/`).
