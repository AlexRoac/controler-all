# Protocolo RemotePad v1

`GET /api/info` devuelve información pública del servidor. `POST /api/pair` recibe `{ code, clientName }`; al validar el PIN entrega un token temporal. El token no se persiste en la laptop y el navegador lo conserva sólo en `sessionStorage`.

El cliente abre `/ws` y en cinco segundos debe enviar:

```json
{"type":"authenticate","token":"…","protocolVersion":1}
```

Después de `authenticated`, cada acción mutable lleva una secuencia creciente. El servidor ignora secuencias menores o iguales a la última aceptada. `heartbeat` no lleva secuencia y se envía cada tres segundos; doce segundos sin mensajes cierran la sesión.

Los mensajes admitidos están definidos como enums cerrados en `crates/protocol/src/lib.rs`. No existe un mensaje de comandos genérico. Ejes aceptan `[-1,1]`, triggers `[0,1]`, movimiento y scroll se limitan también en el adaptador de entrada.

Al cerrar el socket, superar el rate limit, recibir una parada de emergencia o finalizar el proceso, se ejecuta el reset de ambos backends.

