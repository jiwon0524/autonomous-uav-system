# Phase 5 안전 조사

확인일 2026-11-26. 각 링크는 공식 문서이며 제품·버전별 차이가 있을 수 있다. 아래는 설계 근거이지 운용 승인이 아니다.

- [PX4 v1.16 Safety 및 failsafe](https://docs.px4.io/v1.16/en/config/safety): 배터리, 조종 입력, GCS 데이터 링크, 위치 및 offboard 손실은 별도의 조건과 설정된 대응을 가진다. 동시에 발생하면 심각도가 높은 대응을 선택한다. 설정·기체·상황에 따라 결과가 달라지므로 GCS에서 착륙이나 RTL 실행을 추정하지 않는다.
- [PX4 flight modes](https://docs.px4.io/v1.16/en/concept/flight_modes), [manual setup](https://docs.px4.io/v1.16/en/config_mc/): 모드마다 자세·위치·수동 입력 등 요구 조건이 다르다. 연결만으로 조종 권한이나 입력 장치 준비를 추론하지 않는다.
- [PX4 사전 점검](https://docs.px4.io/v1.16/en/advanced_config/prearm_arm_disarm): PX4는 자체 arming 검사를 수행하며 SYS_STATUS는 그 일부만 제공한다. FireScene 체크리스트는 전체 PX4 판정이 아니다.
- [PX4 Offboard](https://docs.px4.io/v1.16/en/flight_modes/offboard): 외부 setpoint의 지속 신호와 손실 시 설정된 동작이 필요하다. 이번 단계에는 offboard나 실제 setpoint 송신을 구현하지 않는다.
- [PX4 SITL](https://docs.px4.io/v1.16/en/simulation/), [Gazebo](https://docs.px4.io/v1.16/en/sim_gazebo_gz/): 별도 PX4 프로세스와 시뮬레이터가 필요하다. 합성 상태 생성기는 SITL이 아니다.
- [MAVLink common](https://mavlink.io/en/messages/common.html): 메시지별 단위·센티널·범위는 독립적으로 확인한다. 데이터가 존재한다는 사실은 인증이나 품질 보증이 아니다.
- [QGroundControl 조이스틱](https://docs.qgroundcontrol.com/master/en/qgc-user-guide/setup_view/joystick.html): 장치 선택·보정·축과 버튼 확인 후 활성화하며 실제 MAVLink 제어를 전송한다. FireScene 입력 연습은 별개다. [QGC 안전 설정](https://docs.px4.io/v1.16/en/config/safety)은 PX4 설정을 노출한다.
- [DJI Pilot 2 공식 자료](https://enterprise.dji.com/dji-pilot-2)와 [FlightHub 2 자료](https://enterprise.dji.com/flighthub-2/downloads): 관제 및 기체 제품군의 기능·매뉴얼을 제공한다. 세부 권한·failsafe 동작은 제품·버전에 따라 다르며 여기서 PX4 정책으로 일반화하지 않는다.

FireScene 안전 상태는 수신된 제한적 정보의 관찰 결과이며, PX4의 failsafe, 실제 제어권, RC 링크나 현장 안전을 대체하거나 보증하지 않는다. 원격 장치 식별은 인증되지 않았다.
