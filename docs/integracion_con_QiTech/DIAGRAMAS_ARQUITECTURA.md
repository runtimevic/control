# 🎨 Diagramas de Arquitectura - Integración

Este documento contiene diagramas visuales de la arquitectura de integración entre `rust-ethercat-devices` y `control`.

---

## 📐 Arquitectura General

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          PROYECTO CONTROL (QiTech)                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌────────────────┐         ┌──────────────────┐         ┌──────────────┐ │
│  │   Frontend     │◄───────►│     Server       │◄───────►│ Control Core │ │
│  │  (Electron +   │  Socket │  (Rust + Axum)   │   API   │   (Logic)    │ │
│  │    React)      │   IO    │                  │         │              │ │
│  └────────────────┘         └──────────┬───────┘         └──────────────┘ │
│                                        │                                   │
│                                        │ Control Loop                      │
│                                        ▼                                   │
│                             ┌──────────────────────┐                       │
│                             │   ethercat-hal       │                       │
│                             │  ┌────────────────┐  │                       │
│                             │  │ EthercatDevice │  │◄─── Trait principal  │
│                             │  └────────────────┘  │                       │
│                             │  ┌────────────────┐  │                       │
│                             │  │ PDO Management │  │                       │
│                             │  └────────────────┘  │                       │
│                             │  ┌────────────────┐  │                       │
│                             │  │ CoE Config     │  │                       │
│                             │  └────────────────┘  │                       │
│                             └──────────┬───────────┘                       │
│                                        │                                   │
│                                        │ ethercrab                         │
│                                        ▼                                   │
│                             ┌──────────────────────┐                       │
│                             │   EtherCAT Master    │                       │
│                             │   (ethercrab crate)  │                       │
│                             └──────────┬───────────┘                       │
└────────────────────────────────────────┼─────────────────────────────────┘
                                         │
                              Network    │    EtherCAT Protocol
                             Interface   │    (Layer 2 Ethernet)
                                         │
                  ╔══════════════════════╧═══════════════════════╗
                  ║         ADAPTADOR (NUEVO)                    ║
                  ╠══════════════════════════════════════════════╣
                  ║  ┌────────────────────────────────────────┐ ║
                  ║  │      ServoDeviceAdapter<T>             │ ║
                  ║  │  Implementa: EthercatDevice            │ ║
                  ║  │  Contiene: ServoAdapter<T: Servo>      │ ║
                  ║  │  - PDO Input/Output conversion         │ ║
                  ║  │  - Bit-level serialization             │ ║
                  ║  │  - CiA402 state machine                │ ║
                  ║  └────────────────────────────────────────┘ ║
                  ╚══════════════════════╤═══════════════════════╝
                                         │
                                         │ Trait Servo
                                         │
┌────────────────────────────────────────┼─────────────────────────────────┐
│                    RUST-ETHERCAT-DEVICES (Biblioteca)                    │
├────────────────────────────────────────┴─────────────────────────────────┤
│                                                                           │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                      Trait Servo (CiA402)                        │    │
│  │  + process_control_word()   + set_target_position()             │    │
│  │  + get_status_word()        + get_position_actual()             │    │
│  │  + set_mode_of_operation()  + emergency_stop()                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                           ▲           ▲           ▲                      │
│                           │           │           │                      │
│           ┌───────────────┴───┐   ┌───┴────────┐ └────────────┐         │
│           │                   │   │            │              │         │
│  ┌────────▼─────────┐  ┌──────▼────────┐  ┌───▼─────────────▼──────┐  │
│  │ LichuanSimulator │  │ MitsubishiSim │  │  BeckhoffTerminalSim   │  │
│  │  + CiA402        │  │  + CiA402     │  │  + Digital I/O         │  │
│  │  + Physics       │  │  + Physics    │  │  + Analog I/O          │  │
│  └──────────────────┘  └───────────────┘  └────────────────────────┘  │
│           │                   │                       │                 │
│           │ Feature Flag      │                       │                 │
│           ▼                   ▼                       ▼                 │
│  ┌──────────────────┐  ┌───────────────┐  ┌────────────────────────┐  │
│  │ LichuanHardware  │  │MitsubishiHW   │  │  BeckhoffTerminalHW    │  │
│  │  + ethercrab     │  │ + ethercrab   │  │  + ethercrab           │  │
│  │  + Real PDO      │  │ + Real PDO    │  │  + Real PDO            │  │
│  └──────────────────┘  └───────────────┘  └────────────────────────┘  │
│                                                                           │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                     Common Modules                               │    │
│  │  + ESI Parser      + PDO Config       + Error Handling           │    │
│  │  + CiA402 State    + Hardware Detect  + Mock/Stub Hardware       │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└───────────────────────────────────────────────────────────────────────────┘
```

---

## 🔄 Flujo de Datos: Control Loop

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         CONTROL LOOP (1ms cycle)                        │
└─────────────────────────────────────────────────────────────────────────┘

    ┌──────────────────────────────────────────────────────────────┐
    │  1. READ PHASE (Input)                                       │
    └──────────────────────────────────────────────────────────────┘
          │
          │ EtherCAT Frame (Layer 2)
          ▼
    ┌──────────────────────────────────────────────────────────────┐
    │  ethercrab::MainDevice                                       │
    │  - Receive network packets                                   │
    │  - Extract PDO data                                          │
    └───────────────────────┬──────────────────────────────────────┘
                            │ Raw bits (TxPDO from devices)
                            ▼
    ┌──────────────────────────────────────────────────────────────┐
    │  ServoDeviceAdapter::input(&mut self, bits)                  │
    │  - Deserialize bits → PDO fields                             │
    │  - txpdo.status_word = bits[0..16]                           │
    │  - txpdo.position_actual = bits[16..48]                      │
    │  - txpdo.velocity_actual = bits[48..80]                      │
    └───────────────────────┬──────────────────────────────────────┘
                            │ Structured data
                            ▼
    ┌──────────────────────────────────────────────────────────────┐
    │  Servo Internal State Update                                 │
    │  - Update CiA402 state machine                               │
    │  - Update physics simulation (if simulator)                  │
    │  - Store actual values                                       │
    └───────────────────────┬──────────────────────────────────────┘
                            │
    ┌──────────────────────────────────────────────────────────────┐
    │  2. COMPUTE PHASE                                            │
    └──────────────────────────────────────────────────────────────┘
                            │
                            │ Application logic
                            ▼
    ┌──────────────────────────────────────────────────────────────┐
    │  Machine::act()                                              │
    │  - Read current position: servo.get_position_actual()        │
    │  - Calculate control signals                                 │
    │  - Set new targets: servo.set_target_position(target)        │
    └───────────────────────┬──────────────────────────────────────┘
                            │
    ┌──────────────────────────────────────────────────────────────┐
    │  3. WRITE PHASE (Output)                                     │
    └──────────────────────────────────────────────────────────────┘
                            │
                            ▼
    ┌──────────────────────────────────────────────────────────────┐
    │  ServoDeviceAdapter::output(&self, bits)                     │
    │  - Serialize PDO fields → bits                               │
    │  - bits[0..16] = rxpdo.control_word                          │
    │  - bits[16..48] = rxpdo.target_position                      │
    │  - bits[48..80] = rxpdo.target_velocity                      │
    └───────────────────────┬──────────────────────────────────────┘
                            │ Raw bits (RxPDO to devices)
                            ▼
    ┌──────────────────────────────────────────────────────────────┐
    │  ethercrab::MainDevice                                       │
    │  - Compose network packets                                   │
    │  - Send EtherCAT frame                                       │
    └───────────────────────┬──────────────────────────────────────┘
                            │
                            │ EtherCAT Frame (Layer 2)
                            ▼
    ┌──────────────────────────────────────────────────────────────┐
    │  PHYSICAL HARDWARE or SIMULATOR                              │
    │  - Execute commands                                          │
    │  - Update internal state                                     │
    └──────────────────────────────────────────────────────────────┘
                            │
                            │ (Next cycle)
                            └────────────────► Back to READ PHASE
```

---

## 🎭 Patrón Adapter en Detalle

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              ADAPTER PATTERN                            │
└─────────────────────────────────────────────────────────────────────────┘

    Problema: Dos interfaces incompatibles
    ┌──────────────────────┐          ┌──────────────────────┐
    │   Trait Servo        │          │ Trait EthercatDevice │
    ├──────────────────────┤          ├──────────────────────┤
    │ + control_word(u16)  │    ✗     │ + input(&[bits])     │
    │ + status_word()→u16  │ Incomp   │ + output(&[bits])    │
    │ + set_position(i32)  │  atible  │ + input_len() → usize│
    │ + get_position()→i32 │          │ + output_len()→usize │
    │                      │          │                      │
    │ High-level API       │          │ Low-level bit I/O    │
    │ (CiA402 semantic)    │          │ (PDO bytes)          │
    └──────────────────────┘          └──────────────────────┘

    Solución: Adapter que traduce entre ambos
    
    ┌─────────────────────────────────────────────────────────────┐
    │            ServoDeviceAdapter<T: Servo>                     │
    ├─────────────────────────────────────────────────────────────┤
    │                                                             │
    │  Fields:                                                    │
    │  ┌───────────────────────────────────────────────────────┐ │
    │  │ servo: T  (impl Servo)                                │ │
    │  │ rxpdo: ServoPDOOutput  (master → servo)               │ │
    │  │ txpdo: ServoPDOInput   (servo → master)               │ │
    │  │ used: bool                                            │ │
    │  └───────────────────────────────────────────────────────┘ │
    │                                                             │
    │  Implements EthercatDevice:                                 │
    │  ┌───────────────────────────────────────────────────────┐ │
    │  │ fn input(&mut self, bits: &BitSlice) -> Result<()> { │ │
    │  │   // Deserialize bits into txpdo struct               │ │
    │  │   self.txpdo.status_word = read_u16(bits, 0);         │ │
    │  │   self.txpdo.position_actual = read_i32(bits, 16);    │ │
    │  │   // ... more fields ...                              │ │
    │  │ }                                                     │ │
    │  │                                                       │ │
    │  │ fn output(&self, bits: &mut BitSlice) -> Result<()> {│ │
    │  │   // Serialize rxpdo struct into bits                │ │
    │  │   write_u16(bits, 0, self.rxpdo.control_word);       │ │
    │  │   write_i32(bits, 16, self.rxpdo.target_position);   │ │
    │  │   // ... more fields ...                             │ │
    │  │ }                                                    │ │
    │  └───────────────────────────────────────────────────────┘ │
    │                                                             │
    │  Helper methods:                                            │
    │  ┌───────────────────────────────────────────────────────┐ │
    │  │ fn servo(&self) -> &T {                               │ │
    │  │   &self.servo  // Direct access to high-level API     │ │
    │  │ }                                                     │ │
    │  │                                                       │ │
    │  │ fn servo_mut(&mut self) -> &mut T {                   │ │
    │  │   &mut self.servo                                     │ │
    │  │ }                                                     │ │
    │  └───────────────────────────────────────────────────────┘ │
    └─────────────────────────────────────────────────────────────┘

    Uso desde control loop:
    
    // Low-level (EtherCAT cycle)
    adapter.input(&input_bits)?;    // Called by ethercrab
    adapter.output(&mut output_bits)?;
    
    // High-level (Application logic)
    let pos = adapter.servo().get_position_actual()?;
    adapter.servo_mut().set_target_position(target)?;
```

---

## 🔌 PDO Mapping (CiA402)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          PDO LAYOUT (96 bits = 12 bytes)                │
└─────────────────────────────────────────────────────────────────────────┘

RxPDO (Master → Servo):  Output from control, input to device
┌───────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┐
│ Byte0 │Byte1 │Byte2 │Byte3 │Byte4 │Byte5 │Byte6 │Byte7 │Byte8 │Byte9 │Byte10│Byte11│
├───────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┤
│  Control Word │   Target Position (32 bits)   │  Target Velocity (32b) │Target Torq│
│    (16 bits)  │                               │                         │  (16 bits)│
│    0x6040     │           0x607A              │        0x60FF           │   0x6071  │
└───────────────┴───────────────────────────────┴─────────────────────────┴───────────┘
     Bits 0-15         Bits 16-47                      Bits 48-79           Bits 80-95

Control Word (0x6040) bits:
  [0] Switch On
  [1] Enable Voltage
  [2] Quick Stop
  [3] Enable Operation
  [4-6] Operation Mode Specific
  [7] Fault Reset
  [8] Halt
  [9-15] Reserved


TxPDO (Servo → Master):  Input to control, output from device
┌───────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┐
│ Byte0 │Byte1 │Byte2 │Byte3 │Byte4 │Byte5 │Byte6 │Byte7 │Byte8 │Byte9 │Byte10│Byte11│
├───────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┤
│  Status Word  │   Position Actual (32 bits)   │  Velocity Actual (32b) │ Torque Act│
│   (16 bits)   │                               │                         │  (16 bits)│
│    0x6041     │           0x6064              │        0x606C           │   0x6077  │
└───────────────┴───────────────────────────────┴─────────────────────────┴───────────┘
     Bits 0-15         Bits 16-47                      Bits 48-79           Bits 80-95

Status Word (0x6041) bits:
  [0] Ready to Switch On
  [1] Switched On
  [2] Operation Enabled
  [3] Fault
  [4] Voltage Enabled
  [5] Quick Stop
  [6] Switch On Disabled
  [7-8] Reserved
  [9] Remote
  [10] Target Reached
  [11] Internal Limit Active
  [12-13] Operation Mode Specific
  [14-15] Manufacturer Specific
```

---

## 🎮 Estado de Máquinas CiA402

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    CiA402 STATE MACHINE                                 │
└─────────────────────────────────────────────────────────────────────────┘

    Power On
       │
       ▼
    ┌────────────────────────┐
    │ Not Ready to Switch On │  Initialize
    └───────────┬────────────┘
                │ Automatic
                ▼
    ┌────────────────────────┐
    │  Switch On Disabled    │◄────────────────┐
    └───────────┬────────────┘                 │
                │ Control Word: SHUTDOWN       │
                │ (0x06)                       │ Fault Reset
                ▼                              │ (0x80)
    ┌────────────────────────┐                 │
    │  Ready to Switch On    │                 │
    └───────────┬────────────┘                 │
                │ Control Word: SWITCH_ON      │
                │ (0x07)                       │
                ▼                              │
    ┌────────────────────────┐                 │
    │     Switched On        │                 │
    └───────────┬────────────┘                 │
                │ Control Word:                │
                │ ENABLE_OPERATION (0x0F)      │
                ▼                              │
    ┌────────────────────────┐                 │
    │  Operation Enabled     │  ◄─── NORMAL OPERATION
    └───────────┬────────────┘                 │
                │                              │
                │ Fault occurs                 │
                ▼                              │
    ┌────────────────────────┐                 │
    │        Fault           │─────────────────┘
    └────────────────────────┘

Control Word Transitions:
  SHUTDOWN         = 0x06  (000 0110)
  SWITCH_ON        = 0x07  (000 0111)
  ENABLE_OPERATION = 0x0F  (000 1111)
  DISABLE_VOLTAGE  = 0x00  (000 0000)
  QUICK_STOP       = 0x02  (000 0010)
  FAULT_RESET      = 0x80  (100 0000)

Status Word States:
  Not Ready        = 0x00XX (bit 0 = 0)
  Switch Disabled  = 0x0040 (bit 6 = 1)
  Ready to Switch  = 0x0021 (bits 0,5 = 1)
  Switched On      = 0x0023 (bits 0,1,5 = 1)
  Operation Enable = 0x0027 (bits 0,1,2,5 = 1)
  Fault            = 0x00X8 (bit 3 = 1)
```

---

## 🏭 Factory Pattern para Dispositivos

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      DEVICE FACTORY PATTERN                             │
└─────────────────────────────────────────────────────────────────────────┘

Entrada: DeviceIdentification
┌────────────────────────────────┐
│ VendorID: u32                  │
│ ProductCode: u32               │
│ Revision: u32                  │
│ DeviceName: String             │
└───────────────┬────────────────┘
                │
                ▼
    ┌───────────────────────────────────────┐
    │   create_device_from_identification() │
    └───────────────┬───────────────────────┘
                    │
        ┌───────────┴───────────┬───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
┌───────────────────┐  ┌────────────────────┐  ┌──────────────────┐
│ Lichuan LC10E?    │  │ Mitsubishi MR-J4?  │  │ Beckhoff EL2xxx? │
│ VID: 0x0766       │  │ VID: 0x0066        │  │ VID: 0x0002      │
│ PID: 0x0402       │  │ PID: 0x0031        │  │ PID: 0x07D43052  │
└─────────┬─────────┘  └─────────┬──────────┘  └─────────┬────────┘
          │ Yes                  │ Yes                   │ Yes
          ▼                      ▼                       ▼
┌───────────────────┐  ┌────────────────────┐  ┌──────────────────┐
│ Simulation mode?  │  │ Simulation mode?   │  │ Simulation mode? │
└─────────┬─────────┘  └─────────┬──────────┘  └─────────┬────────┘
      Yes │ No             Yes │ No                Yes │ No
    ┌─────┴─────┐        ┌─────┴─────┐         ┌─────┴─────┐
    ▼           ▼        ▼           ▼         ▼           ▼
┌───────┐ ┌─────────┐ ┌─────────┐ ┌────────┐ ┌────────┐ ┌────────┐
│Lichuan│ │Lichuan  │ │Mitsub.  │ │Mitsub. │ │Beckh.  │ │Beckh.  │
│Simula-│ │Hardware │ │Simula-  │ │Hardware│ │Simula- │ │Hardware│
│tor    │ │         │ │tor      │ │        │ │tor     │ │        │
└───┬───┘ └────┬────┘ └────┬────┘ └────┬───┘ └───┬────┘ └────┬───┘
    │          │           │           │         │           │
    └──────────┴───────────┴───────────┴─────────┴───────────┘
                            │
                            ▼
              ┌──────────────────────────────┐
              │ ServoDeviceAdapter<T: Servo> │
              │                              │
              │ or TerminalAdapter<...>      │
              └──────────────┬───────────────┘
                             │
                             ▼
              ┌──────────────────────────────┐
              │ Box<dyn EthercatDevice>      │
              │ (polymorphic)                │
              └──────────────────────────────┘
                             │
                             ▼
                   Added to device list in
                   ethercat-hal EthercatSetup
```

---

## 📊 Comparación: Simulador vs Hardware

```
┌─────────────────────────────────────────────────────────────────────────┐
│               SIMULATOR vs HARDWARE COMPARISON                          │
└─────────────────────────────────────────────────────────────────────────┘

┌──────────────────┬─────────────────────────┬─────────────────────────┐
│   Aspecto        │   Simulador             │   Hardware Real         │
├──────────────────┼─────────────────────────┼─────────────────────────┤
│ Latencia         │ ~1µs (in-memory)        │ 50-200µs (network)      │
│ Determinismo     │ 100% determinista       │ ~99% (jitter <100µs)    │
│ Física           │ Simulada (Newton)       │ Real (mecánica)         │
│ Inercia          │ Configurable            │ Real del motor          │
│ Límites          │ Software (configurable) │ Hardware (físicos)      │
│ Errores          │ Solo lógicos            │ Lógicos + físicos       │
│ Costo            │ Gratis                  │ Hardware + instalación  │
│ Setup time       │ Inmediato               │ Horas/días              │
│ Safety testing   │ Sin riesgo              │ Requiere precauciones   │
│ CI/CD            │ ✅ Perfecto             │ ❌ Impracticable        │
│ Debugging        │ ✅ Fácil (logs)         │ ⚠️ Complejo (oscilos.)  │
│ Validación final │ ❌ No suficiente        │ ✅ Requerido            │
└──────────────────┴─────────────────────────┴─────────────────────────┘

Uso recomendado:
┌─────────────────────────────────────────────────────────────────────┐
│ Fase                │ Modo Recomendado                              │
├─────────────────────┼───────────────────────────────────────────────┤
│ Desarrollo inicial  │ 100% Simulación                               │
│ Unit tests          │ 100% Simulación                               │
│ Integration tests   │ 100% Simulación                               │
│ CI/CD pipeline      │ 100% Simulación                               │
│ Manual testing      │ 80% Simulación, 20% Hardware                  │
│ Validation          │ 20% Simulación, 80% Hardware                  │
│ Production          │ 100% Hardware (simulación como fallback)      │
└─────────────────────┴───────────────────────────────────────────────┘
```

---

## 🔄 Modo Híbrido (Mix de Real y Simulado)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    HYBRID MODE ARCHITECTURE                             │
└─────────────────────────────────────────────────────────────────────────┘

    Máquina con múltiples dispositivos:
    
    ┌─────────────────────────────────────────────────────────────┐
    │                  Winding Machine                            │
    ├─────────────────────────────────────────────────────────────┤
    │                                                             │
    │  ┌────────────────────────────┐    🔌 HARDWARE REAL       │
    │  │ Servo Principal (Lichuan)  │◄─── Connected             │
    │  │ - Posición: 12500          │    Network: OK            │
    │  │ - Velocidad: 1000 rpm      │    Tested: ✅             │
    │  │ - Estado: Running          │                           │
    │  └────────────────────────────┘                           │
    │                                                             │
    │  ┌────────────────────────────┐    💻 SIMULACIÓN          │
    │  │ Servo Secundario (Mitsub.) │◄─── Not available         │
    │  │ - Posición: 8340           │    Simulated behavior     │
    │  │ - Velocidad: 800 rpm       │    For testing            │
    │  │ - Estado: Running [SIM]    │                           │
    │  └────────────────────────────┘                           │
    │                                                             │
    │  ┌────────────────────────────┐    🔌 HARDWARE REAL       │
    │  │ DI Terminal (Beckhoff)     │◄─── Connected             │
    │  │ - Inputs: [1,0,1,1,0,0,0,0]│    Sensors OK             │
    │  └────────────────────────────┘                           │
    │                                                             │
    │  ┌────────────────────────────┐    💻 SIMULACIÓN          │
    │  │ DO Terminal (Beckhoff)     │◄─── Being developed       │
    │  │ - Outputs: [1,1,0,0]       │    Virtual outputs        │
    │  └────────────────────────────┘                           │
    │                                                             │
    └─────────────────────────────────────────────────────────────┘
    
    Configuración (config.toml):
    
    [machine.winding_v2]
    
    [[machine.winding_v2.devices]]
    name = "main_servo"
    type = "lichuan_lc10e"
    force_simulation = false    # Use hardware if available
    
    [[machine.winding_v2.devices]]
    name = "secondary_servo"
    type = "mitsubishi_mrj4tm"
    force_simulation = true     # Always simulate (for testing)
    
    [[machine.winding_v2.devices]]
    name = "input_terminal"
    type = "beckhoff_el1008"
    force_simulation = false    # Use hardware
    
    [[machine.winding_v2.devices]]
    name = "output_terminal"
    type = "beckhoff_el2004"
    force_simulation = true     # In development, simulate
```

---

**Estos diagramas son complementarios al documento principal de integración.**

Ver también:
- [INTEGRACION_CON_CONTROL.md](INTEGRACION_CON_CONTROL.md) - Plan completo
- [RESUMEN_INTEGRACION.md](RESUMEN_INTEGRACION.md) - Resumen ejecutivo
