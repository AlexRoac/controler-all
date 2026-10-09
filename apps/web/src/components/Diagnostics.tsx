import type { GamepadState } from '../types'

export function Diagnostics({ state, backend }: { state: GamepadState | null; backend: string }) {
  if (!state) return <div className="diagnostic-panel empty"><span>Envía una entrada del gamepad para ver telemetría.</span></div>
  const active = Object.entries(state.buttons).filter(([, pressed]) => pressed).map(([name]) => name.toUpperCase())
  return <div className="diagnostic-panel"><div><small>BACKEND</small><strong>{backend}</strong></div><div><small>BOTONES</small><strong>{active.join(' · ') || 'NINGUNO'}</strong></div><div><small>L STICK</small><strong>{state.leftStick.x.toFixed(2)} / {state.leftStick.y.toFixed(2)}</strong></div><div><small>R STICK</small><strong>{state.rightStick.x.toFixed(2)} / {state.rightStick.y.toFixed(2)}</strong></div><div><small>TRIGGERS</small><strong>{state.triggers.left.toFixed(2)} / {state.triggers.right.toFixed(2)}</strong></div></div>
}

