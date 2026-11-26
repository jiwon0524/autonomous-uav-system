# Stage 14 QGroundControl 연동 보고

기준: `dev/firescene-gcs`, Stage 13 `5dc7416`, `design-v1.0`, `alpha-v0.1.0`. 로컬 커밋은 이 문서를 포함한 `git log -1`로 확인한다. 원격 push, 실제 기체 연결·비행 시험 및 QGC GUI 실행은 하지 않았다. F450·KOA FC·PX4 v1.16.0·M10 GPS·KOA 텔레메트리는 사용자 제공 구성으로 이 작업에서 확인하지 않았다.

## 조사와 선택

[연동 안내](qgroundcontrol-integration-guide.md)에 공식 QGC/PX4/MAVLink 자료, serial·무선·UDP/TCP·다중 GCS·forwarding과 A–E 대안, 라이선스 및 설치 절차를 정리했다. QGC forwarding은 공식 V5.0 문서 기준 연결된 차량에서 지정 UDP 대상으로 단방향이다. FireScene에 전달되는 사본은 단순 수신이며 QGC 또는 Router와 같은 포트를 공유하거나 직렬 장치를 이중 개방하지 않는다. QGC가 직렬을 소유하는 구조를 우선 선택한다. Router는 대안이며 독립 구성·보안 및 플랫폼 검증이 필요하다. QGC 앱 내 임베딩, 코드 재사용, Qt/QGC 번들 및 앱에서 실행은 하지 않았다.

이 Ubuntu에서 PATH·dpkg·일반 데스크톱 등록에서 확인되지 않았으나 다운로드 디렉터리의 실행 권한 있는 QGroundControl AppImage는 발견했다. 크기 198,601,208 bytes, 내부 desktop/AppStream 메타데이터 `5.2.0`, forwarding 관련 심볼과 Daily 설정 디렉터리가 있다. 이는 실행·GUI·정확한 소스 커밋이나 forwarding 동작 성공을 증명하지 않는다. UI 자동 설치 확인은 UNKNOWN이다. QGC 공식 [master Telemetry 설정](https://docs.qgroundcontrol.com/master/en/qgc-user-guide/settings_view/telemetry.html)과 [V5.0 MAVLink 설정](https://docs.qgroundcontrol.com/Stable_V5.0/en/qgc-user-guide/settings_view/mavlink.html)을 비교했으나 발견된 AppImage에서 직접 메뉴를 검증하지 않았다.

## 구현·안전

시스템 설정에서 QGC 또는 Router 경로와 포트 선택 후 `127.0.0.1` UDP 수신을 시작한다. 포트 0·미지원 경로 거부, 포트 점유 오류, HEARTBEAT 시간 초과 및 연결 해제, 연결 경로와 송신자 미인증 표시를 지원한다. 선택된 이름은 실제 소스 판별이 아니다. 기존 직접 직렬 및 개발용 simulation과 동일한 디코더·안전 감시를 사용하되 worker는 하나만 허용한다. UDP 선택 시 직렬 자동 연결 허용을 끈다. 전달 데이터그램을 bounded 버퍼에 보존해 한 패킷의 여러 MAVLink 프레임을 처리한다. 포트는 사용자가 지정하며 remote 인터페이스에 바인딩하지 않는다. 다른 컴퓨터에서의 직접 UDP 및 TCP는 미지원이다.

UDP 수신은 `recv`만 사용하며 비행 명령·HEARTBEAT·파라미터·임무 업로드를 보내지 않는다. 기본 앱은 개발용 simulation handler와 생성기를 포함하지 않는다. MAVLink CRC는 신원 인증이 아니고 서명된 프레임은 검증하지 못해 거부한다. 로컬의 다른 프로세스가 데이터를 위조할 수 있고 UDP 손실·혼합·지연을 방지하지 못한다. PX4 failsafe와 FireScene 보조 안전 감시는 별개다. QGC 및 다른 GCS의 제어 권한은 FireScene이 제한하지 않는다.

## 검증

- 프론트엔드 8개 테스트 파일, TypeScript 및 Vite 빌드 통과. 메인 청크 500 kB 경고는 남는다.
- Rust 기본 47개, simulation 48개 및 각 Tauri capability 구성 검사 1개 통과; 합성 성능 시험 각 1개 제외. 로컬 UDP의 포트 점유·다중 프레임 전달·부분 읽기·수신 경로·HEARTBEAT 손실·연결 해제 및 송신자에게 응답 없음 확인. 실기체 신뢰성 시험은 아니다. 루프백 검사는 승인된 환경에서 실행했다.
- 프로덕션 feature에서 telemetry_feed 예제 빌드는 feature 요구로 거부되었고, 프론트엔드 번들과 릴리스 심볼에서 simulation 연결 구현을 찾지 못했다. 소스 검색에서 통신 송신은 테스트용 코드로 한정됨을 확인했다. QGC를 실행해 forwarding 전체 경로와 실제 안전 감시 동시 동작은 아직 시험하지 않았다.
- Ubuntu 24.04 amd64 Tauri 릴리스·DEB 성공. `src-tauri/target/release/bundle/deb/FireScene GCS_0.1.0_amd64.deb` **5,708,628 bytes**, Stage 13의 5,693,670 bytes보다 **14,958 bytes 증가**. 실행 파일 18,694,520 bytes, frontend/dist 691,277 bytes. DEB 목록에 QGC/Qt 배포 파일이나 개발 생성기는 없다. DEB 설치·GUI 실행은 PENDING이다.

Windows 전체 앱·UDP 및 실제 QGC GUI·KOA FC·USB 포트 충돌·무선 텔레메트리·실기체 검증은 **PENDING**이다. 데이터 수신은 송신자와 기체 신원을 증명하지 않는다. QGC와 FireScene을 독립 실행해 로컬 대상 포트 및 데이터 손실 상태를 검증하는 단계별 방법은 [연동 안내](qgroundcontrol-integration-guide.md)를 따른다.
