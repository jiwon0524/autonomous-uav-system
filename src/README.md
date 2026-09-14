# src

실제 자율비행 소프트웨어 코드를 관리합니다.

프로젝트가 진행되면 아래와 같이 세분화할 예정입니다.

```text
src/
├── perception/   # YOLO, OpenCV, 목표물 탐지·추적
├── planning/     # A*, 장애물 회피, 경로 재계획
├── control/      # 추종 제어, 속도/자세 명령
├── integration/  # ROS2 ↔ PX4 연동
└── safety/       # Fail-safe, 통신/센서 오류 대응
```

아직 구현되지 않은 폴더는 필요해지는 시점에 생성합니다.
