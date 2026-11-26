# Stage 3 — USB 직렬 및 MAVLink 수신 기반

## 범위와 상태

Ubuntu에서 사용자가 선택한 USB 직렬 포트를 읽기 전용으로 열고 PX4 HEARTBEAT를 확인하는 기반을 구현했다. 실제 KOA FC는 이 환경에서 확인하지 못했다. 따라서 실제 하드웨어 검증, Windows 직렬 연결, 실제 Tauri GUI 상호작용은 미완료다. 장치 존재, 유효한 PX4 형식의 수신, 실제 장비의 신원 확인은 서로 다르다. CRC는 오류 검출이며 송신자 인증이 아니다. 이 구현은 실제 기체와 시뮬레이터를 구분하지 못한다.

기준은 `design-v1.0` (`53b07c6`)이며 디자인 토큰과 9개 메뉴를 유지하고 시스템 설정에 연결 관리 UI만 추가했다. 기준 자료: [디자인 시스템](design-system.md), [디자인 확정](design-freeze-v1.md), [Stage 2 기록](stage2-desktop-report.md), [리뉴얼 기록](aerospace-design-renewal.md), 로컬 `references/research/technical-research.txt`와 `design-research.txt`. 로컬 참고 자료는 Git에 포함되지 않는다.

## 기술 비교와 선택

| 방식 | 연결·확장 및 배포 | 이번 단계 판단 |
|---|---|---|
| MAVSDK Python | PX4 중심 고수준 API와 직렬·네트워크 확장, Python 패키지/네이티브 런타임 번들 필요 | 향후 기능 후보. 버전별 바인딩 차이와 송신 동작을 별도로 검증해야 한다. |
| MAVSDK Server | 별도 프로세스와 IPC로 언어 분리, 플랫폼별 바이너리 및 생명주기 관리 필요 | MAVSDK는 연결 시 자체 HEARTBEAT를 송신할 수 있어 무송신 요구에 그대로 사용하지 않았다. |
| PySerial | Windows/Linux 직렬 포트 열거·I/O 가능, MAVLink 검증은 별도 필요 | Python 번들 및 관리가 필요하다. |
| pymavlink | 직렬·TCP·UDP와 MAVLink 처리, 메시지 송수신 API 지원 | Python 런타임 번들과 송신 경로 통제가 별도로 필요하다. |
| Rust `serialport` + `mavlink` | Tauri에 직접 정적 통합, 직렬 탐색과 공통 dialect CRC 검사, 별도 Python/server 불필요 | 선택. Linux는 별도 OS 읽기 전용 핸들만 사용한다. TCP/UDP 및 Windows 수신 구현은 후속 과제다. |

`serialport`는 열거에만 사용한다. `mavlink`의 공통 dialect 및 CRC 검증을 사용하고 transport 송신 feature는 켜지 않았다. UI는 Tauri 명령을 통해 상태만 조회한다. 프로덕션 코드에는 MAVLink 직렬화·전송 또는 비행 명령 IPC가 없다. Linux 디스크립터는 `O_RDONLY`이고 테스트에서 쓰기 실패를 확인했다. MAVSDK와 달리 GCS HEARTBEAT도 전송하지 않는다. 단일 작업자만 하나의 포트를 소유한다. Linux `flock`과 `TIOCEXCL`은 협력하는 프로세스 및 이후 열기를 제한하지만, 이미 열려 있는 모든 비협력 프로그램을 완전히 탐지한다고 보장하지 않는다. QGroundControl 등은 반드시 종료한다.

공식 근거: [PX4 1.16 직렬 설정](https://docs.px4.io/v1.16/en/peripherals/serial_configuration), [PX4 MAVLink 포트](https://docs.px4.io/v1.16/en/peripherals/mavlink_peripherals), [MAVLink HEARTBEAT](https://mavlink.io/en/services/heartbeat.html), [MAVLink 메시지 정의](https://mavlink.io/en/messages/common.html), [서명](https://mavlink.io/en/guide/message_signing.html), [MAVSDK HEARTBEAT 송신](https://mavsdk.mavlink.io/main/en/cpp/guide/heartbeat_watchdog.html), [MAVSDK Python](https://mavsdk.mavlink.io/main/en/python/), [pymavlink](https://mavlink.io/en/mavgen_python/), [pySerial 열거](https://pyserial.readthedocs.io/en/latest/tools.html), [QGroundControl 링크](https://docs.qgroundcontrol.com/master/en/qgc-user-guide/settings_view/comm_links.html), [Tauri 명령](https://v2.tauri.app/develop/calling-rust/), [Rust serialport](https://docs.rs/serialport/latest/serialport/).

## 구조 및 상태

`src-tauri/src/connection.rs`의 Manager가 장치 열거, 단일 작업자, 상태와 종료를 담당한다. Linux의 열기는 `O_RDONLY`, nonblocking, raw 8N1 및 선택한 속도, 독점 시도를 사용한다. USB 유형으로 열거된 포트만 선택할 수 있다. `/dev/ttyACM*`, `/dev/ttyUSB*`는 OS가 USB 직렬 포트로 보고할 때 표시된다. `/dev/serial/by-id/` 별칭은 별도 목록으로 표시하거나 선택하지 않는다. VID/PID, 제조사, 제품, 일련번호와 읽기 권한 여부는 OS 제공 범위에서 표시한다. 읽기 권한 표시는 개략적이며 점유 여부는 열 때 확인한다. Windows COM 열거 구조는 라이브러리 수준에서 가능하지만 읽기 전용 Windows transport는 미구현이고 명시적으로 오류를 반환한다.

- `DISCONNECTED` → 열거 결과가 있으면 `PORT_DETECTED` → 사용자의 연결 요청 시 `CONNECTING` → 포트 열기 성공 시 `WAITING_HEARTBEAT`.
- MAVLink 1/2 CRC가 유효하고 system/component ID가 0이 아니며, HEARTBEAT가 PX4 autopilot 12와 지원 기체 유형을 보고할 때 `CONNECTED`. 이 식별은 메시지 내용에 근거하며 KOA 모델이나 PX4 펌웨어 버전을 입증하지 않는다. 첫 기체의 system/component ID를 고정한다.
- 10초 동안 유효한 HEARTBEAT가 없으면 `ERROR`, 연결 후 5초간 없거나 읽기가 중단되면 `CONNECTION_LOST`. 연결 중단 시 현재 기체 정보는 지우고 마지막 수신 시각만 이전 수신으로 표시한다. 사용자가 해제하면 `DISCONNECTED`.
- 서명 검증 키·재전송 방어가 없으므로 서명된 MAVLink 2와 미지원 incompatibility flag는 거부한다. 허용된 미서명 메시지는 인증되지 않는다. ARM, 비행 모드, 배터리, GPS, 고도, 속도 및 안전 데이터는 해석하지 않으며 확인 불가 상태와 명령 잠금을 유지한다.

타임아웃은 초기값이며 실제 연결 품질에 대한 검증 결과가 아니다. 포트가 열려도 HEARTBEAT 수신 전에는 연결로 표시하지 않는다. PX4 USB가 프로토콜 자동 감지 모드에서 상대 송신을 기다리거나 기체가 스트리밍하지 않으면 이 무송신 구현은 시간 초과할 수 있다. 이를 해결하기 위한 송신이나 FC 설정 변경은 수행하지 않았다.

## 검증

- Rust 자동 시험 7개: 시험용 pseudo-terminal에서 MAVLink 1 수신, 손상 CRC·다른 autopilot 거부, MAVLink 2 CRC 및 미검증 서명 거부, 수신 대기 시간 초과, 수신 중단, 가상 장치 분리, 해제, 존재하지 않는 포트, 독점 재열기 실패 및 읽기 전용 디스크립터 쓰기 실패. 시험 데이터는 실제 기체 데이터가 아니다.
- 이 환경의 USB 직렬 열거 결과 0개. 실제 권한 부족 장치와 이미 다른 프로그램에 점유된 실제 USB 장치는 시험하지 않았다. `access()`는 권한의 근사치다.
- TypeScript 검사와 프론트엔드 빌드 통과. 정적 렌더링 시험은 초기 9개 메뉴·미연결·명령 잠금과 가상 연결/손실 상태 표시를 확인한다. 브라우저 클릭, Tauri IPC와 실제 장치가 연결된 전체 경로는 미검증이다.
- `cargo tauri build --no-bundle -- --offline` 릴리스 성공. 새 DEB 생성, 설치, Windows 빌드 및 GUI 실기 검증은 이번 단계에서 수행하지 않았다. 기존 DEB는 Stage 3 구현을 포함하지 않는다.

## Ubuntu에서 실제 KOA FC 시험 준비

1. **프로펠러가 모두 제거됐는지 직접 확인**하고 기체를 안전하게 고정한다. USB 연결로 FC가 켜질 수 있다. 예상치 못한 모터 동작 가능성을 고려해 배선과 전원 상태를 확인한다.
2. QGroundControl과 포트를 사용할 다른 프로그램을 종료한다. FC를 데이터 가능한 USB 케이블로 연결한다. 필요한 드라이버와 장치 권한은 Ubuntu의 일반 사용자 세션에 적용한다. `ls -l /dev/ttyACM* /dev/ttyUSB*` 및 `id -nG`로 확인할 수 있다. 장치 그룹은 시스템에 따라 다르며 `dialout` 등이 사용될 수 있다. 권한이 없으면 관리자가 해당 장치에 필요한 권한을 부여하고 새 로그인으로 확인한다. root 실행이나 전체 장치 권한 완화는 사용하지 않는다.
3. 저장소 루트에서 `cargo tauri dev`를 실행하고 **시스템 설정 → 새로고침**에서 새 장치를 선택한다. 속도는 우선 115200, 무선 시 해당 링크 설정을 확인한다. VID/PID와 일련번호를 다른 USB 장치와 구별한다. 포트 선택 자체는 기체 확인이 아니다.
4. **연결**을 누르고 PX4 HEARTBEAT, System ID, Component ID, 수신 시각을 확인한다. 숫자와 상태를 실제 출력으로 기록한다. 다른 텔레메트리 값은 미수신이어야 하고 비행 명령은 잠겨 있어야 한다.
5. 케이블을 분리해 연결 끊김과 기체 정보 제거를 확인한다. 이후 앱에서 해제한다. 시간 초과면 케이블·권한·포트 점유·속도와 PX4 USB 스트림 설정을 확인한다. FC 설정을 변경하거나 GCS HEARTBEAT 송신을 추가하기 전 별도 검토한다.

최종 배포에는 Python, Rust, Node 및 MAVSDK 설치가 필요 없도록 Tauri 실행 파일에 Rust 코드를 포함한다. Ubuntu는 적절한 udev/그룹 권한과 시스템 WebKitGTK가 필요하다. Windows 11은 장치별 드라이버, COM 열거, 읽기 전용 핸들·독점 및 실제 패키지 검증이 필요하다. ROS2와 Gazebo는 앱 의존성이 아니다.

## Stage 4 후보

실제 FC 수신과 USB 분리 검증을 우선 완료한 뒤, 장치 식별과 자동 재연결, 보다 강한 장치 식별 및 서명 검증, 통신 이벤트 기록, Windows 읽기 전용 transport를 설계·시험한다. 데이터 freshness와 실제/시험 출처 구분은 실제 모델을 만들 때 구현한다. 송신·비행 명령은 별도 범위와 승인 전까지 추가하지 않는다. 로컬 커밋은 이 보고서를 포함한 Stage 3 커밋의 `git log -1`로 확인한다. 원격 push는 하지 않는다.
