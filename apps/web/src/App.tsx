import { useState } from 'react'
import { Gamepad2, Gauge, LogOut, Monitor, Octagon } from 'lucide-react'
import { ConnectionBadge } from './components/ConnectionBadge'
import { ConnectionScreen } from './components/ConnectionScreen'
import { Diagnostics } from './components/Diagnostics'
import { DesktopController } from './controllers/DesktopController'
import { GamepadController } from './controllers/GamepadController'
import { useRemotePad } from './hooks/useRemotePad'

type Mode = 'gamepad' | 'desktop'

export default function App() {
  const remote = useRemotePad()
  const [mode, setMode] = useState<Mode>('gamepad')
  const [showDiagnostics, setShowDiagnostics] = useState(false)
  if (remote.status !== 'connected') return <ConnectionScreen info={remote.serverInfo} status={remote.status} error={remote.error} onPair={remote.pair} onRefresh={remote.refreshInfo} />
  const emergency = () => { remote.send({ type: 'emergency_stop', sequence: undefined }); remote.disconnect() }
  return <main className={`app-shell mode-${mode}`}>
    <header className="app-header"><div className="compact-brand"><div className="brand-mark"><Gamepad2 /></div><div><strong>Remote<span>Pad</span></strong><small>{remote.serverInfo?.name}</small></div></div><nav className="mode-switch" aria-label="Modo de control"><button type="button" className={mode === 'gamepad' ? 'active' : ''} onClick={() => setMode('gamepad')}><Gamepad2 /> GAMEPAD</button><button type="button" className={mode === 'desktop' ? 'active' : ''} onClick={() => setMode('desktop')}><Monitor /> DESKTOP</button></nav><div className="header-actions"><ConnectionBadge status={remote.status} latency={remote.latency} /><button className="icon-action" type="button" aria-label="Diagnóstico" aria-pressed={showDiagnostics} onClick={() => setShowDiagnostics((v) => !v)}><Gauge /></button><button className="icon-action emergency" type="button" aria-label="Parada de emergencia" onClick={emergency}><Octagon /></button><button className="icon-action" type="button" aria-label="Desconectar" onClick={remote.disconnect}><LogOut /></button></div></header>
    <div className="mode-content">{mode === 'gamepad' ? <GamepadController send={remote.send} diagnostic={remote.diagnostic} /> : <DesktopController send={remote.send} />}</div>
    {showDiagnostics ? <Diagnostics state={remote.diagnostic} backend={remote.serverInfo?.gamepadBackend ?? 'diagnostic'} /> : null}
  </main>
}

