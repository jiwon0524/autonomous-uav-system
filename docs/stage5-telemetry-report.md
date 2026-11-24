# Stage 5 — 텔레메트리와 비행 계기

## 사용자 확인 추가 (Stage 6)

사용자가 일반 Ubuntu 환경에서 Stage 5 가상 텔레메트리 수신과 화면 연동에 성공했다고 확인했다. 이는 사용자 보고이며 이 작업에서 별도 로그·스크린샷을 확보한 것은 아니다. 실제 PX4 SITL 및 기체 검증 성공을 의미하지 않는다. Stage 6부터 개발용 시뮬레이션은 `simulation` Cargo feature가 필요하고 기본 프로덕션 빌드에는 포함되지 않는다.

## 범위 및 한계

디자인 기준 `design-v1.0`, Stage 4 `945d378`에서 읽기 전용 MAVLink 수신을 확장했다. 실제 드론·무선 링크는 연결하지 않았다. 데이터 수신이 곧 장치 인증이나 실제 기체 확인을 뜻하지 않으며, SIMULATION은 사용자 지정 로컬 경로이고 HARDWARE LINK도 출처 미검증이다. 이 자동 작업 환경에서 Tauri GUI 수신 전체 경로는 독립적으로 검증하지 못했다. 사용자의 일반 Ubuntu 확인 결과는 위에 기록했다. 실제 하드웨어 및 Windows는 미검증이다. 지도, 기체 제어, 데이터 요청·송신은 구현하지 않았다.

## 디코딩과 데이터 모델

`src-tauri/src/telemetry.rs`는 Rust MAVLink common dialect의 CRC 검사 후 메시지를 디코딩한다. `connection.rs`는 먼저 유효한 PX4 HEARTBEAT로 선택한 같은 System ID **및** Component ID만 저장소에 전달한다. 다른 컴포넌트가 전달하는 같은 기체 데이터도 이번 단계에서는 보수적으로 제외된다. 서명된 프레임과 미지원 incompatibility flag는 검증 키가 없어 거부한다. 연결 해제·손실·새 세션에서 값과 그래프를 초기화한다. UI에는 상태 조회 IPC로 약 500ms마다 전달하며, 조회 실패·1.5초 타임아웃 시 마지막 수치를 숨긴다. 자동 장치 탐색은 기존 주기를 유지한다. 이 방식은 이벤트 push가 아니라 폴링이며, 화면의 구독 컴포넌트만 텔레메트리 갱신을 받는다.

각 필드는 숫자 또는 텍스트, 단위, 로컬 수신 시각, 메시지 출처, System/Component ID 및 유효성과 최신 상태를 가진다. 전송 경로는 연결 상태의 source에 별도로 표시한다. 미수신 필드와 연결 종료를 구분하고 LIVE만 숫자·계기에 사용한다. STALE/INVALID는 기존 수치를 현재 값처럼 그리지 않는다. 이 값들은 보안상 인증되지 않았으며 수신 시각은 기체 시간 동기화 결과가 아니다.

| 메시지 | 처리 범위 |
|---|---|
| HEARTBEAT | PX4 custom main/submode 일부와 armed bit. 알 수 없는 모드는 확인 불가 |
| SYS_STATUS | 배터리 잔량·전압·전류. 센서 건강 비트는 해석하지 않음 |
| BATTERY_STATUS | 배터리 ID 0의 잔량·전류. 개별 셀 전압 합산과 확장 셀은 미구현 |
| GPS_RAW_INT | fix, 위성 수, 원시 위도·경도. 추정 위치와 구분 |
| GLOBAL_POSITION_INT | 상대 고도, MSL 고도, 지상 속도, 상승 속도, heading |
| ATTITUDE / ATTITUDE_QUATERNION | 유효한 roll·pitch·yaw; quaternion은 정규화 범위를 검사 후 변환 |
| VFR_HUD | 대기·지상 속도, MSL 고도, 상승 속도, heading |
| EXTENDED_SYS_STATE / STATUSTEXT | 착륙 상태와 최신 상태 문구. 분할 STATUSTEXT 재조립은 미구현 |
| LOCAL_POSITION_NED | 북·동·하 위치, 북·동 속도 및 상승 속도. 현재 화면에서 상세 표시하지 않음 |

동일 의미의 메시지가 여럿 오면 마지막으로 유효성 검사된 메시지가 해당 필드를 갱신하며 메시지 출처가 기록된다. `SYS_STATUS` 배터리는 다중 배터리에서 모호하고, `BATTERY_STATUS`는 ID 0만 처리한다. 복수 배터리 판단과 전압 소스 선택 정책은 후속 검증이 필요하다. STATUSTEXT는 영구 이벤트 로그가 아니며 최신 메시지 하나만 저장한다.

## 단위, 유효성 및 최신성

공식 [MAVLink common 정의](https://mavlink.io/en/messages/common.html)를 기준으로 mV→V, cA→A, mm→m, cm/s→m/s, rad→degree, degE7→degree로 변환한다. GLOBAL_POSITION의 상대 고도는 home 기준, MSL은 해발이며, NED 아래 방향 속도는 부호를 뒤집어 상승 양수로 표시한다. 지상 속도는 북·동 속도 벡터의 크기이고 대기 속도와 별도다. yaw는 heading으로 대체하지 않는다. GPS 원시 좌표는 fix 2 이상 및 좌표 범위일 때만 표시한다. 위치 추정·고도는 별도의 수신값이며 GPS fix를 증명하지 않는다.

배터리 `-1`, 전압·heading·위성 수의 최대 정수 sentinel, 범위 밖 배터리·좌표·heading, NaN·무한대 및 비정상 자세를 무효로 표시한다. 상대/MSL/로컬 위치 ±100,000m, 속도 0–1,000m/s, 수직 속도 ±1,000m/s는 초기 방어 범위이며 해당 기체의 운용 한계가 아니다. 숫자 0은 정의상 유효할 때 유지한다. MAVLink에 명시적 무효값이 없는 필드는 수신 자체만으로 센서 신뢰도를 보증할 수 없다.

| 데이터 | 초기 STALE 기준 |
|---|---:|
| 자세 | 2초 |
| 위치·속도·방향 | 3초 |
| HEARTBEAT 모드·ARM, GPS 및 착륙 상태 | 5초 |
| 배터리 | 10초 |
| 상태 문구 | 30초 |

기준은 초기 UI 정책이며 PX4 스트림 주기나 무선 링크의 안전 기준으로 검증되지 않았다. HEARTBEAT가 계속 와도 각 필드는 별도로 오래됨이 된다. 5초 heartbeat 손실 시 연결 상태가 지워진다. 구체 필드 및 메시지의 의미는 [MAVLink](https://mavlink.io/en/messages/common.html), [PX4 1.16 문서](https://docs.px4.io/v1.16/en/), [QGroundControl](https://docs.qgroundcontrol.com/)을 참고했다.

## 화면과 그래프

종합 관제의 상단 모드·ARM·GPS·배터리, 상대 고도·지상 속도·heading·위성, 배터리 전압과 GPS 원시 좌표, 마지막 수신 시각 및 최근 상태 문구가 수신값을 사용한다. 지도는 여전히 연동하지 않았다. 비행 제어에는 실제 roll/pitch 수신 시에만 움직이는 인공수평계, 북쪽 기준 heading 표시, 상대·MSL 고도, 지상·대기·상승 속도 및 배터리 계기를 배치했다. 양의 roll은 수평선을 반대 방향으로 회전시키고 양의 pitch는 수평선을 아래로 이동시킨다. 실제 조종 용도로 검증된 계기는 아니다. 디자인 토큰과 두 테마를 유지한다.

배터리, 상대 고도, 지상 속도, roll·pitch·yaw 각각 최대 120개 샘플을 메모리에 보관한다. 무효값이나 수신 간격이 벌어진 곳은 선을 끊고 보간하지 않는다. 오래된 필드는 오래됨으로 표시한다. 영구 로그·재생은 없다.

## 개발용 생성기

일반 Ubuntu 그래픽 세션에서 `cargo tauri dev --features simulation`으로 Tauri 앱을 실행하고 시스템 설정에서 **시뮬레이션 수신 시작**을 선택한 직후 별도 터미널에서:

```bash
cd /home/jiwon/autonomous-uav-system
cargo run --offline --manifest-path src-tauri/Cargo.toml --features simulation --example telemetry_feed -- normal
```

앱은 10초 내 첫 HEARTBEAT가 필요하다. 실행 도구는 `127.0.0.1:14540`에만 시험 MAVLink를 전송하고 Ctrl+C로 종료한다. `normal`, `gps-loss`, `stale-attitude`, `invalid`, `other-id`, `stop`, `corrupt` 시나리오를 선택할 수 있다. `stop` 또는 Ctrl+C 후 다시 시작하면 연결 종료 후 재시작을 시험할 수 있다. 생성기는 개발용 Cargo example이며 앱에 포함되지 않는다. 프론트엔드에 수치를 직접 주입하지 않는다.

## PX4 SITL 조사

`/home/jiwon/PX4-Autopilot`에 `px4_sitl_default/bin/px4` 실행 파일이 있으며 소스 Git 설명은 `v1.18.0-beta1-759-gf90b40d61e`, `gz sim --versions`는 8.15.0이었다. 이 빌드는 실제 장비의 PX4 v1.16 검증이 아니다. 임시 `/tmp/firescene-stage5-sitl` 작업 폴더와 localhost UDP 수신기를 사용해 기존 SITL 바이너리를 사용자 정의 초기화 스크립트로 두 차례 시작했으나 스크립트가 반환값 65280으로 실패했고 수신 datagram은 0개였다. 원인은 확정하지 못했으며 SITL 데이터 수신은 **미검증**이다. 기존 PX4 설치를 재설치·삭제하지 않았다. [PX4 시뮬레이션 문서](https://docs.px4.io/v1.16/en/simulation/)의 포트 설정과 충돌 여부를 후속 조사한다.

## 검증

- Rust: 12개 테스트 통과. Stage 3/4 회귀, 읽기 전용 FD, CRC·서명 거부, 가상 serial·loopback, 시간 초과, ID 분리, 세션 초기화, 단위·무효값·개별 freshness, quaternion 및 그래프 상한 포함. UDP 테스트는 샌드박스 밖 localhost에서 수행했다.
- 개발용 생성기 `stop` 시나리오의 로컬 수신기에서 실제 MAVLink v1 ID 0·1·24·30·33 수신을 확인했다. 별도 Rust loopback 통합 테스트는 시험 프레임을 디코더·상태 저장소까지 전달하고 다른 System ID를 차단했다. 생성기 전체 시나리오와 Tauri IPC/화면을 하나의 종단간 경로로 시험하지는 않았다.
- 프론트엔드 타입 검사, 빌드와 정적 테스트: 미연결, 메뉴·명령 잠금, 시뮬레이션 표기, 자동 연결 정책 및 수평계 변환의 방향을 확인했다. 수평계 방향은 변환 단위 테스트이며 실제 장비 기준의 인간공학 검증이 아니다.
- Chrome headless에서 미연결 종합 관제 1280×800, 1440×900, 1920×1080을 캡처했다. 1280과 1440 화면을 확인했다. 비행 제어 및 실제 데이터를 포함한 크기별 Tauri GUI·시각적 상호작용은 미검증이며 일부 그래프와 이벤트는 스크롤 영역에 있다.
- `cargo tauri build --no-bundle -- --offline` Tauri 릴리스 빌드 통과. DEB 재생성·설치는 수행하지 않았다. 실기체 항목은 [체크리스트](hardware-validation-checklist.md)에서 PENDING이다.

## Stage 6 후보

PX4 SITL 정상 기동과 실제 텔레메트리·다른 컴포넌트 정책, 기기별 배터리·센서 유효성과 출처 인증, Tauri 실제 GUI 및 다양한 창 크기 테스트를 우선 검증한다. 하드웨어 검증은 준비 후 별도 진행한다. 비행 명령은 별도 범위다. 변경 파일과 로컬 커밋은 `git log -1 --stat`으로 확인한다. 원격 push는 하지 않는다.
