// Explicit basis conversion only; origin and geographic registration are not inferred.
export function nedToEnu([north,east,down]: readonly [number,number,number]): [number,number,number] { return [east,north,-down]; }
// q = [x,y,z,w], unit quaternion, active rotation of a vector.
export function rotateUnitQuaternion([x,y,z]: readonly [number,number,number], [qx,qy,qz,qw]: readonly [number,number,number,number]): [number,number,number] {
  const norm=qx*qx+qy*qy+qz*qz+qw*qw;
  if(!Number.isFinite(norm)||Math.abs(norm-1)>1e-6) throw new Error('unit quaternion required');
  const tx=2*(qy*z-qz*y),ty=2*(qz*x-qx*z),tz=2*(qx*y-qy*x);
  return [x+qw*tx+qy*tz-qz*ty,y+qw*ty+qz*tx-qx*tz,z+qw*tz+qx*ty-qy*tx];
}
