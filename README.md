# RemotePad

RemotePad convierte un iPhone o Android en un control remoto para una laptop Windows. Este repositorio contiene un MVP ejecutable: React se sirve al teléfono, un WebSocket autenticado transporta las entradas y Rust las aplica con Enigo. El modo gamepad usa un backend de diagnóstico; no se presenta como XInput hasta conectar un motor de dispositivo virtual real.

## Estado real

| Capacidad | Estado |
|---|---|
| Emparejamiento por PIN, token de sesión y un solo cliente | Funcional |
| WebSocket, heartbeat, latencia, secuencias y reconexión | Funcional |
| Mouse, clics, scroll, texto, teclas y multimedia en Windows | Funcional mediante Enigo |
| Gamepad táctil multitouch y telemetría | Funcional |
| Liberación de entradas en desconexión/parada | Funcional |
| Dispositivo Xbox visible en XInput / `joy.cpl` | Requiere integrar e instalar HIDMaestro u otro driver firmado |
| TLS fuera de la LAN | No incluido; use un proxy HTTPS de confianza antes de exponerlo |

## Requisitos

- Windows 10 u 11 x64.
- [Rust](https://rustup.rs/) estable con Cargo.
- [Node.js](https://nodejs.org/) 20 o posterior y npm.
- La laptop y el teléfono en la misma red Wi-Fi; la red debe permitir tráfico entre clientes.

## Inicio rápido

Desde la raíz del repositorio:

```powershell
npm install
npm run build:web
cargo run -p remotepad-server
```

El servidor muestra el nombre de la laptop, cada IPv4 local válida, una URL, un QR y un PIN temporal. Escanee el QR o abra `http://IP-DE-LA-LAPTOP:8787` en el teléfono, escriba el PIN y pulse **Conectar dispositivo**.

## Ejecutable portable para usuarios

La interfaz web se incrusta dentro del binario de Rust durante la compilación. El usuario final sólo necesita `RemotePad.exe`: no requiere Node.js, Rust, el repositorio ni una carpeta `dist`.

Para generar localmente el ZIP de distribución:

```powershell
.\scripts\package-windows.ps1
```

El resultado queda en `release/RemotePad-v0.1.0-windows-x64.zip`. Para indicar otra versión:

```powershell
.\scripts\package-windows.ps1 -Version 0.2.0
```

El ejecutable todavía no está firmado con un certificado de firma de código. Windows SmartScreen puede mostrar una advertencia de editor desconocido; no debe ocultarse ni evadirse. Para distribución pública estable conviene firmar el `.exe` y publicar su suma SHA-256.

El workflow `.github/workflows/release.yml` compila y adjunta `RemotePad-windows-x64.zip` automáticamente al publicar una etiqueta:

```powershell
git tag v0.1.0
git push origin v0.1.0
```

También puede ejecutarse manualmente desde **GitHub → Actions → Windows release**; en ese caso genera un artifact descargable pero no crea una publicación pública.

Si Windows Firewall pregunta, permita el ejecutable únicamente en redes privadas. Para detenerlo, use `Ctrl+C`; el servidor libera mouse, teclas, modificadores y gamepad durante el cierre de cada sesión.

### Desarrollo

Ejecute el backend y Vite en terminales distintas:

```powershell
npm run dev:server
npm run dev:web
```

Vite escucha en la red y reenvía `/api` y `/ws` al puerto 8787. Para un origen de desarrollo distinto, añádalo explícitamente:

```powershell
$env:REMOTEPAD_ALLOWED_ORIGINS="http://192.168.1.20:5173"
cargo run -p remotepad-server
```

También puede cambiar el puerto con `REMOTEPAD_PORT`.

## Uso

### Desktop

- Un dedo mueve el cursor; dos dedos desplazan verticalmente.
- Un toque hace clic izquierdo y dos toques rápidos hacen doble clic.
- Los botones inferiores envían clic izquierdo/derecho.
- El campo de texto escribe en la ventana activa de Windows.
- Ctrl, Alt, Shift y Win son modificadores enclavables; el servidor siempre intenta liberarlos al soltar o desconectar.
- Los botones multimedia usan las teclas multimedia nativas de Windows.

Enigo inyecta en el escritorio interactivo del usuario. Aplicaciones elevadas pueden rechazar entrada desde un proceso sin elevar por la protección UIPI de Windows. No ejecute RemotePad como administrador salvo que comprenda ese riesgo.

### Gamepad

Use el teléfono en horizontal. Los joysticks tienen dead zone radial de 8 %, normalización `[-1, 1]`, diagonales y retorno automático. Un toque corto centrado en un stick envía L3/R3. Los botones usan Pointer Events con captura, por lo que admiten combinaciones simultáneas.

El panel de diagnóstico muestra exactamente el último estado aceptado por Rust. El backend incluido se llama `diagnostic`: valida y registra el control, pero **no crea un dispositivo Xbox**.

## XInput real: decisión técnica

ViGEmBus fue retirado y su repositorio quedó archivado en 2023; no se adopta como dependencia nueva. El candidato actual es [HIDMaestro](https://github.com/hifihedgehog/HIDMaestro), un minidriver UMDF2 que incluye perfiles Xbox con interfaz XUSB. Microsoft documenta UMDF2 como una vía soportada para minidrivers HID.

Antes de distribuir comercialmente hay que auditar, fijar una versión y probar su instalación/firma en las versiones de Windows objetivo. La integración prevista es un pequeño adaptador C#/C++ que use su SDK y exponga creación, actualización y destrucción; Rust lo invocaría por FFI o por un proceso auxiliar autenticado. `VirtualGamepadBackend` en `crates/virtual-gamepad` mantiene esa sustitución aislada del protocolo y del servidor.

Pasos pendientes para XInput:

1. Auditar y fijar una versión de HIDMaestro y su licencia.
2. Crear el adaptador al SDK para un perfil Xbox 360/Series.
3. Mapear `GamepadState` a ejes de 16 bits, triggers de 8 bits y bitmask de botones.
4. Empaquetar e instalar el driver con elevación explícita y firma adecuada.
5. Probar creación/destrucción repetida, cuatro slots, suspensión y cierre forzado.
6. Verificar en `joy.cpl`, XInput 1.4, Windows.Gaming.Input y juegos reales.

Un HID genérico puede aparecer en DirectInput sin ser reconocido por XInput, por eso RemotePad no ofrece un reemplazo falso basado en teclado.

## Seguridad

- El PIN cambia al reiniciar; el token aleatorio no aparece en la URL.
- El servidor conserva solamente SHA-256 del token en memoria y lo vence tras 12 horas.
- La autenticación ocurre como primer mensaje del WebSocket.
- Hay límite de 16 KiB por mensaje y 240 mensajes por segundo.
- Estados con ejes fuera de rango, tipos inválidos y secuencias antiguas se rechazan/ignoran.
- Sólo existe un controlador activo.
- La parada roja corta la sesión y libera las entradas.
- No existe ningún mensaje para ejecutar comandos, abrir archivos ni iniciar procesos.
- CORS sólo acepta la aplicación servida por el propio servidor y orígenes de desarrollo explícitos.

La versión LAN usa HTTP/WS porque los navegadores móviles permiten Pointer Events y WebSocket en ese contexto. No publique el puerto en Internet. Para despliegue remoto se necesita HTTPS/WSS con un certificado confiable para el teléfono, rotación/revocación de dispositivos y una política de firewall más estricta.

## Pruebas y calidad

```powershell
npm test
npm run build:web
cargo clippy --workspace --all-targets -- -D warnings
```

Las pruebas cubren normalización, diagonales y dead zones; validación de rangos y mensajes malformados; reset de botones/ejes activos; y utilidades de sesión. La compilación TypeScript usa modo estricto.

## Arquitectura

```text
apps/web/                 React + TypeScript + Vite + Tailwind
  src/controllers/       Joysticks, gamepad, trackpad y teclado
  src/hooks/             Sesión WebSocket y reconexión
  src/services/          Contrato de mensajes TypeScript
crates/protocol/         Tipos Serde y validación compartida en Rust
crates/server/           Axum, emparejamiento, sesión y archivos web
crates/input-controller/ Adaptador seguro de Enigo
crates/virtual-gamepad/  Trait sustituible + backend diagnóstico
docs/                    Decisiones y protocolo
```

La arquitectura deja libre el camino para envolver el servidor en Tauri 2 y agregar bandeja del sistema sin mover el protocolo ni los controladores.

## Problemas conocidos

- Sin backend HID/XUSB, el modo gamepad es diagnóstico únicamente.
- El PIN se muestra en consola, no existe aún UI nativa de aprobación en la laptop.
- No hay TLS ni descubrimiento mDNS en este MVP.
- iOS puede suspender WebSocket al enviar Safari a segundo plano; al volver se reconecta.
- Algunas redes de invitados aíslan dispositivos Wi-Fi.
- El trackpad no implementa todavía gestos de zoom o arrastre prolongado.

