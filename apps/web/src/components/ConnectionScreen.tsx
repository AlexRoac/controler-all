import { useEffect, useState } from 'react'
import { ArrowRight, Laptop, Radio, ShieldCheck, Wifi } from 'lucide-react'
import type { ConnectionStatus, ServerInfo } from '../types'

type Props = { info: ServerInfo | null; status: ConnectionStatus; error: string | null; onPair: (code: string) => void; onRefresh: () => void }

export function ConnectionScreen({ info, status, error, onPair, onRefresh }: Props) {
  const [code, setCode] = useState('')
  useEffect(() => { const id = window.setInterval(onRefresh, 5000); return () => window.clearInterval(id) }, [onRefresh])
  const connecting = status === 'connecting' || status === 'reconnecting'
  return <main className="connection-page">
    <div className="ambient ambient-one" /><div className="ambient ambient-two" />
    <section className="connect-card">
      <header className="brand-lockup"><div className="brand-mark"><Radio /></div><div><p className="eyebrow">CONTROL LOCAL</p><h1>Remote<span>Pad</span></h1></div></header>
      <div className="device-panel">
        <div className="device-icon"><Laptop /></div>
        <div className="device-copy"><small>LAPTOP DISPONIBLE</small><strong>{info?.name ?? 'Buscando servidor…'}</strong><span><Wifi size={13} /> {info?.addresses[0] ?? location.hostname}:{info?.port ?? 8787}</span></div>
        <i className={info ? 'pulse-dot' : 'pulse-dot idle'} />
      </div>
      <div className="pair-copy"><h2>Conecta tu control</h2><p>Escribe el código de seis dígitos que aparece en la consola de la laptop.</p></div>
      <form onSubmit={(event) => { event.preventDefault(); if (code.length === 6) onPair(code) }}>
        <label htmlFor="pair-code">Código de emparejamiento</label>
        <input id="pair-code" value={code} onChange={(event) => setCode(event.target.value.replace(/\D/g, '').slice(0, 6))} inputMode="numeric" pattern="[0-9]{6}" autoComplete="one-time-code" placeholder="000 000" aria-describedby={error ? 'pair-error' : undefined} />
        {error ? <p id="pair-error" className="form-error">{error}</p> : null}
        <button className="connect-button" disabled={code.length !== 6 || connecting || !info} type="submit"><span>{connecting ? 'Conectando…' : 'Conectar dispositivo'}</span><ArrowRight size={18} /></button>
      </form>
      <footer><ShieldCheck size={15} /><span>La conexión permanece en tu red local. El token nunca se incluye en la URL.</span></footer>
    </section>
  </main>
}

