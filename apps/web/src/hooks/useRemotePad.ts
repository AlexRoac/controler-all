import { useCallback, useEffect, useRef, useState } from 'react'
import type { OutgoingMessage } from '../services/protocol'
import type { ConnectionStatus, GamepadState, ServerInfo, ServerMessage } from '../types'

const TOKEN_KEY = 'remotepad.session.v1'

export function useRemotePad() {
  const [status, setStatus] = useState<ConnectionStatus>('disconnected')
  const [serverInfo, setServerInfo] = useState<ServerInfo | null>(null)
  const [latency, setLatency] = useState<number | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [diagnostic, setDiagnostic] = useState<GamepadState | null>(null)
  const socketRef = useRef<WebSocket | null>(null)
  const sequenceRef = useRef(0)
  const reconnectTimerRef = useRef<number | null>(null)
  const intentionalCloseRef = useRef(false)
  const urlRef = useRef(baseUrl())

  const send = useCallback((message: OutgoingMessage) => {
    const socket = socketRef.current
    if (!socket || socket.readyState !== WebSocket.OPEN || status !== 'connected') return false
    const payload = 'sequence' in message && message.sequence === undefined ? { ...message, sequence: ++sequenceRef.current } : message
    socket.send(JSON.stringify(payload))
    return true
  }, [status])

  const openSocket = useCallback((token: string, reconnecting = false) => {
    intentionalCloseRef.current = false
    setStatus(reconnecting ? 'reconnecting' : 'connecting')
    setError(null)
    const wsUrl = urlRef.current.replace(/^http/, 'ws') + '/ws'
    const socket = new WebSocket(wsUrl)
    socketRef.current = socket
    socket.addEventListener('open', () => socket.send(JSON.stringify({ type: 'authenticate', token, protocolVersion: 1 })))
    socket.addEventListener('message', (event) => {
      const message = JSON.parse(String(event.data)) as ServerMessage
      if (message.type === 'authenticated') setStatus('connected')
      if (message.type === 'heartbeat_ack') setLatency(Math.max(0, Math.round((Date.now() - message.clientTime) / 2)))
      if (message.type === 'diagnostic') setDiagnostic(message.gamepad)
      if (message.type === 'error') { setError(message.message); setStatus('error') }
    })
    socket.addEventListener('close', () => {
      socketRef.current = null
      setLatency(null)
      if (!intentionalCloseRef.current && sessionStorage.getItem(TOKEN_KEY)) {
        setStatus('reconnecting')
        reconnectTimerRef.current = window.setTimeout(() => openSocket(token, true), 1500)
      } else setStatus('disconnected')
    })
    socket.addEventListener('error', () => setError('No se pudo abrir el canal WebSocket.'))
  }, [])

  const refreshInfo = useCallback(async () => {
    try {
      const response = await fetch(`${urlRef.current}/api/info`)
      if (!response.ok) throw new Error('Servidor no disponible')
      setServerInfo(await response.json() as ServerInfo)
    } catch (cause) { setError(cause instanceof Error ? cause.message : 'Servidor no disponible') }
  }, [])

  const pair = useCallback(async (code: string) => {
    setStatus('connecting'); setError(null)
    try {
      const response = await fetch(`${urlRef.current}/api/pair`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ code, clientName: navigator.userAgent.slice(0, 80) }) })
      if (!response.ok) throw new Error('Código incorrecto o vencido')
      const { token } = await response.json() as { token: string }
      sessionStorage.setItem(TOKEN_KEY, token)
      openSocket(token)
    } catch (cause) { setStatus('error'); setError(cause instanceof Error ? cause.message : 'No se pudo emparejar') }
  }, [openSocket])

  const disconnect = useCallback(() => {
    intentionalCloseRef.current = true
    sessionStorage.removeItem(TOKEN_KEY)
    if (reconnectTimerRef.current) window.clearTimeout(reconnectTimerRef.current)
    socketRef.current?.close()
    setStatus('disconnected')
  }, [])

  useEffect(() => {
    void refreshInfo()
    const token = sessionStorage.getItem(TOKEN_KEY)
    if (token) openSocket(token)
    return () => { intentionalCloseRef.current = true; socketRef.current?.close(); if (reconnectTimerRef.current) window.clearTimeout(reconnectTimerRef.current) }
  }, [openSocket, refreshInfo])

  useEffect(() => {
    if (status !== 'connected') return
    const id = window.setInterval(() => socketRef.current?.send(JSON.stringify({ type: 'heartbeat', clientTime: Date.now() })), 3000)
    return () => window.clearInterval(id)
  }, [status])

  return { status, serverInfo, latency, error, diagnostic, pair, disconnect, send, refreshInfo }
}

function baseUrl() {
  if (import.meta.env.DEV) return `${location.protocol}//${location.hostname}:8787`
  return location.origin
}
