import { useRef, useState } from 'react'
import type { PointerEvent } from 'react'
import { ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Delete, Music2, Play, SkipBack, SkipForward, Volume1, Volume2, VolumeX } from 'lucide-react'
import type { MediaAction, Modifier, OutgoingMessage, RemoteKey } from '../services/protocol'

type Send = (message: OutgoingMessage) => boolean

export function DesktopController({ send }: { send: Send }) {
  const pointers = useRef(new Map<number, { x: number; y: number }>())
  const [sensitivity, setSensitivity] = useState(1.25)
  const [text, setText] = useState('')
  const [modifiers, setModifiers] = useState<Modifier[]>([])
  const pointerDown = (event: PointerEvent<HTMLDivElement>) => { event.currentTarget.setPointerCapture(event.pointerId); pointers.current.set(event.pointerId, { x: event.clientX, y: event.clientY }) }
  const pointerMove = (event: PointerEvent<HTMLDivElement>) => {
    const previous = pointers.current.get(event.pointerId); if (!previous) return
    const dx = event.clientX - previous.x; const dy = event.clientY - previous.y; pointers.current.set(event.pointerId, { x: event.clientX, y: event.clientY })
    if (pointers.current.size === 1) send({ type: 'mouse_move', sequence: undefined, dx: Math.round(dx * sensitivity), dy: Math.round(dy * sensitivity) })
    else if (Math.abs(dy) > 2) send({ type: 'mouse_scroll', sequence: undefined, delta: Math.sign(-dy) })
  }
  const pointerUp = (event: PointerEvent<HTMLDivElement>) => {
    const previous = pointers.current.get(event.pointerId); pointers.current.delete(event.pointerId)
    if (previous && Math.hypot(event.clientX - previous.x, event.clientY - previous.y) < 5) click('left')
  }
  const click = (button: 'left' | 'right') => { send({ type: 'mouse_button', sequence: undefined, button, pressed: true }); window.setTimeout(() => send({ type: 'mouse_button', sequence: undefined, button, pressed: false }), 45) }
  const key = (remoteKey: RemoteKey) => { send({ type: 'key', sequence: undefined, key: remoteKey, pressed: true, modifiers }); window.setTimeout(() => send({ type: 'key', sequence: undefined, key: remoteKey, pressed: false, modifiers }), 55) }
  const media = (action: MediaAction) => send({ type: 'media', sequence: undefined, action })
  const toggleModifier = (modifier: Modifier) => setModifiers((current) => current.includes(modifier) ? current.filter((item) => item !== modifier) : [...current, modifier])
  return <section className="desktop-layout">
    <div className="trackpad-card">
      <div className="trackpad-head"><div><small>SUPERFICIE TÁCTIL</small><span>1 dedo mueve · 2 dedos desplazan</span></div><label>Sensibilidad <input aria-label="Sensibilidad del cursor" type="range" min=".4" max="2.4" step=".1" value={sensitivity} onChange={(e) => setSensitivity(Number(e.target.value))} /></label></div>
      <div className="trackpad" role="application" aria-label="Trackpad remoto" tabIndex={0} onPointerDown={pointerDown} onPointerMove={pointerMove} onPointerUp={pointerUp} onPointerCancel={pointerUp}><div className="trackpad-grid" /><span>Mueve el cursor</span></div>
      <div className="mouse-buttons"><button type="button" onPointerDown={() => click('left')}>CLIC IZQUIERDO</button><button type="button" onPointerDown={() => click('right')}>CLIC DERECHO</button></div>
    </div>
    <div className="desktop-side">
      <section className="control-card"><header><Music2 size={16} /><span>Multimedia</span></header><div className="media-grid"><IconButton label="Anterior" onPress={() => media('previous_track')}><SkipBack /></IconButton><IconButton label="Reproducir o pausar" primary onPress={() => media('play_pause')}><Play /></IconButton><IconButton label="Siguiente" onPress={() => media('next_track')}><SkipForward /></IconButton><IconButton label="Bajar volumen" onPress={() => media('volume_down')}><Volume1 /></IconButton><IconButton label="Silenciar" onPress={() => media('mute')}><VolumeX /></IconButton><IconButton label="Subir volumen" onPress={() => media('volume_up')}><Volume2 /></IconButton></div></section>
      <section className="control-card keyboard-card"><header><span>Teclado rápido</span></header><form onSubmit={(e) => { e.preventDefault(); if (text) { send({ type: 'text_input', sequence: undefined, text }); setText('') } }}><input aria-label="Texto para enviar" value={text} onChange={(e) => setText(e.target.value)} placeholder="Escribe en la laptop…" /><button type="submit">ENVIAR</button></form><div className="modifier-row">{(['ctrl','alt','shift','meta'] as const).map((item) => <button type="button" aria-pressed={modifiers.includes(item)} className={modifiers.includes(item) ? 'active' : ''} key={item} onClick={() => toggleModifier(item)}>{item === 'meta' ? 'WIN' : item.toUpperCase()}</button>)}</div><div className="key-grid"><button type="button" onClick={() => key('escape')}>ESC</button><button type="button" onClick={() => key('tab')}>TAB</button><button type="button" onClick={() => key('backspace')} aria-label="Backspace"><Delete /></button><button type="button" onClick={() => key('enter')}>ENTER</button><span /><button type="button" onClick={() => key('arrow_up')} aria-label="Arriba"><ArrowUp /></button><span /><button type="button" onClick={() => key('arrow_left')} aria-label="Izquierda"><ArrowLeft /></button><button type="button" onClick={() => key('arrow_down')} aria-label="Abajo"><ArrowDown /></button><button type="button" onClick={() => key('arrow_right')} aria-label="Derecha"><ArrowRight /></button></div></section>
    </div>
  </section>
}

function IconButton({ label, onPress, primary = false, children }: { label: string; onPress: () => void; primary?: boolean; children: React.ReactNode }) {
  return <button type="button" aria-label={label} className={primary ? 'primary' : ''} onPointerDown={onPress}>{children}</button>
}

