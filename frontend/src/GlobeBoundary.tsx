import {Component, type ReactNode} from 'react';
export class GlobeBoundary extends Component<{onClose:()=>void;children:ReactNode},{failed:boolean}>{
 state={failed:false};
 static getDerivedStateFromError(){return {failed:true};}
 render(){return this.state.failed?<section className="globe-experiment"><p role="alert">Global 실험을 열지 못했습니다.</p><button onClick={this.props.onClose}>기본 지도 복귀</button></section>:this.props.children;}
}
