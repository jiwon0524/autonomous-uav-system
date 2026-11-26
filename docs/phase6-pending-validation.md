# Phase 4에서 Phase 6으로 이관한 검증

## 2026-11-26 Phase 6 추가 증거

- 후속 제한 내구 시험: 실제 개발 Tauri 가상 세션 870회, 618.8초, 4,640건에 대해 세션별 정상 종료·명시 잠금·watchdog 만료의 예상 순서와 개수 및 전체 건수를 대조해 PASS (`~/FireScene-evidence/phase6/safety/firescene-phase6-journal-10min/`). 명시 잠금과 watchdog이 각각 같은 `safety_lock` 코드를 남기는 두 기록은 기존 동작으로 예상에 포함했다. 무제한 기간의 누락·중복이 없다는 보장은 아니다. 0.1.1 개발 Tauri의 `/dev/full` 쓰기 실패 시 기존 토큰 거부·중립/권한 회수·신규 시작 거부도 PASS (`firescene-phase6-journal-error-011/`).
- 합성 UDP의 연결 종료 후 기존 텔레메트리 제거, 이전 포트 및 다른 system ID의 자세 데이터 무시, 새 연결의 실제 0 수용 단위 시험 PASS (`udp_new_session_discards_old_telemetry_and_rejects_other_system`). MAVLink 패킷 자체에 애플리케이션 세션 ID가 없으므로 같은 식별자로 새 포트에 도착하는 패킷의 과거 세션 여부를 구분한다는 뜻은 아니다. 실제 FC는 별도 NOT TESTED다.

- 실제 개발 Tauri 가상 세션에서 합성 키 이벤트로 전후·좌우 동시 입력, 부분 해제, 전체 해제와 백엔드 중립 속도, 종료 후 기존 토큰 입력 거부 PASS (`~/FireScene-evidence/phase6/safety/firescene-phase6-opposed-virtual/`). 물리 키보드 시험은 아니다.
- 실제 개발 Tauri에서 정상 종료·명시 잠금·watchdog 만료를 각각 30회, 총 90세션·63.9초, 480개 이벤트의 세션 ID별 순서·개수와 전체 합계를 검사했다 (`firescene-phase6-journal-endurance/`). 명시 잠금 뒤 watchdog 잠금 기록 두 개는 서로 다른 경로에서 의도적으로 생성된 기존 이벤트이며 예상에 포함했다. 기록 오류·시작 거부는 기존 Phase 5 실제 오류 시험을 참조한다. 63.9초는 유한 시험으로 더 장시간의 누락·중복 전수를 보장하지 않으므로 이벤트 기준은 PARTIAL 유지.
- 자세 허용 경계 양쪽과 경계 초과, 센서 health 실제 0·배터리 0, 센서만 만료된 조합의 추가 단위 시험 PASS. 내부 자세 변환 단위 시험이지 모든 메시지 조합이나 실제 데이터 시험은 아니다. 다른 기체 차단은 기존 `udp_simulation_receive_is_loopback_and_exclusive` 합성 UDP 시험 근거. 이전 연결의 모든 지연 데이터 조합은 계속 미완료다.

2026-11-26 사용자 지시에 따라 Phase 5를 시작한다. Phase 4 완료를 뜻하지 않으며 기존 18개 판정은 `phase4-validation-report.md` 그대로 보존한다.

- MAVLink 메시지별 유효성·범위·미상·0·비유한 조합 전수 및 지원하지 않는 필드 확인.
- 센서별 TTL, 다른 기체 및 연결 세션 조합 전수.
- 기존 합성 GUI 증거 당시 코드와 최신 코드의 정확한 일치 및 변경 영향 대조.
- Windows 설치 및 GUI와 Windows 클립보드.
- 물리 COM 연결 및 오류·자동 복구, KOA FC HEARTBEAT 미수신 원인.
- 실제 텔레메트리, 연결 손실 및 복구.
- STATUSTEXT 다중 청크 재조립의 현재 제한 및 필요성 확인.
- 생산 JavaScript 청크 크기 경고 및 성능 영향.

Phase 5의 안전 판단에 직접 사용하는 범위는 사용 전 별도 검증하며, 이관 자체가 검증을 대체하지 않는다. 실환경 항목은 실제 수행 전 NOT TESTED다.

외부 검증 목록에 안전하게 네트워크 격리된 PX4 SITL, 실제 Windows 설치·GUI 및 물리 키보드를 유지한다. 물리 COM, KOA FC HEARTBEAT, 실제 텔레메트리 및 하드웨어 연결 복구도 미검증이다. Phase 5의 생산 IPC 차단 및 안전 검증 공백은 Phase 6으로 이관해 완료 처리하지 않는다.

2026-11-26 잔여 안전 시험 이후에도 안전하게 격리된 PX4 SITL, Windows 설치·GUI·물리 키보드, 물리 COM, KOA FC HEARTBEAT, 실제 텔레메트리 및 하드웨어 연결 복구는 외부 환경에서 NOT TESTED다. 다른 창 ACL·위험 복구·기록 경합과 GUI 회귀는 Phase 5 내 미완료로 유지한다.

2026-11-26 최종 소프트웨어 평가에서 다른 Tauri 창 ACL, 최소화·복원, 5개 위험 복구, 기록 포화·종료 경합과 전체 GUI 46개는 통과했다. 이전 절의 Phase 5 내 해당 미검증 표기는 이 평가로 대체한다. 격리 PX4 SITL, 실제 Windows 설치·GUI·물리 키보드, 물리 COM, KOA FC HEARTBEAT, 실제 텔레메트리 및 하드웨어 연결 복구는 계속 NOT TESTED다. 가상 세션 반대 키 동시 입력 종단 간 및 장시간 이벤트 순서·중복/누락 확인은 비핵심 소프트웨어 후속 검증으로 남는다.
