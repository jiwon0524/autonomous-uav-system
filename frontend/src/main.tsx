import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'

function ReadyApp() {
  React.useLayoutEffect(() => { document.getElementById('launch-screen')?.remove() }, [])
  return <App />
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ReadyApp />
  </React.StrictMode>,
)
