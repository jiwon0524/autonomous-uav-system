export function pixelBox(bbox: readonly [number, number, number, number], width: number, height: number) {
  return { x:bbox[0]*width, y:bbox[1]*height, width:bbox[2]*width, height:bbox[3]*height };
}
