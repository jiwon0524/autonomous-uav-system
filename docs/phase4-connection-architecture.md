# Phase 4 연결 구조 (진행 중)

Windows COM 및 Linux USB 직렬은 읽기 전용·독점, UDP는 localhost 수신만 허용한다. QGC/Router 전달은 사용자 선택의 미인증 출처이며 QGC는 필수가 아니다. simulation은 feature 및 개발 UI로 한정한다. 물리 텔레메트리 장치의 OS 열거와 KOA 호환성은 별도 시험이 필요하다.

포트 감지 → 열기/설정 → HEARTBEAT 대기 → CRC 유효 PX4 기체 확인 → 연결로 전이하며 오류·손실은 독립 상태다. 재연결 시 세션 store가 초기화되고 직전 실패 진단은 별도 보관된다. 사용자가 연결을 해제하면 자동 연결 허용이 해제되며 자동 연결은 승인된 고유 식별자에 한정해 최대 3회다. UDP와 직렬 모두 송신 경로를 추가하지 않았다. 재시도는 UI 정책이며 무선 연결 품질을 의미하지 않는다.

## 2026-11-26 후속 구현

USB 열거는 DISCONNECTED/PORT_DETECTED만 갱신하며 연결을 증명하지 않는다. CONNECTING에서 읽기 전용 OS 핸들 취득 뒤 설정을 수행하고 성공 시 WAITING_HEARTBEAT로 진입한다. 바이트와 CRC 유효 프레임은 진단 카운터로 구분하며, 첫 유효 PX4 HEARTBEAT만 CONNECTED를 만든다. 설정 완료 후 최초 10초 이내 유효 HEARTBEAT가 없으면 ERROR, 연결 후 5초 끊기거나 읽기 실패하면 CONNECTION_LOST이며 기체와 저장소를 비운다. 사용자의 해제는 워커를 취소·join하고 DISCONNECTED로 초기화한다. 새 워커 시작 전 이전 워커를 join하므로 이전 수신 스레드가 새 세션을 갱신하지 않는다. 자동 연결은 프런트엔드 정책으로 고유 허용 장치에 최대 3회이며 수동 해제 시 허용을 제거한다. 재연결은 별도 phase가 아니라 새 CONNECTING이다. 데이터 수신 대기와 HEARTBEAT 대기는 동일 phase이며 진단 바이트·프레임 수로 구분한다. 비동기 UI 응답 경합 및 실제 GUI 자동 재연결 시나리오는 추가 검증이 필요하다.
