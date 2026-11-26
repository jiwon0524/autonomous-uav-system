// A slow or failed module load must not leave an unexplained blank screen.
window.setTimeout(() => {
  const screen = document.getElementById('launch-screen');
  if (!screen) return;
  const status = document.getElementById('launch-status-text');
  const retry = document.getElementById('launch-retry');
  if (status) status.textContent = '화면 시작이 지연되고 있습니다 · Interface startup is taking longer than expected';
  if (retry) { retry.hidden = false; retry.onclick = () => window.location.reload(); }
}, 8000);
