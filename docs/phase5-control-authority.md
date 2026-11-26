# 조종 권한 (설계 중)

실제 권한은 항상 잠김이며 읽기 전용 연결로 조종 권한이 생기지 않는다. 향후 개발 전용 가상 상태는 OFFLINE, SIMULATION READY, SIMULATION CONTROL, CONTROL LOCKED, CONNECTION LOST, SAFETY BLOCKED의 명시적 전이와 세션별 초기화를 갖는다. READ ONLY는 실제 연결 관찰 상태다. 재생은 항상 잠김이다. 현재 이 상태 관리자는 아직 구현되지 않았다. 제어권 없음의 이유와 원인은 사용자에게 드러내고, 프런트엔드 버튼 비활성화만을 보호 수단으로 취급하지 않는다.

## 2026-11-26 구현 현황

개발 simulation에서만 세션별 가상 권한과 토큰을 사용하며 연결·재생과 상호 배제한다. 500ms 입력 임대 만료 시 백엔드 감시가 입력을 중립화하고 해당 토큰의 권한을 회수한다. 실기체 제어 권한은 계속 잠겨 있다. 세부 검증 한계는 `phase5-validation-report.md`를 따른다.
