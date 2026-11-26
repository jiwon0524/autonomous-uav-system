# FireScene GCS

FireScene GCS는 소방 현장 공간정보, 임무 계획, 수신 텔레메트리와 안전 경고를 확인하는 Tauri 2 데스크톱 앱입니다. Aerospace Dark와 Soft Neutral Day를 지원합니다. 기체 연결은 읽기 전용이며 ARM, 이륙, 착륙, 임무 업로드와 비행 명령은 구현하지 않았습니다. 안전 감시는 개발 중인 보조 기능으로 기체의 failsafe를 대체하지 않습니다.

VWorld 2D 지도·검색, 임무 로컬 저장, 초당 스냅샷 비행 기록과 별도 재생 화면이 구현되어 있습니다. 실제 지도 GUI, 실제 기체 및 Windows 장치 검증은 아직 완료되지 않았습니다. 3D 지도 실험 기능은 기본적으로 비활성입니다. 자세한 제한은 [통합 검증 체크리스트](docs/integration-validation-checklist.md)와 각 단계 보고서를 확인하세요.

## Ubuntu 설치와 실행

Ubuntu 24.04 amd64용 DEB가 제공된 경우 파일 위치에서 다음을 실행합니다.

```sh
sudo apt install ./FireScene\ GCS_0.1.0_amd64.deb
```

설치 후 애플리케이션 메뉴에서 **FireScene GCS**를 실행합니다. DEB는 운영체제의 WebKitGTK 등 필요한 라이브러리에 의존하며 패키지 관리자가 설치합니다. 일반 사용자가 프로그램을 실행하며, 프로그램 전체를 관리자 권한으로 실행할 필요는 없습니다. 설치된 장치가 `/dev/ttyACM0`라면 `ls -l /dev/ttyACM0`와 `groups`로 접근 권한을 확인하세요. 해당 장치가 `dialout` 그룹 소유인 일반적인 Ubuntu 설정에서는 관리자가 사용자를 추가할 수 있습니다.

```sh
sudo usermod -aG dialout "$USER"
```

그룹 변경은 로그아웃 후 다시 로그인해야 적용됩니다. 실제 장치 이름·그룹과 조직 정책을 먼저 확인하세요. 연결 시험은 프로펠러를 제거하고 비행 명령이 잠긴 상태에서 진행합니다.

## Windows 설치

Windows 11 x64용 NSIS 설치 파일은 아직 생성·검증되지 않았습니다. Windows 빌드 워크플로는 수동 실행용으로 준비되어 있으며 실제 CI 실행은 수행하지 않았습니다. Tauri 설치 설정은 기존 Microsoft Edge WebView2를 재사용하고 없으면 인터넷에서 설치하는 방식을 사용합니다. 오프라인 설치는 별도 설정으로 선택할 수 있습니다. Windows 장치에는 해당 USB-직렬 장치의 공급업체 드라이버가 필요할 수 있습니다. 읽기 전용 COM 열기 구현과 포트 점유·재연결은 실제 Windows 및 해당 드라이버에서 검증해야 합니다. 드라이버가 읽기 전용 열기 또는 설정을 거부하면 연결은 실패하며 쓰기 가능한 핸들로 자동 전환하지 않습니다. [Tauri Windows 설치 문서](https://v2.tauri.app/distribute/windows-installer/)를 참고하세요.

최종 사용자는 앱을 실행하기 위해 Python, Node.js, npm, Rust, Cargo, ROS2, Gazebo, PX4 SITL 또는 QGroundControl을 별도 설치할 필요가 없습니다. 운영체제 런타임, 지도 인터넷 연결 및 장치 드라이버는 필요할 수 있습니다.

## 지도와 로컬 데이터

VWorld 키는 시스템 설정에서 입력·변경합니다. 클라이언트 키는 로컬 파일과 지도 요청에 노출될 수 있으므로 완전한 비밀로 취급할 수 없습니다. 사용 도메인 및 이용 조건은 발급처에서 확인하고 배포 전 키를 교체하세요. 임무와 기록은 키와 분리된 사용자 데이터 디렉터리에 저장됩니다. 비행 기록은 초당 스냅샷이며 모든 원본 MAVLink 프레임이나 짧은 변화를 보존하지 않습니다. 상세 제한은 [Stage 9 보고서](docs/stage9-flight-recording-report.md)를 참고하세요.

## 개발 및 테스트

개발 머신에는 Node.js 22, Rust와 Tauri 플랫폼 의존성이 필요합니다. 저장소 루트에서:

```sh
npm --prefix frontend ci
npm --prefix frontend test
npm --prefix frontend run build
cargo test --offline --manifest-path src-tauri/Cargo.toml
cargo tauri dev
cargo tauri build --bundles deb -- --offline
```

`cargo tauri`는 별도 설치된 Tauri CLI가 필요합니다. 기본 빌드는 개발용 UDP 수신, 텔레메트리 생성기와 3D 실험 feature를 켜지 않습니다. 개발용 시뮬레이션만 별도로 시험할 때:

```sh
cargo tauri dev --features simulation
# 별도 터미널에서, 시뮬레이션 수신 시작 후
cargo run --offline --manifest-path src-tauri/Cargo.toml --features simulation --example telemetry_feed -- battery-low
```

[디자인 시스템](docs/design-system.md)과 [디자인 확정 기록](docs/design-freeze-v1.md)을 유지합니다. `.github/workflows/desktop-build.yml`은 수동 실행용이며 외부 CI 실행이나 원격 push를 수행하지 않았습니다.
