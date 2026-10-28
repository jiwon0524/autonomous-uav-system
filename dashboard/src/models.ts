export type MissionState = 'STANDBY' | 'TAKEOFF' | 'POSITION_CONTROL' | 'TARGET_TRACKING' | 'HOLD' | 'PATROL' | 'RETURNING' | 'LANDING' | 'MISSION_CANCELLED';
export const missionLabels: Record<MissionState, string> = {STANDBY:'대기',TAKEOFF:'이륙',POSITION_CONTROL:'위치 제어',TARGET_TRACKING:'타겟 추적',HOLD:'위치 유지',PATROL:'자동 순찰',RETURNING:'복귀 중',LANDING:'착륙 중',MISSION_CANCELLED:'임무 중단'};
export interface VehicleData { missionState: MissionState; altitude: number | null; groundSpeed: number | null; battery: number | null }
export const vehicle: VehicleData = {missionState:'STANDBY',altitude:null,groundSpeed:null,battery:null};
export type { SensorId, SensorSnapshot } from './sensors/model';
export type { Target } from './targets/model';
export type { SafePoint, Destination, DestinationSelection } from './navigation/model';
export type RiskLevel = 'NORMAL' | 'WARNING' | 'CRITICAL';
export const riskLabels: Record<RiskLevel,string> = {NORMAL:'정상',WARNING:'주의',CRITICAL:'위험'};
