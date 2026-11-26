// Invalidate observations and queued connects when the user changes intent.
export function createConnectionEpoch() {
  let current = 0;
  return {
    capture: () => current,
    advance: () => ++current,
    isCurrent: (token: number) => token === current,
  };
}
