# Stage 21 Windows 독립 실행 준비 보고서

기준: `design-v2-experimental` / 15f5f4a. Windows 네이티브 러너에서 빌드와 설치를 **아직 실행하지 않았다**. KOA FC의 펌웨어·드라이버와 실제 MAVLink 호환성도 미확인이다. 따라서 현재 Windows 11 배포 준비를 완료했다고 판단하거나 실제 기체 연결 성공을 주장할 수 없다.

## Windows 호환성 분석과 변경

`serialport::available_ports`의 USB 포트 목록을 사용하며 VID/PID·일련번호를 표시한다. 목록에 나타나는 장치와 MAVLink 통신 성공은 다르다. Windows는 `COM` 번호를 검증하고 `CreateFileW`를 `GENERIC_READ`, 공유 없음으로 열며 속도와 8N1, 읽기 타임아웃을 설정한다. 자동 연결은 고유 일련번호를 사용자가 명시적으로 허용한 경우로 제한한다. 유효한 PX4 HEARTBEAT 수신 후에만 연결된 기체를 표시하고, 수신 시간 초과 및 연결 손실 시 기체 식별을 해제한다. 현재 코드는 PX4 autopilot 타입의 유효한 HEARTBEAT만 인정한다. KOA FC가 이 조건과 케이블·드라이버·baud를 만족하는지 Windows에서 확인해야 한다. USB 종류로 식별되지 않는 COM 장치는 기존 목록에서 제외될 수 있다.

기존 임무 파일 쓰기에는 Unix 프로세스 간 잠금만 있었다. Windows에서도 `.missions.lock`을 공유 불가로 열어 동시 쓰기를 막도록 보완하고 Windows 전용 경합 테스트를 추가했다. 해당 테스트는 Windows에서 실행해야 한다. Windows 접근 거부는 권한 부족과 포트 점유를 구분하기 어려워 두 가능성을 함께 안내한다. USB 감지, 포트 점유 및 케이블·드라이버 확인 안내를 연결 화면에 추가했다. Linux 전용 시스템 호출은 조건부 컴파일되어 있다. 이 분석은 실제 Windows COM 핸들 또는 FC 통신 시험을 대체하지 않는다.

변경 파일: `src-tauri/src/missions.rs`, `connection.rs`, `src-tauri/tauri.conf.json`, `frontend/src/Connection.tsx`, 두 Windows 워크플로, 본 보고서와 [Windows 사용자 가이드](windows-hardware-test-guide.md).

## 설치와 실행

Tauri 2 NSIS, Windows x64, 현재 사용자 설치, 한글/영어, 앱 아이콘 및 시작 메뉴 폴더를 설정했다. Tauri 기본 NSIS는 완료 화면의 바탕화면 바로가기 선택과 제거 기능을 제공한다. Stage 22에서 기본 WebView2 모드를 `downloadBootstrapper`로 변경했다. 기존 런타임은 재사용하고 없으면 인터넷에서 설치한다. 오프라인 선택지는 `src-tauri/offline-webview.conf.json`으로 별도 보존한다. [Tauri 공식 문서](https://v2.tauri.app/distribute/windows-installer/)에 따르면 오프라인 방식은 설치 파일에 약 127 MB를 추가할 수 있다. 실제 크기·동작은 빌드 후 측정한다. 서명 인증서는 설정하지 않았으므로 결과는 서명되지 않은 테스트 빌드다. KOA FC 드라이버나 QGroundControl, ROS2, Gazebo, SITL을 번들로 추가하지 않았다.

임무와 비행 기록/내보내기는 Tauri `app_data_dir`, 지도 키는 `app_config_dir`, 자동 연결 및 기타 UI 설정은 웹뷰 로컬 저장소를 이용한다. 고정된 개발 PC 경로를 저장 경로로 사용하지 않는다. Windows에서의 실제 경로, 업데이트·제거 후 데이터 보존은 미검증이다. 지도 키와 네트워크가 없어도 시리얼 연결 코드는 지도와 독립되어 있으나 Windows GUI에서 오프라인 통합 확인은 남았다.

## 자동 빌드와 공개 분리

[Windows 테스트 Artifact 워크플로](../.github/workflows/windows-build.yml)는 수동 실행 시 Windows 2022 x64에서 Node 22/Rust, `npm ci`, TypeScript/프런트엔드 테스트, Windows Rust 테스트와 NSIS 빌드를 수행하고, 목표 이름의 설치 파일·SHA256·빌드 커밋을 Artifact로 업로드한다. 기본 Cargo features를 사용하며 simulation feature를 켜지 않는다. [별도 초안 Release 워크플로](../.github/workflows/windows-draft-release.yml)는 수동 실행만 지원한다. 성공한 지정 빌드의 Artifact, 체크섬, 기존 태그와 빌드 커밋을 대조한 뒤 **사전 릴리스 초안만** 만든다. 공개는 별도의 사용자 승인과 검증 후 GitHub에서 수행해야 한다. 공식 [Tauri GitHub Actions 안내](https://v2.tauri.app/distribute/pipelines/github/)와 [tauri-action](https://github.com/tauri-apps/tauri-action)을 확인했다. 이 프로젝트는 기존에 고정한 Tauri CLI를 직접 호출하며 tauri-action은 필수가 아니다.

GitHub 저장소 연결 계정은 admin/push 권한을 보였고 Actions는 활성화, 허용 동작은 all, 기본 workflow token 권한은 read로 조회됐다. 실제 workflow job의 write 허용 여부와 보호 정책은 원격 실행에서 확인해야 한다. 이 작업에서는 원격 push, Actions 실행, 태그 생성 및 Release 생성을 하지 않았다. `workflow_dispatch`를 사용하려면 워크플로 파일이 GitHub 기본 브랜치에 존재해야 한다. 현재 기본 브랜치에서는 기존 `desktop-build.yml`도 조회되지 않았으므로 실험 브랜치만 push해서 새 수동 워크플로를 실행할 수 없다. 사용자의 별도 승인 후 검토한 변경을 원격에 반영하고, Actions에서 `Windows NSIS test artifact`를 원하는 커밋이 포함된 브랜치로 수동 실행한다. 성공한 run의 Artifact와 체크섬·서명 상태를 검토하고 Windows 11 설치 및 KOA FC 읽기 전용 시험을 완료한다. Release 공개는 다시 별도 승인한다.

## 검증 및 보안 한계

- Ubuntu 로컬 프런트엔드 테스트 13개 파일, TypeScript, 프런트엔드 빌드 PASS. Rust 기본 47개와 simulation 48개 PASS, 각각 1개 벤치마크 ignored; Tauri 권한 통합 테스트 각각 1개 PASS. Rust 직렬 관련 시험 대부분은 Linux 전용이다.
- Windows 네이티브 빌드, NSIS 생성, 설치·제거, 바로가기, WebView2 처리, COM 포트 및 Windows 잠금 테스트는 **NOT TESTED**. 설치 파일 크기와 SHA256은 생성 후 확인한다. Ubuntu에서 반복 크로스 컴파일하지 않았다.
- 기본 features에 simulation이 없고 개발 생성기는 simulation feature가 필요한 example이다. 프로덕션 Tauri handler는 simulation 연결 명령을 등록하지 않는다. 설정에 추가 번들 리소스/외부 프로그램이 없다. 코드 검토상 실제 비행 명령 송신 경로는 없고 시리얼 핸들은 읽기 전용이다. 이는 Windows 빌드 파일 내부 검사나 기체 시험이 아니다.
- CI는 깨끗한 체크아웃을 빌드하며 로컬 API 키를 전달하지 않는다. 로컬 키의 정확한 바이트를 추적 파일 및 이번에 추가한 문서/워크플로와 비교한 결과 일치 항목은 없었다. 이는 다른 비밀의 부재를 증명하지 않는다. 실제 설치 파일을 생성하지 못해 그 내부의 키·시험 자료·시뮬레이션 배제 여부는 검사하지 못했다. Artifact 생성 후 바이너리와 설치 내용 및 개인 정보 노출을 별도로 점검해야 한다.
- 실행 중인 Windows 기체 연결, 실데이터 표시, 장치 관리자 드라이버, 연결 끊김·재연결, 데이터 보존 및 성능은 [하드웨어 가이드](windows-hardware-test-guide.md)에 따라 별도 시험한다. 이전 GUI QA의 미검증 항목은 유지한다.

로컬 커밋은 이 보고서를 포함하는 커밋으로 `git log -1`에서 확인한다.
