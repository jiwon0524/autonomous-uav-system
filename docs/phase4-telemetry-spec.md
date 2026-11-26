# Phase 4 텔레메트리 매핑 (진행 중)

수신은 CRC 검증 후 선택된 기체의 system/component에 한정하고, 각 필드는 개별 TTL 및 LIVE/STALE/INVALID를 가진다. 다음은 현재 코드 기준이며 송신 빈도를 보장하지 않는다.

| 메시지 | 주 데이터 / 변환 | TTL |
|---|---|---|
| HEARTBEAT | ARM, PX4 custom mode, MAV 상태 | 5초 |
| SYS_STATUS | 센서 비트, 잔량 %, mV→V, cA→A; 센티널 무효 | 10초 |
| BATTERY_STATUS id 0 | 잔량·전류, 셀 합산 전압 미추정 | 10초 |
| GPS_RAW_INT | fix·위성, 1e-7도 위치; fix 및 좌표 검증 | 5초 |
| GLOBAL_POSITION_INT | mm→m 상대/해발, cm/s→m/s, NED 수직속도 부호 반전, centidegree→degree | 3초 |
| ATTITUDE / ATTITUDE_QUATERNION | rad→degree, quaternion 정규화 검사 | 2초 |
| VFR_HUD | m/s, 해발 m, 상승 m/s, heading degree; 비유한 값 무효 | 3초 |
| LOCAL_POSITION_NED | 북·동·하 m 및 속도 m/s, 상승 부호 변환 | 3초 |
| EXTENDED_SYS_STATE | 착륙 상태, 미정은 무효 | 5초 |
| STATUSTEXT | 심각도와 수신 문구 | 30초 |

미수신과 무효는 0으로 대체하지 않는다. 각 메시지별 상세 유효 범위 및 화면·테스트 전수 표는 추가 작업이다. 데이터는 미인증이며 수신 전용이므로 펌웨어 버전 등 요청이 필요한 정보를 확보한다고 가정하지 않는다.

## 구현 범위와 검증 한계

기준은 [MAVLink common 정의](https://mavlink.io/en/messages/common.html), [PX4 v1.16 시스템 모듈](https://docs.px4.io/v1.16/en/modules/modules_system)이며 빌드의 `mavlink` 0.21.0 common dialect를 사용한다. 수신 메시지 송출 여부나 빈도는 보장하지 않는다. 동일 세션의 첫 유효 기체 system/component와 일치하는 메시지만 처리한다. 저장 필드는 값·단위·수신시각·메시지 출처·system/component·TTL 상태를 갖고 세션은 연결 저장소 초기화와 안전 모니터로 구분한다. 재생은 별도 화면 데이터다.

GPS_RAW_INT 위치는 fix ≥2 및 위도 ±90°, 경도 ±180°에서만 유효하며 위성 255는 미상이다. GLOBAL_POSITION_INT의 고도와 NED 속도는 위 표와 같고 이 구현은 해당 메시지의 좌표를 지도에 사용하지 않는다. SYS_STATUS의 65535 mV 및 -1 cA/잔량은 미상, 잔량은 0–100만 허용한다. BATTERY_STATUS는 id 0의 잔량과 전류만 처리하고 다른 배터리와 셀 전압 합산은 지원하지 않는다. ATTITUDE는 유한한 각도와 roll/yaw ±(π+0.001), pitch ±(π/2+0.001) 범위를 검사해 도로 변환한다. VFR_HUD는 유한값, 속도 ≥0 및 heading 0–360을 검사한다. LOCAL_POSITION_NED는 북·동·하 축이며 비유한 값은 무효다. EXTENDED_SYS_STATE와 STATUSTEXT는 수신한 착륙 상태와 문구를 제공한다. 지도는 GPS 필드를, 비행 계기는 자세·고도·속도·방향·배터리를, 안전 감시는 수신 필드와 HEARTBEAT를 사용한다. 상세 메시지별 독립 경계값·GUI 시험 전수는 미완료다. Rust `telemetry::tests`는 단위·센티널·최신성, GPS, quaternion, pitch, NED 및 heading 일부를 시험한다.

## 최종 지원 범위 대조

공식 [MAVLink common 메시지 정의](https://mavlink.io/en/messages/common.html)와 `src-tauri/src/telemetry.rs`를 대조했다. 다음의 범위는 프로토콜 전체의 한계가 아니라 이 앱의 검증·표시 정책이다. 정수 필드에는 NaN/무한대 표현이 없으며 해당 시험은 부동소수 필드에만 적용한다. 아직 수신되지 않은 필드는 맵에 없고, 수신된 미상·범위 밖 값은 INVALID, 유효값은 TTL 초과 시 STALE이다. 최근 수신한 메시지가 동일 필드를 덮어쓸 수 있다. 기체 선택 및 세션 분리는 Store 바깥의 연결 계층이 담당한다.

| 메시지 | 지원 및 표시 / 제한 | 검증 근거 및 미검증 |
|---|---|---|
| HEARTBEAT | ARM, PX4 custom mode, MAV 상태; 5초. 알 수 없는 모드 표시 또는 INVALID | ARM·모드·TTL 단위 시험, 연결 기체 선택 및 세션 시험. 모든 펌웨어별 모드는 미지원 |
| ATTITUDE | roll/yaw ±(π+0.001), pitch ±(π/2+0.001), rad→°; 2초; 자세계 | 0, 범위 초과, NaN/무한대 및 TTL 시험. 범위 양 끝 전수는 미시험 |
| GLOBAL_POSITION_INT | 상대·해발 mm→m, 수평·수직 cm/s→m/s, heading centidegree; 3초 | 속도·부호·heading 경계와 미상 시험. 좌표는 사용하지 않음 |
| GPS_RAW_INT | fix, 위성, 위경도 1e-7°; 5초; 지도·상태 | 좌표 경계·범위 초과, fix 미상/불충분, 위성 0/255 시험 |
| SYS_STATUS | 센서 비트, 잔량 0–100%, mV→V, cA→A; 10초 | 0·잔량 경계/초과·센티널 및 센서 비트 시험 |
| BATTERY_STATUS | id 0 잔량·전류만; 10초. 셀 전압 및 다른 id 미지원 | 0·100·미상·다른 id 시험. 전압은 SYS_STATUS에서만 표시 |
| VFR_HUD | 대기·지상 속도, 해발 고도, 상승 속도, heading; 3초 | 음수 속도, NaN/무한대, heading 0/359/360/361 및 미상 시험 |
| LOCAL_POSITION_NED | N/E/D m, N/E 및 상승 m/s; 3초. 독립 비행 계기 표시는 없음 | 0, 제한 경계/초과, NaN/무한대, 상승 부호 및 TTL 단위 시험 |
| EXTENDED_SYS_STATE | 착륙 상태만; 5초, 비행 계기 표시. VTOL 상태 미지원 | 미상, 지상 및 TTL 단위 시험 |
| STATUSTEXT | 심각도·텍스트; 30초, 이벤트·안전 처리. 여러 청크 재조립 미지원 | 심각도 및 텍스트 단위 시험. 긴 메시지의 청크 재조립 미검증 |

고도·로컬 위치는 절댓값 100,000 m 이하, 속도는 1,000 m/s 이하의 앱 제한이 별도로 적용된다. 각 메시지의 모든 필드 조합과 모든 경계의 GUI 출력이 검증된 것은 아니다.
