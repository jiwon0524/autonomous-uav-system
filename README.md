# Autonomous UAV System

ROS2/PX4 기반 자율비행 드론 시스템 프로젝트입니다.

본 프로젝트는 카메라 기반 목표물 탐지·추적, 장애물 인식·회피, 동적 경로 재계획, 목표물 재탐색 및 안전 비행 기능을 하나의 시스템으로 통합하는 것을 목표로 합니다.

## Core Pipeline

```text
Camera / Sensor
      ↓
Perception (AI Detection / Tracking)
      ↓
Decision / State Machine
      ↓
Path Planning / Replanning
      ↓
Control
      ↓
ROS2 ↔ PX4 / Pixhawk
      ↓
UAV
```

## Planned Tech Stack

- Ubuntu / Linux
- ROS2
- PX4 / Pixhawk
- Python / C++
- YOLO / OpenCV
- A* path planning

## Repository Structure

```text
autonomous-uav-system/
├── src/          # 실제 자율비행 소프트웨어 코드
├── config/       # 파라미터 및 설정 파일
├── experiments/  # 실험 코드와 결과 정리
├── docs/         # 설계 및 기술 문서
└── README.md
```

## Development Plan

1. ROS2 / PX4 개발환경 구축
2. PX4 SITL 기본 비행 검증
3. 카메라 입력 및 목표물 탐지
4. 목표물 추적 및 자동 추종
5. 장애물 인식
6. A* 경로계획 및 동적 재계획
7. 목표물 재탐색
8. ROS2/PX4 통합
9. 실제 기체 단계별 시험
10. 반복 실험 및 성능 분석

> 현재 프로젝트 초기 단계이며, 실제 기체 및 센서 사양은 학교 승인과 구매 확정 후 업데이트합니다.

## Dashboard / UAV Control Station

`dashboard/` contains the React + TypeScript control-station UI. See [Dashboard README](dashboard/README.md) for development and validation commands.

`dashboard_bridge/` preserves the read-only ROS2/PX4 status bridge; see [Bridge README](dashboard_bridge/README.md). Actual ROS2/PX4 validation remains pending on Ubuntu. Flight commands and sensor/position integrations are not enabled.
