export type Stick = { x: number; y: number }
export type ButtonName = 'a' | 'b' | 'x' | 'y' | 'lb' | 'rb' | 'start' | 'back' | 'l3' | 'r3' | 'dpadUp' | 'dpadDown' | 'dpadLeft' | 'dpadRight'
export type GamepadState = {
  leftStick: Stick
  rightStick: Stick
  buttons: Record<ButtonName, boolean>
  triggers: { left: number; right: number }
}
export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'reconnecting' | 'error'
export type ServerInfo = { name: string; addresses: string[]; port: number; connected: boolean; gamepadBackend: string; protocolVersion: number }
export type ServerMessage =
  | { type: 'authenticated'; sessionId: string; serverName: string }
  | { type: 'heartbeat_ack'; clientTime: number; serverTime: number }
  | { type: 'diagnostic'; sequence: number; gamepad: GamepadState & { sequence: number }; backend: string }
  | { type: 'error'; code: string; message: string }
  | { type: 'disconnected'; reason: string }

export const emptyGamepad = (): GamepadState => ({
  leftStick: { x: 0, y: 0 }, rightStick: { x: 0, y: 0 },
  buttons: { a: false, b: false, x: false, y: false, lb: false, rb: false, start: false, back: false, l3: false, r3: false, dpadUp: false, dpadDown: false, dpadLeft: false, dpadRight: false },
  triggers: { left: 0, right: 0 },
})

