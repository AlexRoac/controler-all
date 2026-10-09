import { useRef, useState } from 'react'
import type { PointerEvent } from 'react'
import type { Stick } from '../types'
import { normalizeJoystick } from './joystickMath'

type Props = { label: string; value: Stick; deadZone: number; onChange: (value: Stick) => void; onPress: (pressed: boolean) => void }

export function Joystick({ label, value, deadZone, onChange, onPress }: Props) {
  const baseRef = useRef<HTMLDivElement>(null)
  const pointerRef = useRef<number | null>(null)
  const startRef = useRef(0)
  const [active, setActive] = useState(false)
  const update = (event: PointerEvent<HTMLDivElement>) => {
    const rect = baseRef.current?.getBoundingClientRect(); if (!rect) return
    onChange(normalizeJoystick(event.clientX - (rect.left + rect.width / 2), event.clientY - (rect.top + rect.height / 2), rect.width * 0.34, deadZone))
  }
  const down = (event: PointerEvent<HTMLDivElement>) => { if (pointerRef.current !== null) return; pointerRef.current = event.pointerId; startRef.current = performance.now(); event.currentTarget.setPointerCapture(event.pointerId); setActive(true); update(event) }
  const move = (event: PointerEvent<HTMLDivElement>) => { if (event.pointerId === pointerRef.current) update(event) }
  const release = (event: PointerEvent<HTMLDivElement>) => { if (event.pointerId !== pointerRef.current) return; if (performance.now() - startRef.current < 180 && Math.hypot(value.x, value.y) < .25) { onPress(true); window.setTimeout(() => onPress(false), 80) } pointerRef.current = null; setActive(false); onChange({ x: 0, y: 0 }) }
  return <div className="stick-wrap"><div ref={baseRef} className={`stick-base ${active ? 'active' : ''}`} role="slider" aria-label={`${label}, pulsa para L3/R3`} aria-valuetext={`${value.x.toFixed(2)}, ${value.y.toFixed(2)}`} tabIndex={0} onPointerDown={down} onPointerMove={move} onPointerUp={release} onPointerCancel={release}>
    <div className="stick-rings" /><div className="stick-knob" style={{ transform: `translate(${value.x * 34}%, ${value.y * 34}%)` }}><span /></div>
  </div><small>{label}</small></div>
}

