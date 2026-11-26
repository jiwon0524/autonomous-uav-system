# Phase 4 연결 진단 (진행 중)

포트 열기와 설정 결과, OS 코드, 수신 바이트, 유효 프레임, 보고된 파싱 오류, HEARTBEAT와 거부 사유 및 직전 실패를 보존한다. 열기와 직렬 설정의 실패는 아직 별도 단계로 분리되지 않는다. 파싱 오류는 라이브러리가 보고한 수치로 모든 손상 프레임을 의미하지 않는다.

추가된 최근 프레임/초는 해당 링크의 CRC 유효 프레임을 약 1초 간격으로 센 값이다. 기체 식별 전 또는 다른 시스템·컴포넌트의 유효 프레임도 포함하므로 특정 기체의 메시지율이나 패킷 손실률이 아니다. HEARTBEAT 간격은 선택된 동일 기체의 연속 유효 HEARTBEAT 사이의 단조 시계 경과 시간이다. 재연결 시 두 수치는 초기화되고 종료·오래된 표본은 화면에서 확인 불가로 표시한다. 둘 다 무선 RSSI와 왕복 지연이 아니다. 진단 복사에는 수집한 수치가 포함되며 장치 일련번호와 원시 패킷은 포함하지 않는다.

## 후속 오류 단계

진단은 `port_opened`와 `serial_configured`를 분리한다. `PORT_OPEN`, `SERIAL_CONFIGURATION`, `READ`, `RECEIVE_TIMEOUT`, `HEARTBEAT_TIMEOUT`, `HEARTBEAT_LOST`는 실제 실패 지점을 기록하며 가능한 경우 원래 OS 오류 번호를 함께 보존한다. Linux의 잠금 및 ioctl 실패는 핸들 취득 이후 설정 단계에 포함된다. Windows는 CreateFileW 성공 이후 GetCommState/BuildCommDCB/SetCommState/SetCommTimeouts 실패를 설정 단계로 기록한다. Windows 접근 거부는 점유와 권한을 구별할 근거가 없으므로 단정하지 않는다. 장치 목록 검증 실패는 연결 워커 시작 전 반환된다. UI 수신율은 다른 컴포넌트를 포함할 수 있으며 HEARTBEAT 간격은 지연이 아니다. OS별 실제 장치 오류 및 Windows 클립보드는 미검증이다.
