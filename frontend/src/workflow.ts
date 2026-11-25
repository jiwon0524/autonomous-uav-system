import type { MapSelection } from './MapView';
export type SelectedMission = {id:string;name:string;status:string;selection:MapSelection|null};
export function sameLocation(a:MapSelection|null,b:MapSelection|null):boolean {
 if(!a||!b)return a===b;
 return a.id===b.id && a.name===b.name && a.address===b.address && a.center[0]===b.center[0] && a.center[1]===b.center[1] && a.radius===b.radius && JSON.stringify(a.geometry)===JSON.stringify(b.geometry);
}
export function missionAfterMapChange(mission:SelectedMission|null,selection:MapSelection|null):SelectedMission|null {
 return mission && sameLocation(mission.selection,selection) ? mission : null;
}
export function selectedMissionIfPresent<T extends {id:string}>(id:string|null,items:T[]):T|null {return id ? items.find(m=>m.id===id)??null : null;}
