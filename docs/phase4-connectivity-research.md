# Phase 4 연결 기술 조사 (진행 중)

확인일 2026-11-26. PX4 v1.16의 CDC ACM autostart는 수신 바이트에 따라 프로토콜을 자동 식별할 수 있으며 SYS_USB_AUTO의 0/1/2는 각각 비활성/자동 감지/MAVLink다. 문서상 기본값 2는 KOA 장치의 실제 설정을 입증하지 않는다. 수신 전용 호환성은 실제 KOA 시험 전 미확정이고 송신은 추가하지 않았다. USB와 TELEM 설정은 서로 다르며 출력 메시지·속도는 설정에 의존한다.

QGroundControl은 장치 링크 및 HEARTBEAT를 바탕으로 기체를 인식하고 이후 파라미터·임무 요청을 한다. FireScene은 그 요청을 보내지 않으며 하드웨어 인식과 검증된 HEARTBEAT를 구분한다. HEARTBEAT와 system/component ID는 식별·상태에 쓰이나 패킷 인증을 제공하지 않는다. 신호 강도와 지연은 프레임 수신만으로 계산하지 않는다.

출처: https://docs.px4.io/v1.16/en/modules/modules_system ; https://docs.px4.io/v1.16/en/advanced_config/parameter_reference ; https://docs.px4.io/v1.16/en/peripherals/serial_configuration ; https://mavlink.io/en/services/heartbeat.html ; https://mavlink.io/en/guide/serialization.html ; https://github.com/mavlink/qgroundcontrol/blob/master/docs/en/qgc-dev-guide/communication_flow.md .

DJI Pilot 2·FlightHub 2와 기타 제품의 상세 UX 비교 및 별도 메시지·손실률 조사는 미완료다. 위 문서는 FC 실패 원인 확인이나 제품 간 성능 비교를 뜻하지 않는다.
