/** UI-facing window contract. A future desktop shell can replace this adapter. */
export interface StationShell {
  currentView(): 'dashboard' | 'sensors';
  openSensorMonitor(): boolean;
}
export function createBrowserShell():StationShell {
  let sensorWindow: Window | null = null;
  return {
    currentView:()=>typeof window!=='undefined' && new URLSearchParams(window.location.search).get('view')==='sensors'?'sensors':'dashboard',
    openSensorMonitor:()=>{
      if(typeof window==='undefined')return false;
      if(sensorWindow && !sensorWindow.closed){sensorWindow.focus();return true;}
      const url=new URL(window.location.href);
      url.searchParams.set('view','sensors');
      sensorWindow=window.open(url.href,'uav-sensor-monitor','popup,width=1280,height=760');
      return sensorWindow!==null;
    },
  };
}
export const stationShell:StationShell=createBrowserShell();
