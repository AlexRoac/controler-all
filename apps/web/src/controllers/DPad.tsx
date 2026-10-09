import { ChevronDown, ChevronLeft, ChevronRight, ChevronUp } from 'lucide-react'
import type { ButtonName, GamepadState } from '../types'
import { PressButton } from './PressButton'

export function DPad({ buttons, onButton }: { buttons: GamepadState['buttons']; onButton: (name: ButtonName, value: boolean) => void }) {
  const items = [
    ['dpadUp', 'Arriba', <ChevronUp key="u" />], ['dpadLeft', 'Izquierda', <ChevronLeft key="l" />],
    ['dpadRight', 'Derecha', <ChevronRight key="r" />], ['dpadDown', 'Abajo', <ChevronDown key="d" />],
  ] as const
  return <div className="dpad" aria-label="Cruceta direccional">{items.map(([name, label, icon]) => <PressButton key={name} className={`dpad-${name.slice(4).toLowerCase()}`} label={label} pressed={buttons[name]} onPress={(value) => onButton(name, value)}>{icon}</PressButton>)}<div className="dpad-center" /></div>
}

