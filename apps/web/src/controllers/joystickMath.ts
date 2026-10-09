import type { Stick } from '../types'

export function normalizeJoystick(dx: number, dy: number, radius: number, deadZone = 0.08): Stick {
  if (!Number.isFinite(dx) || !Number.isFinite(dy) || radius <= 0) return { x: 0, y: 0 }
  const distance = Math.min(Math.hypot(dx, dy), radius)
  if (distance / radius <= deadZone) return { x: 0, y: 0 }
  const angle = Math.atan2(dy, dx)
  const scaled = (distance / radius - deadZone) / (1 - deadZone)
  return { x: clamp(Math.cos(angle) * scaled), y: clamp(Math.sin(angle) * scaled) }
}

const clamp = (value: number) => Math.max(-1, Math.min(1, value))

