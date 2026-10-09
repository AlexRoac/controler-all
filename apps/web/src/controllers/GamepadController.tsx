import { useCallback, useEffect, useRef, useState } from 'react'
import { Menu, PanelsTopLeft } from 'lucide-react'
import type { OutgoingMessage } from '../services/protocol'
import { emptyGamepad } from '../types'
import type { ButtonName, GamepadState } from '../types'
import { DPad } from './DPad'
import { Joystick } from './Joystick'
import { PressButton } from './PressButton'

export function GamepadController({ send, diagnostic }: { send: (message: OutgoingMessage) => boolean; diagnostic: GamepadState | null }) {
  const [state, setState] = useState<GamepadState>(emptyGamepad)
  const stateRef = useRef(state); const frameRef = useRef<number | null>(null)
  const update = useCallback((recipe: (current: GamepadState) => GamepadState) => {
    const next = recipe(stateRef.current); stateRef.current = next; setState(next)
    if (frameRef.current === null) frameRef.current = requestAnimationFrame(() => { frameRef.current = null; send({ type: 'gamepad_state', sequence: undefined, ...stateRef.current }) })
  }, [send])
  useEffect(() => () => { if (frameRef.current !== null) cancelAnimationFrame(frameRef.current); send({ type: 'gamepad_state', sequence: undefined, ...emptyGamepad() }) }, [send])
  const button = useCallback((name: ButtonName, value: boolean) => update((s) => ({ ...s, buttons: { ...s.buttons, [name]: value } })), [update])
  const trigger = (side: 'left' | 'right', value: number) => update((s) => ({ ...s, triggers: { ...s.triggers, [side]: value } }))
  return <section className="gamepad-shell" aria-label="Control Xbox virtual">
    <div className="shoulders left"><PressButton label="LB" className="bumper" pressed={state.buttons.lb} onPress={(v) => button('lb', v)} /><label className="trigger"><span>LT</span><input aria-label="Gatillo izquierdo" type="range" min="0" max="1" step="0.01" value={state.triggers.left} onChange={(e) => trigger('left', Number(e.target.value))} /></label></div>
    <div className="shoulders right"><label className="trigger"><span>RT</span><input aria-label="Gatillo derecho" type="range" min="0" max="1" step="0.01" value={state.triggers.right} onChange={(e) => trigger('right', Number(e.target.value))} /></label><PressButton label="RB" className="bumper" pressed={state.buttons.rb} onPress={(v) => button('rb', v)} /></div>
    <div className="gamepad-left"><Joystick label="L STICK" value={state.leftStick} deadZone={.08} onChange={(v) => update((s) => ({ ...s, leftStick: v }))} onPress={(v) => button('l3', v)} /><DPad buttons={state.buttons} onButton={button} /></div>
    <div className="gamepad-center"><div className="mini-buttons"><PressButton label="View" className="mini" pressed={state.buttons.back} onPress={(v) => button('back', v)}><PanelsTopLeft size={14} /></PressButton><PressButton label="Menu" className="mini" pressed={state.buttons.start} onPress={(v) => button('start', v)}><Menu size={15} /></PressButton></div><div className="x-orb">R</div><div className="diagnostic-mini"><i className={diagnostic ? 'live' : ''} /><span>{diagnostic ? 'INPUT LIVE' : 'ESPERANDO'}</span></div></div>
    <div className="gamepad-right"><div className="abxy">{(['y','x','b','a'] as const).map((name) => <PressButton key={name} label={name.toUpperCase()} className={`face face-${name}`} pressed={state.buttons[name]} onPress={(v) => button(name, v)} />)}</div><Joystick label="R STICK" value={state.rightStick} deadZone={.08} onChange={(v) => update((s) => ({ ...s, rightStick: v }))} onPress={(v) => button('r3', v)} /></div>
  </section>
}
