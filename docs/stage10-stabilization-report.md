# Stage 10 안정화·최적화·배포 보고

기준: `dev/firescene-gcs` / `e394239` / `design-v1.0`. 환경: Ubuntu 24.04.5 amd64, Rust 1.100.0, Node 22.23.4, Tauri CLI 2.12.6. 실제 Ubuntu GUI·DEB 설치, Windows 앱·장치, KOA FC 및 실기체는 **PENDING**이다. Windows 설치 파일은 생성하지 않았으며 이번 작업만으로 Windows 11 실사용 지원을 확정할 수 없다. 최종 로컬 커밋은 `git log -1`로 확인한다. 원격 push와 외부 CI 실행은 하지 않았다.

## 점검과 변경

- 프론트엔드에서 참조하지 않는 `lucide-react`를 제거했다. 아이콘은 기존 SVG를 사용한다. 기존 화면·메뉴 삭제는 없다. 임무 및 기록 화면을 지연 로딩하고, 지도는 `memo`와 GPS 필드 선택으로 다른 텔레메트리만 바뀔 때의 불필요한 렌더를 줄였다. 실제 렌더 횟수·GPU 사용량은 측정하지 않았다.
- 2초 연결 관리 폴링은 그래프와 안전 이벤트 전체를 직렬화하던 `connection_status` 대신 동일 연결 정보만 주는 `connection_summary`를 사용한다. 연결 중에는 USB 장치 재탐색을 중지하고 연결 손실 후 다시 탐색한다. 자동 연결의 고유 일련번호와 재시도 제한은 유지한다. 실제 USB 이탈·재연결은 PENDING이다.
- 기록 상태의 중복 폴링을 공유하고 마지막 구독 해제 시 타이머를 종료한다. 텔레메트리와 기록 상태 조회의 유한 타임아웃을 정리하고, 구독 종료 후 늦은 응답이 이전 상태를 되살리지 않도록 했다. 화면을 떠난 기록 목록의 늦은 결과를 무시한다. 이미 Rust에서 시작된 파일 읽기 작업은 강제 취소하지 않는다. 지도·재생 지도는 화면 종료 시 `ResizeObserver`를 해제하고 지도 인스턴스를 정리한다.
- 재생 구간은 새 `read_recording_window`로 필요한 10분 구간 이후 읽기를 중단한다. 이전 전체 스캔 명령은 보존했다. 구간 내용·순서·최대 개수를 테스트했다. 기존 목록 조회는 파일 전체의 손상을 검사한다. 새 구간 조회는 요청 범위 이후의 손상을 그 요청에서 검출하지 않으므로 파일이 목록 조회 후 바뀌었다면 해당 구간 또는 다음 전체 검사에서 발견될 수 있다. 늦은 재생 샘플 선택은 배열 전체를 복사하지 않고 역방향 탐색한다.
- Windows 파일명 테스트의 임시 경로를 Windows에서 허용되는 이름으로 바꾸었다. README의 오래된 미구현 설명을 현재 상태로 수정하고 설치·런타임·직렬 권한 안내를 추가했다. DEB 메타데이터에 Ubuntu 24.04의 `dpkg-shlibdeps`가 확인한 직접 런타임 의존성을 추가했다. 실제 깨끗한 OS 설치는 PENDING이다.

## Windows 읽기 전용 COM

기존 USB VID/PID·일련번호 탐색 및 수동/자동 연결 인터페이스를 유지한다. Windows에서 `COM` 숫자 이름만 받아 `\\\\.\\COMn` 경로로 정규화하고 `CreateFileW`를 `GENERIC_READ`, 공유 모드 0, `OPEN_EXISTING`으로 호출한다. `GetCommState`/`BuildCommDCBW`/`SetCommState`로 호스트 직렬 설정을 적용하고, 즉시 반환 읽기에서 0바이트는 `WouldBlock`으로 처리해 기존 50ms 대기·HEARTBEAT 손실 경로를 사용한다. 쓰기 핸들로 재시도하거나 `Write`를 노출하지 않는다. 이는 호스트 포트 설정이며 기체 파라미터 명령이 아니다. 드라이버가 읽기 전용 핸들에서 설정을 거부할 수 있으며 오류로 종료한다. 포트 열기나 제어선 설정이 기체에 미치는 영향도 실제 장치에서 확인해야 한다.

[Microsoft의 통신 핸들 규칙](https://learn.microsoft.com/en-us/windows/win32/devio/communications-resource-handles)과 [COMMTIMEOUTS](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-commtimeouts)를 참고했다. 순수 COM 이름 검증 테스트는 Linux에서 통과했다. Windows Rust 타깃을 설치한 뒤 COM 구현만 분리한 `cargo check --target x86_64-pc-windows-gnu`가 통과했다. **전체 Windows 앱 교차 검사는 `x86_64-w64-mingw32-gcc` 부재로 `aws-lc-sys`에서 실패했고, 실제 Windows 빌드·장치 열기·쓰기 불가 정책·점유 및 재연결은 미검증이다.**

## 기록 안정성

Stage 9의 초당 스냅샷을 유지한다. 안전 이벤트는 JSONL의 별도 이벤트 줄로 기록하며 기록 전 활성 경고는 구분한다. 1초보다 짧은 연결 변화·상태 메시지, 또는 1초 사이 200개 안전 이력의 축출은 놓칠 수 있다. 전체 MAVLink 원본 기록은 향후 정확도·용량·개인정보 요구를 검증한 뒤 검토할 후보이며 이번에 추가하지 않았다.

연결이 끊기면 관측된 단계 변화를 기록하며 사용자가 중지하거나 새 연결 세션이 감지될 때까지 이전 세션에 남는다. 새 세션이 확인되면 혼합하지 않고 미완료로 종료한다. 종료 줄이 없는 파일과 마지막 미완성 줄은 앞부분을 복구하고, 중간 손상은 원본을 보존한 채 표시한다. 파일당 50 MiB, 200개 및 구간 최대 1600개·이벤트 1000개 제한은 유지한다. 디스크 쓰기 실패는 기록을 중지하며 통신 스레드와 분리된다. 종료 시 기록 스레드를 합류하고 동기화하지만 느리거나 멈춘 파일시스템의 종료 지연 및 실제 강제 종료·디스크 부족 GUI 시험은 PENDING이다.

## 성능과 크기

측정은 SSH의 동일 Ubuntu 머신에서만 수행했다. GUI 기동 시간, 유휴·수신·지도 상태 CPU/RAM, GPU와 실제 장시간 재생은 **PENDING**이다. 아래 빌드 크기는 압축 전 파일 크기이며 실행 속도 개선을 뜻하지 않는다.

| 항목 | Stage 9 기준 | Stage 10 |
|---|---:|---:|
| Ubuntu DEB | 5,602,686 bytes (5.34 MiB) | 5,622,548 bytes (5.36 MiB) |
| 릴리스 실행 파일 | 18,392,352 bytes | 18,475,296 bytes |
| 프론트엔드 dist 디렉터리 (`du -sb`) | 667,607 bytes | 670,473 bytes |
| 초기 JS 번들 (Vite 표시) | 약 636.7 kB | 약 615.45 kB |

Stage 10은 임무 약 8.48 kB 및 기록 약 15.64 kB를 별도 지연 청크로 분리했다. 전체 dist는 줄지 않았고 메인 청크는 여전히 500 kB Vite 경고가 있다. 외부 웹폰트나 대용량 지도 자산을 추가하지 않았다.

합성 10시간·36,000 샘플 파일 14,821,043 bytes에서 중간 10분 600개를 조회한 **단일 디버그 테스트**는 기존 전체 스캔 396ms, 이후 prefix 스캔 205ms였다. 두 번째 측정은 파일 캐시와 실행 순서의 영향을 받으며 실제 사용자 속도·CPU·메모리 개선을 보장하지 않는다. 끝부분 탐색은 여전히 대부분의 파일을 읽고, 목록과 이벤트 조회는 전체를 읽는다. 테스트: `cargo test --offline --manifest-path src-tauri/Cargo.toml synthetic_long_recording_scan -- --ignored --nocapture`.

## 배포 및 분리

`cargo tauri build --bundles deb -- --offline --locked` 성공. 파일은 `src-tauri/target/release/bundle/deb/FireScene GCS_0.1.0_amd64.deb`이다. DEB 메타데이터는 WebKitGTK/GTK와 `libudev1` 등 Ubuntu 24.04 런타임 라이브러리를 선언한다. 이 머신의 `ldd`에서 누락 라이브러리는 없었으나 깨끗한 설치 환경과 실행은 시험하지 않았다. 설치 후 사용자에게 Python·Node·npm·Rust·Cargo·ROS2·Gazebo·PX4 SITL·QGroundControl이 요구되지 않도록 패키지 구성했다. 운영체제 라이브러리·USB 드라이버·온라인 지도는 별도다.

`.github/workflows/desktop-build.yml`은 `workflow_dispatch` 전용 Ubuntu 24.04 DEB와 Windows NSIS 빌드 절차다. 파일만 준비했고 실행·업로드하지 않았다. Windows WebView2는 설정상 오프라인 설치 프로그램을 포함하도록 되어 있으나 실제 결과물·설치는 검증하지 않았다. 아티팩트 서명과 배포 경로도 미구현이다.

기본 feature에서 개발 UDP 수신과 합성 생성기는 `cfg(feature = "simulation")` 및 예제 `required-features`로 제외된다. 기본 feature로 `telemetry_feed` 릴리스 예제 빌드는 거부되었다. 최종 DEB 목록에 Gazebo·SITL·생성기·테스트 파일이 없고 프론트엔드 산출물에 시뮬레이션 연결 명령/버튼 문자열이 없으며, 릴리스 바이너리 심볼 검사에서 해당 수신 구현 심볼이 발견되지 않았다. capability 매니페스트에는 개발 빌드용 명령 권한 이름이 남지만 프로덕션 핸들러와 수신 구현은 등록되지 않는다. 3D 실험 feature도 기본 비활성이다. 이 검사는 확인한 빌드 구성에 한정된다.

## 보안 및 안전 점검

`main`에만 열거형 Tauri capability를 부여하고 원격 출처 capability를 추가하지 않았다. 3D는 기본 비활성이며 원격 WebView 권한 거부의 실제 런타임 검증은 PENDING이다. 키는 사용자 로컬 설정에 있고 교체 가능하지만 평문·클라이언트 타일 요청에 노출되므로 배포 전 교체하고 이용 도메인 제한을 확인한다. 실제 키는 보고서·소스·패키지에 넣지 않았다. 임무와 기록은 별도 사용자 데이터 디렉터리, 입력 크기·좌표·파일 ID 검사와 저장 오류 처리를 사용한다. 같은 OS 사용자에 대한 악의적 심볼릭 링크·동일 계정 탈취까지 격리하는 저장소는 아니다.

MAVLink는 CRC·기체 식별·버전 및 메시지 수신 제한을 적용하지만 송신자 암호학적 인증은 제공하지 않는다. 수신 루프는 한 번에 4096바이트로 작업을 제한하고 연결·오류 시 상태를 갱신한다. 테스트 코드의 프레임 생성 및 개발 예제 외에 기체 명령 송신 경로를 추가하지 않았다. 직렬 핸들은 읽기 전용이며 Windows 드라이버가 이를 따르는지는 실측 PENDING이다. 안전 경고 임계값은 실기체 검증 전 보조 정보다.

## 테스트와 사용자 확인

- Rust 기본: 35개 통과, 합성 성능 시험 1개는 기본에서 제외; capability 구성 테스트 1개 통과. 시뮬레이션 feature: 36개 통과·성능 시험 1개 제외, capability 1개 통과. 루프백 UDP는 샌드박스 밖에서 승인 후 실행했다. 별도 성능 시험도 통과했다.
- 프론트엔드 테스트 6개 파일, TypeScript 검사와 Vite 빌드 통과. 공유 폴링 구독·타이머 정리 및 늦은 응답 무시, GPS 선택자, 포트 탐색 분기, 기존 임무·재생·안전 오프라인 회귀를 포함한다. 실제 GUI 상호작용 테스트는 아니다.
- Ubuntu DEB 생성·의존성 메타데이터 확인 통과. Windows 전체 빌드 실패와 미생성, 실제 OS 설치·실행 및 하드웨어 확인은 별도로 PENDING이다.
- 최종 재검증에서 프론트엔드 6개 테스트 파일과 TypeScript를 포함한 Vite 빌드, Rust 기본 35개 및 simulation 36개와 각 capability 검사 1개가 통과했다. 각 Rust 실행의 합성 성능 시험 1개는 제외했다. 동시 실행 시 테스트 임시 경로 충돌이 있어 순차 재실행했고, simulation UDP는 샌드박스의 바인딩 제한 때문에 승인된 환경에서 재실행했다. 기존 DEB 생성 후 빌드 입력 파일의 수정 시각 변경이 없어 DEB 재빌드는 하지 않았다. 이는 수정 시각 기반 확인이며 재현 빌드나 바이너리 일치 증명은 아니다.

사용자 실행·설치 방법은 [README](../README.md), 단계별 기대 결과와 PENDING 항목은 [통합 검증 체크리스트](integration-validation-checklist.md)를 따른다. 3D 실험, 실제 기체·Windows 검증 및 정밀 원본 로그는 후속 과제다. 자율비행 알고리즘이나 비행 명령은 추가하지 않았다.
