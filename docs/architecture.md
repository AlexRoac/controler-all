# Arquitectura

El servidor Rust es el único proceso con autoridad para generar entrada local. Axum autentica y valida; `input-controller` traduce acciones explícitas a Enigo; `virtual-gamepad` consume snapshots completos para que una actualización perdida no deje estados parciales.

`VirtualGamepadBackend` expone `update` y `reset`. El backend diagnóstico es deliberadamente honesto: permite probar protocolo, secuencias y UI sin afirmar que Windows ve un mando. Un backend futuro de HIDMaestro puede vivir en el mismo crate y activarse por configuración.

La interfaz envía snapshots del gamepad como máximo una vez por frame de render. Mouse y teclado usan eventos discretos. El servidor serializa acceso a Enigo, admite un solo socket activo y reinicia el estado cuando termina la sesión.

