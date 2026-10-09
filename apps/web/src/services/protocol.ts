import type { ButtonName, GamepadState } from '../types'

export type ClientMessage =
  | { type: 'heartbeat'; clientTime: number }
  | ({ type: 'gamepad_state'; sequence: number } & GamepadState)
  | { type: 'mouse_move'; sequence: number; dx: number; dy: number }
  | { type: 'mouse_button'; sequence: number; button: 'left' | 'right' | 'middle'; pressed: boolean }
  | { type: 'mouse_scroll'; sequence: number; delta: number }
  | { type: 'text_input'; sequence: number; text: string }
  | { type: 'key'; sequence: number; key: RemoteKey; pressed: boolean; modifiers: Modifier[] }
  | { type: 'media'; sequence: number; action: MediaAction }
  | { type: 'emergency_stop'; sequence: number }

export type OutgoingMessage = ClientMessage extends infer Message
  ? Message extends { sequence: number }
    ? Omit<Message, 'sequence'> & { sequence?: number }
    : Message
  : never

export type RemoteKey = 'enter' | 'escape' | 'backspace' | 'tab' | 'arrow_up' | 'arrow_down' | 'arrow_left' | 'arrow_right'
export type Modifier = 'ctrl' | 'alt' | 'shift' | 'meta'
export type MediaAction = 'volume_up' | 'volume_down' | 'mute' | 'play_pause' | 'next_track' | 'previous_track'

export const buttonNames: ButtonName[] = ['a', 'b', 'x', 'y', 'lb', 'rb', 'start', 'back', 'l3', 'r3', 'dpadUp', 'dpadDown', 'dpadLeft', 'dpadRight']

