# Modos de Operación del Sistema de Control

Este documento explica los dos modos de operación del sistema: **Modo Real** (con hardware EtherCAT) y **Modo Simulado** (sin hardware).

---

## 📋 Tabla de Contenidos

- [Descripción General](#descripción-general)
- [Modo Real (Producción)](#modo-real-producción)
- [Modo Simulado (Desarrollo)](#modo-simulado-desarrollo)
- [Comparación de Modos](#comparación-de-modos)
- [Configuración de Red](#configuración-de-red)
- [Troubleshooting](#troubleshooting)

---

## Descripción General

El proyecto soporta dos modos de operación controlados por la feature flag `mock-machine`:

1. **Modo Real**: Usa hardware EtherCAT conectado a una interfaz Ethernet física
2. **Modo Simulado**: Simula todos los dispositivos en memoria sin hardware real

---

## Modo Real (Producción)

### Características

✅ Hardware EtherCAT real (Beckhoff, servos, etc.)  
✅ Detección automática de interfaz Ethernet  
✅ Control de dispositivos físicos  
✅ Gestión de NetworkManager  
✅ Sincronización de reloj distribuido (DC)

### Compilación y Ejecución

```bash
# Compilar en modo release (recomendado para producción)
cargo build --release

# Ejecutar
cargo run --release

# O usando el script de conveniencia
./cargo_run_linux.sh
```

### Detección Automática de Interfaz

El sistema **detecta automáticamente** la interfaz EtherCAT mediante el siguiente proceso:

#### 1. Escaneo de Interfaces

El sistema escanea todas las interfaces de red y filtra:

- ✅ **Incluye:**
  - Interfaces Ethernet físicas (tipo 1)
  - Interfaces UP y RUNNING
  - Interfaces no-loopback

- ❌ **Excluye:**
  - Interfaces inalámbricas (wlan)
  - Interfaces loopback (lo)
  - Bridges de red
  - Túneles (utun)
  - Apple Wireless Direct Link (awdl)
  - Apple Network Privacy Interface (anpi)
  - Low-latency WLAN (llw)

#### 2. Prueba de Interfaces

Para cada interfaz candidata:

1. Intenta inicializar un `MainDevice` de EtherCAT
2. Busca dispositivos esclavos conectados
3. Si encuentra dispositivos, **marca esa interfaz como EtherCAT**

#### 3. Configuración de NetworkManager

Una vez detectada la interfaz correcta:

```bash
# La interfaz EtherCAT se marca como NO GESTIONADA
nmcli dev set eth0 managed no

# Las demás interfaces permanecen GESTIONADAS
nmcli dev set eth1 managed yes
```

Esto previene que NetworkManager interfiera con el tráfico EtherCAT en tiempo real.

### Logs del Proceso

Cuando el servidor arranca en modo real, verás logs como:

```
[INFO] Discovering EtherCAT interface...
[INFO] Testing interface: eth0
[INFO] Testing interface: eth1
[INFO] Found EtherCAT Interface at: eth0
[INFO] set_interface_managed for eth0 managed was set to: no
[INFO] set_interface_managed for eth1 managed was set to: yes
[INFO] Calling setup_loop
[INFO] Successfully initialized EtherCAT network
```

### Comandos Útiles

```bash
# Ver interfaces disponibles
ip link show

# Ver estado de las interfaces
nmcli device status

# Ver configuración de una interfaz específica
nmcli device show eth0

# Restaurar gestión manual de una interfaz
sudo nmcli dev set eth0 managed yes
```

### Requisitos de Hardware

- Interfaz Ethernet física con soporte EtherCAT
- Dispositivos EtherCAT conectados (Beckhoff, servos, etc.)
- Cable Ethernet categoría 5e o superior
- Topología de red EtherCAT válida (line, star, tree)

---

## Modo Simulado (Desarrollo)

### Características

✅ **Sin hardware requerido**  
✅ Simulación completa de dispositivos  
✅ Física de servos simulada  
✅ Dispositivos seriales virtuales  
✅ Ideal para desarrollo y testing

### Compilación y Ejecución

```bash
# Compilar con la feature mock-machine
cargo build --features mock-machine

# Ejecutar
cargo run --features mock-machine

# O usando el script de auto-restart
./cargo_run_linux_auto_restart.sh
```

### Dispositivos Simulados

El modo mock inicializa automáticamente:

#### Máquinas EtherCAT Simuladas

1. **TestEL2008Machine**
   - Simula módulos Beckhoff EL2004/EL2008
   - 8 salidas digitales virtuales
   - MockDigitalOutputDevice
   - Modos: Manual, Home, Automatic

2. **ServoTestMachine**
   - LichuanSimulator (LC10E virtual)
   - Física de servo simulada:
     - Aceleración/desaceleración
     - Velocidad variable
     - Posición incremental
   - Control CiA402 completo

#### Dispositivos Seriales Simulados

1. **MockSerialDevice** (`/dev/mock-serial`)
   - Dispositivo serial genérico de prueba
   
2. **ExtruderMockSerialDevice**
   - Simula extrusora industrial
   - Control de temperatura virtual
   - Control de velocidad
   
3. **WinderMockSerialDevice**
   - Simula bobinadora
   - Control de tensión virtual
   - Control de velocidad

### Ventajas del Modo Simulado

- 🚀 **Desarrollo rápido** sin hardware
- 🧪 **Testing seguro** sin riesgo de dañar equipos
- 🔄 **Iteración rápida** en algoritmos de control
- 📊 **Debugging simplificado** con datos predecibles
- 👥 **Colaboración** sin hardware físico necesario
- 🎓 **Formación** y demostraciones

### Limitaciones

- ⚠️ La física simulada es aproximada
- ⚠️ No detecta problemas de cableado
- ⚠️ No prueba timing real de hardware
- ⚠️ No valida comunicación EtherCAT real

---

## Comparación de Modos

| Aspecto | Modo Real | Modo Simulado |
|---------|-----------|---------------|
| **Comando** | `cargo run --release` | `cargo run --features mock-machine` |
| **Hardware EtherCAT** | ✅ Requerido | ❌ No necesario |
| **Puerto Ethernet** | 🔍 Auto-detectado | ❌ No usa |
| **NetworkManager** | ⚙️ Se desactiva en interfaz EtherCAT | ✅ No afecta |
| **Dispositivos** | 🔧 Hardware real | 💾 Simulados en memoria |
| **Latencia** | ⚡ Real (~1ms) | 🎯 Predecible |
| **Física** | 🎯 Real | 📐 Aproximada |
| **Uso Principal** | 🏭 Producción, Testing HW | 💻 Desarrollo, Testing SW |
| **Riesgo** | ⚠️ Puede dañar hardware | ✅ Sin riesgo físico |
| **Requisitos** | 💰 Hardware costoso | 💻 Solo software |
| **API REST** | ✅ Misma interfaz | ✅ Misma interfaz |
| **SocketIO** | ✅ Mismo protocolo | ✅ Mismo protocolo |

---

## Configuración de Red

### Configuración Óptima de Red para EtherCAT

#### 1. BIOS/UEFI

```
- Deshabilitar Energy Efficient Ethernet (EEE)
- Deshabilitar Wake-on-LAN (WoL)
- Configurar PCI Express como Gen 2 o superior
```

#### 2. Kernel (NixOS/Linux)

```bash
# Deshabilitar offloading en la interfaz EtherCAT
sudo ethtool -K eth0 rx off tx off sg off tso off gso off gro off lro off

# Verificar configuración
sudo ethtool -k eth0
```

Esto se configura automáticamente en NixOS mediante los módulos en `/nixos/`.

#### 3. NetworkManager

El sistema automáticamente ejecuta:

```bash
# Para interfaz EtherCAT detectada
nmcli dev set eth0 managed no

# Para otras interfaces
nmcli dev set eth1 managed yes
```

#### 4. Firewall

Asegúrate de que el firewall no bloquee:

- Puerto **3000** (HTTP/REST API)
- Puerto **3000** (WebSocket/SocketIO)
- Protocolo EtherCAT (0x88A4) en interfaz específica

---

## Troubleshooting

### Problemas Comunes en Modo Real

#### "No suitable EtherCAT interface found"

**Causas:**
- No hay dispositivos EtherCAT conectados
- Cable Ethernet desconectado o defectuoso
- Dispositivos sin alimentación
- Interfaz Ethernet deshabilitada

**Solución:**
```bash
# 1. Verificar interfaces disponibles
ip link show

# 2. Verificar si la interfaz está UP
sudo ip link set eth0 up

# 3. Verificar dispositivos físicos
# - Comprobar LEDs en módulos EtherCAT
# - Verificar alimentación 24V
# - Revisar cableado

# 4. Probar manualmente con ethercrab
# El sistema ya prueba automáticamente,
# revisa los logs para ver qué interfaces se probaron
```

#### "Interface managed by NetworkManager"

**Solución:**
```bash
# Desactivar manualmente NetworkManager en la interfaz
sudo nmcli dev set eth0 managed no

# Reiniciar el servidor
cargo run --release
```

#### "Permission denied accessing network interface"

**Solución:**
```bash
# Ejecutar con capacidades de red o como root
sudo setcap cap_net_raw,cap_net_admin=eip target/release/server

# O ejecutar como root (no recomendado en producción)
sudo cargo run --release
```

### Problemas Comunes en Modo Simulado

#### "Failed to initialize mock machines"

**Causas:**
- Error en la feature flag
- Conflicto de IDs de máquina

**Solución:**
```bash
# Asegurarse de usar la feature correcta
cargo clean
cargo run --features mock-machine

# Verificar que no haya otros procesos escuchando en el puerto 3000
lsof -i :3000
```

#### "Mock devices not appearing in UI"

**Solución:**
1. Verificar logs del servidor para errores de inicialización
2. Comprobar que el frontend está conectado al WebSocket
3. Refrescar el navegador (Ctrl+Shift+R)

### Verificación de Estado

#### Script de Diagnóstico

```bash
#!/bin/bash
# diagnostic.sh - Verifica la configuración del sistema

echo "=== Interfaces de Red ==="
ip link show | grep -E "^[0-9]+:|state"

echo -e "\n=== Estado de NetworkManager ==="
nmcli device status

echo -e "\n=== Proceso del Servidor ==="
ps aux | grep -E "server|cargo" | grep -v grep

echo -e "\n=== Puerto 3000 ==="
lsof -i :3000 || echo "Puerto 3000 no está en uso"

echo -e "\n=== Configuración Ethernet ==="
for iface in $(ip link show | grep -E "^[0-9]+: (eth|enp)" | cut -d: -f2 | tr -d ' '); do
    echo "--- $iface ---"
    sudo ethtool $iface 2>/dev/null | grep -E "Speed|Duplex|Link detected"
done
```

Ejecutar:
```bash
chmod +x diagnostic.sh
./diagnostic.sh
```

---

## Testing de la API

Ambos modos exponen la **misma API REST** en `http://localhost:3000`.

### Probar en Modo Real

```bash
# Iniciar servidor
cargo run --release

# En otra terminal, probar API
curl -X POST http://localhost:3000/api/v1/machine/mutate \
  -H "Content-Type: application/json" \
  -d '{
    "machine_identification_unique": "test_el2008_machine",
    "data": {
      "action": "SetLed",
      "value": { "index": 0, "on": true }
    }
  }'
```

### Probar en Modo Simulado

```bash
# Iniciar servidor en modo mock
cargo run --features mock-machine

# La API es idéntica
curl -X POST http://localhost:3000/api/v1/machine/mutate \
  -H "Content-Type: application/json" \
  -d '{
    "machine_identification_unique": "test_el2008_machine",
    "data": {
      "action": "SetLed",
      "value": { "index": 0, "on": true }
    }
  }'
```

### Archivos .http para Testing

Ver `/docs/api/` para archivos `.http` que puedes usar con la extensión REST Client de VS Code:

- `test_el2008_machine_api.http` - Control de salidas digitales
- Más archivos disponibles para otras máquinas

---

## Desarrollo Recomendado

### Workflow Sugerido

1. **Desarrollo inicial** → Modo Simulado
   ```bash
   cargo run --features mock-machine
   ```

2. **Testing de algoritmos** → Modo Simulado
   - Validar lógica de control
   - Probar casos extremos
   - Debugging sin riesgos

3. **Validación de hardware** → Modo Real
   ```bash
   cargo run --release
   ```
   - Verificar timing real
   - Validar comunicación EtherCAT
   - Probar con hardware específico

4. **Producción** → Modo Real
   - Despliegue en NixOS
   - Systemd service
   - Logs y monitoreo

### Scripts de Conveniencia

```bash
# Desarrollo con auto-restart en cambios
./cargo_run_linux_auto_restart.sh

# Producción
./cargo_run_linux.sh
```

---

## Referencias

- [Architecture Overview](architecture-overview.md) - Arquitectura general del sistema
- [EtherCAT Basics](ethercat-basics.md) - Conceptos básicos de EtherCAT
- [Control Loop](control-loop.md) - Ciclo de control del sistema
- [API Documentation](api/README.md) - Documentación de la API REST
- [Troubleshooting](troubleshooting.md) - Guía de resolución de problemas

---

## Contribuir

Al agregar nuevas máquinas, asegúrate de:

1. ✅ Implementar versión mock con `#[cfg(feature = "mock-machine")]`
2. ✅ Registrar en `machines/src/registry.rs`
3. ✅ Agregar al `mock_init.rs` si es mock
4. ✅ Documentar en este README
5. ✅ Crear archivo `.http` en `/docs/api-test/`

---

## Licencia

Ver [LICENSE](../LICENSE) en la raíz del proyecto.
