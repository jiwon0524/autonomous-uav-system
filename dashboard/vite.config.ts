import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';
export default defineConfig(({mode})=>{
  const env=loadEnv(mode,'.','DASHBOARD_');
  const proxy={'/api':{target:env.DASHBOARD_BRIDGE_URL || 'http://127.0.0.1:8765',changeOrigin:true}};
  return {plugins:[react()],server:{proxy},preview:{proxy}};
});
