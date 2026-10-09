import type { PointerEvent, ReactNode } from 'react'

type Props = { label: string; className?: string; pressed: boolean; onPress: (pressed: boolean) => void; children?: ReactNode }
export function PressButton({ label, className = '', pressed, onPress, children }: Props) {
  const down = (event: PointerEvent<HTMLButtonElement>) => { event.preventDefault(); event.currentTarget.setPointerCapture(event.pointerId); onPress(true) }
  const up = (event: PointerEvent<HTMLButtonElement>) => { event.preventDefault(); onPress(false) }
  return <button type="button" aria-label={label} aria-pressed={pressed} className={`${className} ${pressed ? 'pressed' : ''}`} onPointerDown={down} onPointerUp={up} onPointerCancel={up} onContextMenu={(e) => e.preventDefault()}>{children ?? label}</button>
}

