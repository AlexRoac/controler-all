import { Radio, WifiOff } from 'lucide-react'
import type { ConnectionStatus } from '../types'

export function ConnectionBadge({ status, latency }: { status: ConnectionStatus; latency: number | null }) {
  const online = status === 'connected'
  return <div className={`connection-badge ${online ? 'online' : ''}`} aria-live="polite">
    {online ? <Radio size={14} /> : <WifiOff size={14} />}
    <span>{online ? `${latency ?? '—'} ms` : status === 'reconnecting' ? 'Reconectando' : 'Sin conexión'}</span>
  </div>
}

