# QGroundControl 연동 안내

Stage 14의 연동은 **QGroundControl을 별도로 실행하고 MAVLink forwarding의 사본을 FireScene이 로컬 UDP로 읽는 방식**이다. QGroundControl이 기체 설정·보정·정비·비행 시험을 담당하며 FireScene은 읽기 전용 관제와 소방 특화 화면을 담당한다. FireScene은 QGroundControl이나 Qt를 포함·실행하지 않는다. 이 단계에서 GUI·실기체 검증은 하지 않았다.

## 공식 자료 및 동작 범위

[PX4 v1.16 MAVLink 설명](https://docs.px4.io/v1.16/en/middleware/mavlink)은 MAVLink를 메시지·명령 및 상위 프로토콜로 설명한다. [PX4 v1.16 텔레메트리 문서](https://docs.px4.io/v1.16/en/data_links/telemetry)는 무선 텔레메트리로 GCS와 기체를 연결할 수 있다고 설명한다. USB 직렬 또는 무선 수신기가 OS에 제공하는 직렬 링크는 오직 한 프로그램이 소유하도록 구성한다. 이 저장소의 KOA FC·M10·무선 모듈과 속도·드라이버는 아직 검증하지 않았다. [PX4 v1.16 MAVLink 주변장치](https://docs.px4.io/v1.16/en/peripherals/mavlink_peripherals)의 여러 MAVLink 인스턴스는 포트·설정에 의존하며 모든 연결에서 모든 데이터가 보장된다는 뜻은 아니다.

[QGroundControl master Telemetry 설정](https://docs.qgroundcontrol.com/master/en/qgc-user-guide/settings_view/telemetry.html)과 [V5.0 MAVLink 설정](https://docs.qgroundcontrol.com/Stable_V5.0/en/qgc-user-guide/settings_view/mavlink.html)에 따르면 forwarding은 연결된 기체의 MAVLink를 다른 UDP endpoint로 보내는 단방향 기능이다. QGC에 돌아오는 해당 endpoint의 메시지는 무시한다. UI 경로·필드 이름 및 사용 가능성은 설치된 QGC 버전마다 확인해야 한다. [MAVLink 연결 개념](https://mavlink.io/en/mavgen_python/)에는 serial, UDP 및 TCP 예시가 있다. QGC의 각 링크 지원·세부 동작을 FireScene의 구현 범위로 혼동하지 않는다. FireScene은 현재 전달 수신에 TCP, 원격 네트워크 또는 여러 동시 기체를 지원하지 않는다. 한 스트림의 첫 유효 PX4 HEARTBEAT의 시스템·컴포넌트 ID를 채택한다.

## 수동 GUI 설정 (기체 연결 전 오프라인 시험 권장)

1. QGroundControl을 별도로 설치하고 실행한다. FireScene이 QGC 설치 여부나 버전을 판정하지 못하면 **UNKNOWN**으로 표시한다. 이 머신의 다운로드 디렉터리에서 실행 권한이 있는 `QGroundControl-x86_64.AppImage` (198,601,208 bytes)를 찾았다. 내부 AppStream/desktop 메타데이터의 버전은 5.2.0이며 Daily 설정 디렉터리도 존재한다. 실행·GUI 및 실제 빌드 식별자는 확인하지 않았다. PATH·일반 애플리케이션 등록 및 dpkg에서는 확인되지 않았다.
2. QGC 설정에서 MAVLink forwarding을 켜고 대상 주소를 `127.0.0.1:14560`으로 지정한다. QGC의 설치 버전에서 UI 명칭과 대상 포트 지원을 확인한다. 다른 포트를 사용하면 아래 값도 일치시킨다. 이 예시는 동일한 OS의 동일 네트워크 네임스페이스에서만 통한다. Mac의 QGC와 SSH 대상 Ubuntu는 서로 다른 loopback이다.
3. FireScene 시스템 설정의 **QGroundControl forwarding**에서 로컬 UDP 포트 `14560`을 선택하고 수신을 시작한다. 수신 포트가 이미 점유됐으면 다른 포트를 선택하고 QGC 대상도 바꾼다. 실제 데이터 없이 HEARTBEAT 대기 후 시간 초과는 정상이다.
4. 연결 전 반드시 FireScene의 직접 직렬 자동 연결을 끄고, QGC만 직렬 포트를 소유하게 한다. UDP 모드 전환 시 FireScene 자동 연결은 해제된다. 직접 연결로 전환할 때는 기존 수신을 해제하고 QGC가 해당 포트를 닫은 뒤 포트를 선택한다. 점유/권한 오류를 확인한다.
5. HEARTBEAT와 각 필드의 출처·시각·미수신 표시를 확인한다. `QGroundControl 전달 (사용자 선택)`은 송신자의 신원 또는 기체를 인증하지 않는다. MAVLink CRC는 인증이 아니고 이 앱은 MAVLink 서명을 검증하지 않으며 서명된 프레임을 거부한다. 로컬 다른 프로세스도 UDP를 보낼 수 있으므로 신뢰할 수 있는 격리 환경에서만 시험한다.

Router 선택도 같은 UDP 리스너를 사용하며 Router 자체를 설치하거나 실행하지 않는다. [MAVLink Router 공식 문서](https://github.com/mavlink-router/mavlink-router)는 UART·UDP·TCP endpoint와 다중 분배를 설명한다. Router가 직렬을 소유할 때 QGC와 FireScene에는 별도 endpoint를 사용해야 하며 Router의 기본 TCP 서버 등 추가 노출과 양방향 명령 전달을 별도로 통제해야 한다. 이 구성은 미검증이며 Windows 지원 여부도 이 작업에서 확인하지 않았다. 다른 GCS가 명령을 보내지 못한다는 보장은 없다.

## 권한 및 안전

FireScene은 UDP socket에 `recv`만 하고 heartbeat, 명령, 설정·임무 업로드를 보내지 않는다. 연결된 각 GCS와 라우터는 별개의 권한을 가질 수 있다. QGC의 제어 권한과 기체의 PX4 failsafe는 FireScene 안전 경고와 별개다. ARM/DISARM/TAKEOFF/LAND/OFFBOARD/임무 업로드/기체 파라미터 변경 기능은 FireScene에 없다. 패킷 손실·지연·중복·혼합 송신자·잘못된 정체성으로 표시가 틀릴 수 있다. UDP는 재전송 및 순서 보장이 없으며 연속 수신과 지상 제어를 보장하지 않는다.

실기체는 현재 보류한다. 추후에는 조직의 지상 시험 절차와 기체 제작사·PX4 지침에 따라 안전한 전원·프로펠러 제거를 확인하고 QGC 단독 링크, 포트 점유, forwarding 후 수신값과 손실 시 표시를 먼저 검증한다. 비행 시험 여부와 조건은 별도 결정한다.

## 대안 및 라이선스

| 방식 | 평가 |
|---|---|
| A 별도 실행, 각자 직접 직렬 | 단순하나 같은 포트 점유 충돌. 서로 다른 물리 링크라면 별도 검증 필요 |
| B QGC forwarding | 본 단계 선택. QGC가 직렬 소유하고 UDP 사본을 전달; QGC·연결이 단일 장애 지점. OS/버전 GUI 시험 필요 |
| C MAVLink Router | 다중 endpoint·유연성, 추가 설치·설정·명령 라우팅 및 Windows 지원 검토 필요 |
| D QGC 코드 통합 | Qt·빌드/유지보수 및 라이선스 검토 비용; 이 단계에서 재사용하지 않음 |
| E 앱에서 QGC 실행 | 설치 경로·AppImage·Windows 배포 차이가 크고 안전한 실행 대상을 확인하기 어려워 미구현. 임의 셸 권한 없음 |

QGC의 [공식 소스 라이선스 안내](https://github.com/mavlink/qgroundcontrol/blob/master/.github/COPYING.md)는 Apache 2.0과 GPLv3 이중 라이선스 및 Qt 라이선스 조건을 설명한다. 실제 특정 버전·배포 바이너리의 Qt 및 제3자 구성요소와 고지·소스 제공 등 조건을 재배포 전에 별도 확인해야 한다. FireScene은 QGC 코드·바이너리·Qt를 번들하지 않으며 통합 설치 파일 여부는 별도 판단한다. 기존 태그와 라이선스는 변경하지 않는다.
