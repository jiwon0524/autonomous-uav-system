# Stage 6 — 안전 감시 및 경고

## 범위와 검증 상태

Stage 5 `7677eec`와 `design-v1.0`을 기준으로 읽기 전용 관제 보조 기능을 추가했다. 사용자는 일반 Ubuntu에서 Stage 5 가상 텔레메트리 수신과 화면 연동에 성공했다고 보고했다. 이 기록은 사용자 확인이며 Stage 6 경고 화면, 실제 PX4 SITL 또는 실제 기체 검증을 뜻하지 않는다. 실제 FC·무선 장비는 연결하지 않았고 하드웨어 항목은 [체크리스트](hardware-validation-checklist.md)에서 PENDING이다.

FireScene 경고는 개발용 관찰 정책이며 [PX4의 독립 failsafe](https://docs.px4.io/v1.16/en/config/safety)를 대체하지 않는다. PX4 파라미터를 읽거나 변경하지 않고, PX4의 임계값·동작을 이 UI 값으로 추정하지 않는다. MAVLink의 [HEARTBEAT, SYS_STATUS 및 STATUSTEXT](https://mavlink.io/en/messages/common.html)와 기존 최신성 정책을 사용한다. 기체 명령·메시지 송신 기능은 추가하지 않았다. [QGroundControl](https://docs.qgroundcontrol.com/)과 로컬 심층 연구는 기존 관제·표시 원칙의 참고다.

## 구조와 정책

`src-tauri/src/safety.rs`의 Monitor가 연결 상태와 `telemetry::Store`에서 공통 판단을 한다. 연결 수신 작업자가 약 250ms마다 평가하고 상태 조회에서도 갱신한다. 수신·검증·기체 ID 분리는 기존 Stage 3–5 경로를 그대로 사용한다. 안전 평가에서는 그래프 복사를 피한다. UI는 기존 상태 조회를 공유하며 경고를 직접 판정하지 않는다. 경고/이벤트는 최대 200개이며 종료된 오래된 이벤트부터 제거한다. 메모리에만 존재하고 재시작 후 복구되지 않는다.

경고 단계는 `NORMAL`, `INFORMATION`, `CAUTION`, `WARNING`, `CRITICAL`, `UNKNOWN`이다. NORMAL은 **필요한 감시 항목이 모두 수신되고 이 개발 정책의 조건을 충족한다는 뜻만**이며 비행 안전이나 PX4 failsafe 정상 동작을 보증하지 않는다. 연결·배터리·GPS·위성 수·센서·자세 또는 ARM/모드/착륙 상태 등이 미확인일 때 전체 상태를 NORMAL로 표시하지 않는다. PX4 failsafe의 직접 상태는 이번 메시지 범위에서 UNKNOWN이다. 비정상 또는 미수신을 색상뿐 아니라 단계·기호·문구로 표시한다.

| 감시 | FireScene 개발용 기준 및 처리 |
|---|---|
| 연결 | 유효 HEARTBEAT 수신 후 5초 중단 또는 읽기 실패는 WARNING. 재연결 후 HEARTBEAT 재수신 시 복구 INFORMATION. 사용자 해제는 연결 상태 미확인으로 처리 |
| 배터리 | 유효 잔량 ≤30% 2초는 WARNING, ≤15% 2초는 CRITICAL. 각각 ≥33%, ≥18%에서 해당 관찰 조건을 해제. 실기체 기준이나 PX4 `BAT_LOW_THR`/`BAT_CRIT_THR`가 아님 |
| 배터리 데이터 | 미수신·무효·10초 초과는 UNKNOWN. 기존 부족/위험 경고는 유효한 회복값 없이 해결 처리하지 않음 |
| GPS | 유효 Fix 값 <2가 2초 지속되면 WARNING. Fix ≥2 시 관찰 조건 해제. 미수신·무효·5초 초과 및 위성 수 미수신/무효/0은 UNKNOWN |
| 센서 | SYS_STATUS의 present ∩ enabled 중 health 비트가 0인 경우 2초 후 WARNING. 비활성·미존재 센서는 실패로 판단하지 않고, 활성 센서 정보가 없거나 오래되면 UNKNOWN |
| 자세 최신성 | roll 또는 pitch가 2초 초과이면 CAUTION. 미수신·무효는 UNKNOWN. 두 값이 다시 유효하면 지연 조건 해제 |
| 기체 상태 | 유효 ARM·모드 변경은 INFORMATION 이벤트. 착륙 상태를 수신한 그대로 표시. HEARTBEAT의 MAV_STATE CRITICAL/EMERGENCY/FLIGHT_TERMINATION은 보고된 상태로 CRITICAL 표시하며 failsafe의 실제 동작을 추론하지 않음 |
| STATUSTEXT | 수신 심각도 0–2 CRITICAL, 3 WARNING, 4 CAUTION, 나머지 INFORMATION. 경고성 단발 문구는 실제 해제 신호가 없으므로 해당 세션에서 활성 유지; 사용자 확인만으로 해제되지 않음. 동일 심각도 메시지는 하나의 활성 항목으로 합쳐 마지막 문구를 표시 |

배터리·GPS·센서의 2초 지속 판정은 유효한 데이터가 계속 최신일 때 적용하며 데이터가 사라지면 대기 타이머를 취소한다. 고착 경고는 센서값이 다시 유효하게 해제 조건을 만족하기 전까지 보존한다. 연결 종료·새 세션에서는 이전 활성 기록을 `세션 종료 · 실제 문제 해제 여부 미확인`으로 종료한다. 이 구분은 실제 문제가 해결됐다는 뜻이 아니다. 배터리/GPS의 초기값은 검증되지 않은 개발값이고 시뮬레이션·하드웨어에 동일하게 적용된다. 자세·GPS·배터리 최신성은 [Stage 5 기록](stage5-telemetry-report.md)의 필드별 값에 의존한다.

이벤트에는 ID, 발생·종료 시각, 세션 번호, System/Component ID, 연결 출처 및 알려진 메시지 출처, 종류·심각도·원인·종료 사유·사용자 확인 여부가 있다. ID는 프로세스 내 순번이다. 확인은 인지 사실만 표시하며 활성 상태나 조건을 바꾸지 않는다. 연결 출처는 SIMULATION 또는 HARDWARE LINK일 뿐 신원 인증이 아니다. 같은 기체라도 다른 Component ID의 정보는 현 수신 정책상 혼합하지 않는다. 상세 센서별 이름, STATUSTEXT 분할 재조립, 실제 failsafe flags와 영구 기록은 미구현이다.

## 화면

기존 안전 관리 메뉴에 통신·배터리·GPS·센서·자세 최신성 요약, 활성 항목, 종료 항목 및 최근 이벤트를 추가했다. 상단과 기체 패널에서 중요한 활성 경고를 볼 수 있고 눌러 상세 화면으로 이동한다. 상태는 기호·문구와 기존 `--warning`, `--critical`, `--muted` 토큰을 사용한다. 깜빡이는 효과는 없다. IPC 조회 실패 시 현재 상태를 UNKNOWN으로 두고 보관된 마지막 이벤트를 이전 정보로 표시한다. 경고 확인과 해제를 구분한다.

## 배포와 개발 시뮬레이션 분리

기본 Cargo feature는 비어 있다. `simulation` feature를 활성화한 개발 빌드에서만 로컬 UDP 리스너·Tauri `connect_simulation` 명령을 컴파일한다. 생성기는 Cargo example이며 `required-features = ["simulation"]`이다. 프론트엔드 시뮬레이션 조작부는 Vite 개발 모드에서 해당 백엔드 feature가 활성일 때만 포함·표시한다. 기본 프로덕션 frontend bundle과 네이티브 바이너리에는 해당 리스너와 생성기를 포함하지 않는다. 테스트 전용 송신 코드는 `#[cfg(test)]` 또는 example에만 있다. 앱의 직렬 핸들은 계속 OS 읽기 전용이다.

개발용 GUI 테스트:

```bash
cd /home/jiwon/autonomous-uav-system
cargo tauri dev --features simulation
```

앱의 **시스템 설정 → 시뮬레이션 수신 시작**을 누르고 10초 내 별도 터미널에서:

```bash
cd /home/jiwon/autonomous-uav-system
cargo run --offline --manifest-path src-tauri/Cargo.toml --features simulation --example telemetry_feed -- battery-low
```

생성기는 오직 `127.0.0.1:14540`에 TEST DATA를 보낸다. `normal`, `battery-low`, `battery-critical`, `gps-loss`, `gps-stop`, `stale-attitude`, `heartbeat-stop`, `multiple`, `invalid`, `corrupt`, `other-id`, `stop`을 선택할 수 있다. `battery-low`, `battery-critical`, `multiple`은 5–16초 구간에 조건을 주고 정상값으로 복구한다. `heartbeat-stop`은 8–16초 HEARTBEAT를 중단한다. 수신 작업자는 연결 손실 시 닫히므로 복구를 확인할 때 시스템 설정에서 시뮬레이션 수신을 다시 시작해야 한다. 테스트 데이터는 SIMULATION·출처 미검증으로 표시한다. 생성기는 개발 도구이고 실제 기체 제어 명령을 생성하지 않는다.

기본 배포 빌드는 다음과 같다. Gazebo, PX4 SITL, Python 및 Node/Rust 개발 도구를 최종 앱의 필수 실행 의존성으로 추가하지 않았다. Ubuntu에는 기존 WebKitGTK/GTK 시스템 라이브러리가 필요하다.

```bash
cargo tauri build --bundles deb -- --offline
```

## 검증 결과

- Rust 개발 feature 테스트 20개 통과. 기존 가상 직렬·CRC·명령 차단·시간 초과·기체 ID 분리, 임계값과 히스테리시스, 지속 시간, UNKNOWN, 중복·동시 경고, 연결 손실/복구, 세션 전환, 200개 기록 상한 및 사용자 확인과 해제 분리 등을 포함한다. 로컬 UDP 수신부터 합성 배터리 CRITICAL 이벤트까지의 통합 테스트는 샌드박스 밖 localhost에서 통과했다.
- 별도 loopback 수신기에서 개발용 `battery-low` 생성기의 실제 MAVLink SYS_STATUS 잔량 85%와 25%를 확인했다. 전체 GUI 경고 표시·복구를 검증한 것은 아니다.
- TypeScript 검사, 프론트엔드 프로덕션 빌드 및 정적 프론트엔드 테스트 5개 통과. 경고 화면에서 활성/종료 항목과 확인 상태, SIMULATION 출처를 확인한다. 자동 테스트는 실제 Tauri GUI 클릭을 대체하지 않는다.
- Chrome headless 1440×900에서 안전 관리 메뉴 이동, 미연결 UNKNOWN 및 명령 잠금 표시를 확인했다. 이번 세션에서 디스플레이 `:1`에 연결할 수 없어 실제 Tauri GUI와 시뮬레이션 경고 표시·확인 조작은 독립적으로 검증하지 못했다. 사용자 Ubuntu 세션 자체의 문제라고 단정하지 않는다.
- 기본 프로덕션 `cargo tauri build --bundles deb -- --offline` 성공. `frontend/dist`와 릴리스 바이너리에서 `connect_simulation`, 시뮬레이션 수신 버튼 및 생성기 문자열을 찾지 못했고, DEB 파일 목록에도 생성기·PX4·Gazebo·테스트 도구가 없다. 기본 feature의 example 빌드는 `simulation` 필요 오류로 거부된다. 이 검사는 빌드 구성 및 결과물 검사이며 모든 가능한 배포 변형에 대한 보증은 아니다.
- DEB `FireScene GCS_0.1.0_amd64.deb` 3,165,846 bytes, SHA-256 `26f2b8497e146a3b43e269f8bb9da05560a436c8f1f7deb38a3c9479bb24fa64`; 릴리스 실행 파일 11,511,256 bytes. 설치·실행·제거, Windows, CPU/RAM·시작 시간은 측정하지 않았다. 위 수치는 빌드 시점의 산출물이다.

## 남은 제한 및 Stage 7

실제 기체와 PX4 설정, SITL, 경고 문구·색상의 실제 현장 가독성 및 사용자 확인 조작을 검증해야 한다. MAVLink 데이터 출처는 인증되지 않고 무선 링크 특성에 맞춘 임계값·freshness는 정해지지 않았다. 위험 메시지의 자동 해제 근거가 없는 경우 세션 종료까지 남는다. 최신 메시지만 저장하므로 다수의 연속 STATUSTEXT를 완전한 비행 기록으로 사용할 수 없다. 종료 시 모든 메모리 기록이 사라진다. 성능과 설치 크기는 실제 대상 장비에서 별도 측정해야 한다.

Stage 7에서는 지도 요구·데이터 출처와 좌표계·오프라인 정책을 별도로 설계하고 실제 위치 freshness 및 미확인 상태를 유지한다. 이번 단계에서 지도나 실제 제어를 추가하지 않았다. 로컬 커밋은 `git log -1 --stat`으로 확인하며 원격 push는 하지 않는다.
