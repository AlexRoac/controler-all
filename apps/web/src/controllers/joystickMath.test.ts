import { describe, expect, it } from 'vitest'
import { normalizeJoystick } from './joystickMath'

describe('normalizeJoystick', () => {
  it('normalizes cardinal movement', () => expect(normalizeJoystick(100, 0, 100, 0)).toEqual({ x: 1, y: 0 }))
  it('clamps beyond radius', () => expect(normalizeJoystick(300, 0, 100, 0).x).toBe(1))
  it('supports diagonals', () => { const v = normalizeJoystick(70, 70, 100, 0); expect(v.x).toBeCloseTo(v.y); expect(v.x).toBeGreaterThan(0.69) })
  it('applies dead zone', () => expect(normalizeJoystick(4, 3, 100, 0.08)).toEqual({ x: 0, y: 0 }))
})

