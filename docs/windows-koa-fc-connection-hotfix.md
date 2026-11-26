# Windows KOA FC 연결 조사 (진행 중)

확인일: 2026-11-26. Preview 1에서 COM4와 USB 식별자가 보였으나 PX4 HEARTBEAT는 확인되지 않았다. 열거는 포트 열기나 통신 성공을 증명하지 않는다. 제공된 결과만으로 원인을 확정할 수 없다. QGroundControl 비교 결과도 아직 없다.

현재 Preview 1 커밋 `0f1b1f5`와 `2da9ad4` 사이에 `src-tauri/src/connection.rs` 변경은 없다. USB 포트 열거는 serialport, Windows 열기는 읽기 전용 독점 CreateFileW, 8N1 설정과 비동기적 폴링에 해당하는 즉시 반환 시간 제한을 사용한다. 수신기는 CRC 확인된 MAVLink에서 유효한 PX4 HEARTBEAT만 연결로 표시하며 동일 시스템/컴포넌트의 데이터만 사용한다. 송신 코드는 없다. 자동 연결은 고유 USB 식별자에 대한 명시적 허용 및 최대 3회 재시도이며 수동 해제는 자동 연결을 끈다.

이번 진행분은 포트 열기 성공, OS 오류 코드, 바이트·유효 프레임·파싱 오류·HEARTBEAT 수 및 바이트 수신 시각을 세션별로 수집한다. 프레임 파서는 동기화 과정에서 버린 모든 잡음 바이트를 파싱 오류로 노출하지 않을 수 있다. 진단 복사에는 일련번호와 원시 패킷을 포함하지 않는다. 연결 시 장치 식별 정보 보존, 단계별 상세 진단과 Windows 클립보드 검증은 아직 필요하다. 빌드 커밋 및 직전 실패 진단은 아래 추가 분석을 참고한다.

QGroundControl 공식 설명에 따르면 장치 링크와 HEARTBEAT로 기체 객체를 생성하고 이후 파라미터와 임무를 요청한다. QGC는 GCS HEARTBEAT도 송신한다. 이는 FireScene과의 차이지만 이번 장치에서 송신이 필요한지 입증하지 않는다. 따라서 메시지를 새로 송신하거나 검증 조건을 완화하지 않았다. 포트 점유, 권한, DTR/RTS, 부트로더, USB CDC 설정, 펌웨어 및 수신 전용 호환성은 모두 미확인이다.

출처 (2026-11-26 확인):
- QGC 통신 흐름: https://github.com/mavlink/qgroundcontrol/blob/master/docs/en/qgc-dev-guide/communication_flow.md
- QGC HEARTBEAT 송신: https://github.com/mavlink/qgroundcontrol/blob/master/src/Vehicle/MultiVehicleManager.cc
- Windows 직렬 포트 독점 열기: https://learn.microsoft.com/en-us/windows/win32/devio/communications-resource-handles
- Windows USB CDC 드라이버: https://learn.microsoft.com/en-us/windows-hardware/drivers/usbcon/usb-driver-installation-based-on-compatible-ids
- QGC 연결 진단: https://github.com/mavlink/qgroundcontrol/blob/master/docs/en/qgc-user-guide/troubleshooting/vehicle_connection.md

이 문서는 전체 공식 자료 조사나 Windows 네이티브 검증을 완료했다는 의미가 아니다. 하드웨어 원인 규명 및 배포용 수정판은 미완료다.

## 2026-11-26 추가 분석 및 진단 변경

Windows는 COM 이름을 검증해 `\\.\COMn`으로 변환하고 `GENERIC_READ`, 공유 모드 0, `OPEN_EXISTING`으로 연다. 핸들 생성 후 GetCommState, BuildCommDCBW, SetCommState로 8N1을 설정하며 COMMTIMEOUTS의 interval MAX / total 0 조합으로 즉시 읽고 0바이트를 WouldBlock으로 다룬다. 열기뿐 아니라 설정까지 끝나야 `port_opened`가 참이다. 실패 시 Windows 오류 코드가 있으면 보존한다. 읽기 오류·타임아웃은 별도 메시지다. DTR/RTS를 명시적으로 설정하지 않으며 이전 DCB 설정의 영향을 받을 수 있다. 이는 호환성 조사의 대상이지 이번 고장의 확인된 원인이 아니다. Microsoft는 직렬 설정이 이전 열린 설정에서 이어질 수 있다고 설명한다: https://learn.microsoft.com/en-us/windows/win32/devio/modification-of-communications-resource-settings ; 시간 제한: https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-commtimeouts .

사용 중인 serialport 4.10.1 Windows 소스는 COM 경로를 같은 방식으로 확장하지만 GENERIC_READ | GENERIC_WRITE로 독점 열고 DCB를 초기화하며 DTR 옵션이 설정된 경우 변경한다: https://docs.rs/crate/serialport/4.10.1/source/src/windows/com.rs . 이 구현으로 교체하면 읽기 전용 핸들 보장이 바뀌므로 교체하지 않았다. QGC 초기 흐름과 GCS 송신은 위 자료를 참고하며, 수신 전용이 이 FC에서 실패하는지는 미확인이다.

MAVLink 1/2는 라이브러리의 CRC 검증 raw reader를 사용한다. CRC는 인증이 아니며 서명된 프레임은 기존 정책대로 거부한다. 상세 규격: https://mavlink.io/en/guide/serialization.html . HEARTBEAT에서 ID, 플래그, 길이, autopilot, 기체 유형, 버전 거부 사유를 기록한다. 이전 세션의 실패 진단은 새 카운터와 별도로 보관하며 앱 종료 후 영구 보관하지 않는다. 파싱 오류 카운터는 라이브러리가 호출자에게 반환한 오류만 센다. 잡음 또는 내부 재동기화 과정에서 버려진 모든 CRC 오류의 총수가 아니다. 장치 검색·클립보드와 Windows 네이티브 COM은 실기기 검증이 필요하다.

GUI 복사는 클립보드 API와 대체 복사 방식을 사용하나 Windows WebView2에서 아직 검증하지 않았다. 앱 빌드 커밋은 GitHub Actions의 GITHUB_SHA를 포함하고 로컬 빌드는 local-unverified로 표시한다. 진단 복사에 장치 일련번호와 원시 데이터를 넣지 않는다. 선택된 포트의 VID/PID는 현재 목록에 있을 때만 포함된다.
