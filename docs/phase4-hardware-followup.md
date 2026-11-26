# Phase 4 하드웨어 후속 검증

Windows 진단 설치 파일을 보존하고 실제 KOA FC와 동일 USB 데이터 케이블로 시험한다. 프로펠러를 제거하고 장치 관리자 COM, 포트 열기/설정, 바이트·프레임·HEARTBEAT, 오류 코드 및 복사 진단을 기록한다. QGroundControl과 FireScene은 같은 포트를 동시에 열지 않는다. FireScene을 종료하고 QGC의 기체 인식·실제 자세를 기록한 뒤 QGC를 완전히 종료하고 FireScene을 시험한다. 펌웨어나 설정을 임의 변경하지 않는다. 기존 절차: `windows-hardware-test-guide.md`, `phase3-windows-followup.md`. 실제 결과가 없으므로 원인은 미확정이다.

후속 단계에서 Windows COM의 존재하지 않음·점유·권한·설정·제거 오류와 OS 코드, 수동 해제 후 자동 연결 억제, 재삽입 자동 복구, 클립보드를 각각 기록한다. Ubuntu 물리 USB에서도 동일하게 확인한다. PX4 USB 자동 감지 가능성은 가설이며 FC 파라미터 및 USB 상태 확인 전 원인으로 확정하지 않는다. QGroundControl은 선택적 비교 도구다.
