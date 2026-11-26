# 전체 자동 QA 및 E2E 결과

실행일 2026-11-25. 기준 `dev/firescene-gcs` / `4d7c0da`; 앱 Tauri 2.12.7, CLI 2.12.6, tauri-driver 2.1.0, Ubuntu X11, WebKitGTK/WebKitWebDriver 2.52.6. 실제 Tauri WebView를 사용했다. 별도 임시 XDG 설정·데이터·캐시에서 시험했으며 기존 임무와 기록을 열거나 수정하지 않았다. 키는 임시 설정에 0600으로 복사해 쓰고 테스트 종료 시 프로필을 삭제했다. 스크린샷과 상세 JSON은 `/tmp/firescene-desktop-qa-expanded/`와 마지막 재실행 `/tmp/firescene-desktop-qa-final-check/`에만 보관하며 Git에 포함하지 않는다. 설정 화면 캡처는 키 보호를 위해 생략했다. `/tmp` 결과는 재부팅 시 사라질 수 있다.

## 환경 및 실행

Tauri [공식 WebDriver 안내](https://tauri.app/develop/tests/webdriver/manual-setup/)에 따라 Linux의 tauri-driver와 WebKitWebDriver를 선정했다. 둘 다 시스템에 설치하지 않고 `/tmp`에 준비했다. WebDriver 표준 HTTP를 Python 표준 라이브러리에서 직접 사용하므로 WebDriverIO/Selenium 설치는 필요하지 않았다. Playwright 브라우저만으로는 실제 Tauri IPC를 검증할 수 없어 채택하지 않았다. AT-SPI와 xdotool은 보조 수단으로 검토했으나 이번 실행의 입력은 WebDriver 및 합성 DOM 이벤트를 사용했다. WebDriver의 앱 화면 캡처는 사용했으나 스크린샷의 자동 시각 비교는 하지 않았다. 샌드박스에서는 X11과 네트워크가 차단되어 권한을 받아 실행했다.

재실행에는 실행 가능한 X11 세션, `tauri-driver`, 설치된 WebKitGTK와 호환되는 `WebKitWebDriver`, Node/Rust 및 기존 의존성이 필요하다. 도구의 설치 및 연결은 위 공식 문서 참조. 개발용 빌드와 생성기를 준비한 뒤 저장소 루트에서:

```sh
cargo build --offline --features simulation --manifest-path src-tauri/Cargo.toml
cargo build --offline --features simulation --example telemetry_feed --manifest-path src-tauri/Cargo.toml
python3 tests/desktop_qa/run.py --driver /path/to/tauri-driver --native-driver /path/to/WebKitWebDriver --with-map-key --dev --results /tmp/firescene-desktop-qa
```

`--with-map-key`는 현재 사용자의 FireScene 로컬 키를 격리 프로필에 복사한다. 이 옵션이 없으면 실제 검색은 실패로 기록된다. 다른 앱이 포트 1420, 4444 또는 14540을 사용하지 않아야 한다. 자동화는 실제 데스크톱 창을 열고 포커스를 사용할 수 있다. 스크립트는 생성한 프로필만 삭제한다. 실제 기체는 연결하지 않았고 테스트 기체 명령은 송신하지 않았다.

## 실행 결과

PASS는 적힌 범위에만 적용한다. 자동화는 화면의 모든 텍스트·색상·겹침 또는 콘솔 오류를 검사하지 않는다.

| 범위 / 실행 방법 | 기대 및 실제 결과 | 상태 |
|---|---|---|
| 실제 앱 시작 | 탐색 메뉴가 표시됨, 1.36초 | PASS |
| 9개 메뉴 / WebDriver | 각 메뉴의 활성 상태를 순서대로 확인. 설정 제외 8개 화면 캡처 | PASS |
| Dark/Day / WebDriver | 테마 전환 및 복귀, Day 캡처 | PASS |
| 창 크기 1100×760, 최대화, 1200×800 복원 | 요청과 최소 크기 확인. 지도 리사이즈의 픽셀 정확도는 미검증 | PASS (제한적) |
| 전체 화면 배치, 스크롤, 글자 잘림, 버튼 겹침, 콘솔 | 자동 레이아웃 판정과 콘솔 수집 미구현. 캡처는 시각 검토 자료 | NOT TESTED |
| 기본/위성 및 3D 비활성 | 레이어 버튼의 선택 상태와 3D 버튼 비활성 확인. 위성 타일 실제 수신 여부 별도 미검증 | PASS (제한적) |
| 실제 VWorld 장소 검색 | `동의대학교` 20개 표시, 첫 결과 선택 후 이동 안내 표시 | PASS |
| 검색 주소·지번·약칭·공백·오타·없음, 지도 중심의 실제 좌표 변화, 확대/이동, 건물 및 50/100m | 이번 GUI 실행에서는 미실행. 실제 API 원시 검색 결과는 Stage 19.6 보고서 참조 | NOT TESTED |
| 시험 임무 | 격리 데이터에서 작성 중 저장 및 복제 | PASS |
| 위치 선택 후 정식 저장, 수정·검색·보관·삭제·재시작 복구·종합 관제 연동 | 이번 GUI 실행 미실행 | NOT TESTED |
| 오프라인 연습 | ON, 매뉴얼, 합성 W 누름/해제, Escape 종료가 확장 실행에서는 성공했으나 마지막 재실행에서 진입 대기 시간 초과 | FAIL (간헐적, 원인 미확정) |
| 나머지 키·동시/반대 입력·Space·OFF·일반 문자·포커스·최소화·메뉴 이동·시간 초과 | 실제 창/물리 키 이벤트 미검증. 상태 기계 단위 테스트와 구분한다 | NOT TESTED |
| simulation 텔레메트리 | 개발용 생성기 `battery-low`; 시스템 42 HEARTBEAT와 배터리 부족 경고 확인 | PASS |
| GPS/고도/속도/자세 표시, GPS 변화, 오래됨, 연결 손실/복구, 경고 해제, 입력 초기화 | 이번 GUI 종단 간 검증 미실행. Rust 자동 테스트는 별도 통과 | NOT TESTED |
| 기록/재생 | 개발 데이터 기록 시작·종료, 목록, 재생 열기·재생·일시정지·JSONL 격리 내보내기. 속도 선택 이벤트 실행, 실제 속도는 미측정 | PASS (제한적) |
| 기록 재시작 복구, 탐색, 속도 정확도, 지도 경로, 안전 이벤트, 삭제, 실시간 데이터 분리 | 이번 GUI 실행 미실행 | NOT TESTED |
| QGroundControl 경로 | `<HOME>/GroundControl.AppImage` 존재, 실행 권한 있음 | PASS |
| QGroundControl 오류 | 격리 설정에 존재하지 않는 경로를 입력해 실패 안내 확인 | PASS |
| QGroundControl 실제 실행 버튼, 전달 구조/UDP 점유/연결 손실 GUI | 실제 QGroundControl 자동 연결 가능성을 고려해 실행하지 않음. UDP 연결은 Rust 회귀 시험에서 별도 검증 | NOT TESTED |
| 하드웨어 연결 및 비행 | 실제 기체 미연결, 비행 명령 잠금 유지 | NOT TESTED |

## 자동 회귀 및 발견 사항

- 프런트엔드 `npm test`: 13개 파일 PASS; `npm run typecheck`: PASS. 지도 및 임무 단위 테스트 포함.
- Rust `cargo test --offline`: 47 PASS, 벤치마크 1 ignored; simulation: 48 PASS, 벤치마크 1 ignored. 각 Tauri 권한 통합 테스트 1 PASS.
- 첫 기록 재생 자동화는 기록 종료 상태 갱신 및 재생 구간 로딩을 기다리지 않아 실패했다. 캡처에서 기록은 저장됐고 앱이 기록 중 재생을 방지한 것을 확인했다. 시험 대기를 수정해 재실행 PASS. 제품 코드 변경은 없었다.
- 한 동시 실행에서 오프라인 입력과 simulation 항목이 실패했다. 이후 단독 확장 실행에서는 통과했지만 마지막 포트 보호 변경 후 재실행에서 오프라인 진입이 다시 시간 초과됐다. 스크립트가 포커스가 있는 실제 창을 보장하지 못한 가능성이 있으나 원인은 미확정이다. 제품 결함으로 단정하지 않고 FAIL로 보류한다. GUI 시험은 단독 실행을 권장한다.
- 재현된 제품 결함을 확정하지 못했으므로 Critical/High/Medium/Low 제품 수정은 없다. 실제 화면 전체 검증이 완료됐다는 뜻은 아니다.
- 기존 `WebProcess didn't exit as expected after the UI process connection was closed`는 이번에 종료 로그를 수집하지 않아 재현 여부를 판정할 수 없다. 원인과 검색과의 관련성은 미확인이다.

## 성능 표본과 제한

개발 빌드, X11, 실제 지도 키 사용, 각 2초 표본. 아래는 **Tauri 주 프로세스만** `/proc`에서 측정한 CPU(한 코어 기준)와 RSS이며 WebKit 자식 프로세스와 시스템 전체 사용량을 포함하지 않는다. 반복 측정이나 장기 안정성 시험이 아니며 성능 보증이 아니다.

| 조건 | CPU | RSS |
|---|---:|---:|
| 시작 직후 유휴 표본 | 2.5% | 211.7 MiB |
| 지도 검색 및 레이어 전환 후 | 4.0% | 217.8 MiB |
| simulation HEARTBEAT 수신 중 | 3.0% | 218.5 MiB |

기록 재생 중 CPU/RAM, 반복 메뉴 이동 후 메모리 증가, 종료 후 모든 자식 프로세스 잔존 여부는 NOT TESTED다. 드라이버와 생성기는 스크립트가 종료 요청을 수행하지만 전체 프로세스 수명 검증을 대체하지 않는다.

## 하드웨어 전 잔여 항목

위 NOT TESTED 항목 중 특히 안전 경고와 연결 손실·복구, 입력 무효화, 실제 GUI 기록 재생과 실시간 분리, 임무 복구를 완료한다. 이후 프로펠러를 분리하고 [하드웨어 가이드](stage16-hardware-test-guide.md)의 직접 USB와 QGroundControl 전달을 순서대로 읽기 전용으로 확인한다. 실제 비행과 모터 작동은 별도 검증 전 금지한다. 새 DEB는 생성하지 않았다. 로컬 커밋은 이 보고서와 자동화 코드를 포함하는 커밋이며 `git log -1`로 확인한다.

## 후속 QA: 격리 X 화면과 키 입력

2026-11-25 후속 검사에서 공유 X11 화면의 실제 테스트 앱이 입력 도중 `창 포커스 상실` 이벤트를 받은 것을 확인했다. 이것이 모든 이전 간헐 실패의 원인이라고 단정할 수는 없다. 창을 WebDriver 클릭으로 조작하고, 별도 Xvfb 화면에서 테스트 창에 X11 포커스를 준 후 재실행했다. 전체 스크립트는 PASS이며 결과는 `/tmp/firescene-qa-xvfb/results.json`이다. 키 검사는 실제 Tauri WebView에 전달한 합성 DOM 이벤트로 W/A/S/D, 방향키, 키 해제, 상반 W/S, Space, Escape와 2초 입력 시간 초과를 확인했다. 물리 키, 최소화 및 실제 포커스 변경의 종단 간 검증은 아직 남았다. 이 시험은 위 표의 미실행 항목 전체나 비행 안전을 검증하지 않는다.

자동화에 WebDriver 클릭, 선택적 `--xdotool` 창 포커스, 실패 시 포커스 진단, 포트 TIME_WAIT 재사용 처리를 추가했다. Xvfb에서는 창 관리자가 없으므로 최대화는 정상 데스크톱 검증으로 간주하지 않는다. Xvfb와 xdotool은 테스트 전용이며 배포 패키지에 포함하지 않는다.
