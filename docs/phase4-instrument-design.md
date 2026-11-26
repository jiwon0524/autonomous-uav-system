# Phase 4 비행 계기 (기존 구현 검토)

기존 인공수평계는 유효한 최신 Roll/Pitch만 표시한다. 비행 화면은 방향, 상대 고도 및 해발 고도, 지상/대기 속도, 상승 속도, 배터리·전압·전류, ARM·모드 및 GPS 정보를 별도 필드 최신성에 따라 표시한다. 미수신·오래됨은 숫자 0으로 표시하지 않는다. PX4 failsafe는 독립적이며 FireScene 안전 상태는 보조 정보다. 이 단계에서 새로운 실제 비행 명령은 추가하지 않았다. 계기 전면 재설계 및 실기체 검증은 미완료다.

2026-11-26: 기존 계기는 LIVE 자세가 없으면 수평계를 숨기고 미수신·오래됨을 표시한다. 실제 GUI 합성 계기 증거는 이전 코드의 `~/FireScene-evidence/phase4/flight-instruments-simulation.png`이며 이번 변경 후 전체 입력값·지연·재연결·다른 기체별 시각 비교를 대체하지 않는다.
최종 코드 재시험의 합성 인공수평계 캡처는 `~/FireScene-evidence/phase4/finalization/firescene-phase4-final-retest/flight-instruments-simulation.png`이다. 입력값별 수치·경계·지연·복구 전수 검증은 아니다.
