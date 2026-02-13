# UML StateChart Examples

## Action Mappings

Los state charts ahora soportan `actionMappings` para vincular acciones genéricas a mutaciones específicas de hardware.

### Formato

```json
{
  "id": "mi-state-chart",
  "initial": "EstadoInicial",
  "actionMappings": {
    "nombreAccion": {
      "action": "NombreMutacion",
      "value": { /* parámetros */ }
    }
  },
  "states": { /* ... */ }
}
```

### Ejemplo: Traffic Light con test_el2008_machine

```json
{
  "id": "traffic-light",
  "actionMappings": {
    "activateRedLight": {
      "action": "SetLed",
      "value": {"index": 0, "on": true}
    },
    "deactivateRedLight": {
      "action": "SetLed",
      "value": {"index": 0, "on": false}
    },
    "activateYellowLight": {
      "action": "SetLed",
      "value": {"index": 1, "on": true}
    },
    "deactivateYellowLight": {
      "action": "SetLed",
      "value": {"index": 1, "on": false}
    },
    "activateGreenLight": {
      "action": "SetLed",
      "value": {"index": 2, "on": true}
    },
    "deactivateGreenLight": {
      "action": "SetLed",
      "value": {"index": 2, "on": false}
    }
  },
  "states": {
    "Red": {
      "entry": ["activateRedLight"],
      "exit": ["deactivateRedLight"],
      "on": {"TIMER": "Green"}
    }
  }
}
```

### Flujo de trabajo

1. **Configurar hardware en EtherCAT tab**:
   - Asignar cada terminal a una máquina: vendor, machine, serial, role
   - Ejemplo: EL2008 → Machine=54, Serial=1, Role=1

2. **Seleccionar máquina en StateChart tab**:
   - Dropdown muestra máquinas disponibles
   - Seleccionar "Machine 1/54/1"

3. **Importar state chart con actionMappings**:
   - Las acciones mapeadas invocarán `api_mutate()` de la máquina seleccionada
   - `activateRedLight` → llama `SetLed {index: 0, on: true}` en hardware real

4. **Ejecutar state chart**:
   - Las transiciones ejecutan acciones reales en hardware
   - Sin `machine_id`: usa acciones mock para testing
   - **Nota**: El statechart puede controlar las salidas independientemente del modo de la máquina

### Máquinas compatibles

- `test_el2008_machine`: Control de 8 LEDs (terminales EL2004/EL2008)
  - Mutaciones: `SetLed`, `SetAllLeds`, `SetMode`, `Start`, `Stop`
  - Ver: `docs/api-test/test_el2008_machine_api.http`
  - Las salidas pueden controlarse desde el statechart independientemente del modo o estado de la máquina

### Testing sin hardware

Si importas un state chart SIN seleccionar máquina (modo "Global (Testing)"):
- Las acciones se registran como mock (solo logs)
- Útil para diseñar y probar la lógica del state chart

## Troubleshooting

### Las salidas físicas no se encienden

**Problema**: Los LEDs/salidas del EL2008 no responden cuando ejecuto el statechart.

**Solución**:

1. **Verificar la selección de máquina en StateChart**:
   - En el dropdown superior del editor StateChart, debe estar seleccionada la máquina correcta (ej: "Machine 1/54/1")
   - Si está en "Global (Testing)", las acciones solo hacen logs sin controlar hardware

2. **Verificar actionMappings en el JSON**:
   - El archivo debe tener la sección `actionMappings` definida
   - Cada acción debe mapear correctamente a la mutación `SetLed` con el índice correcto

3. **Revisar logs del backend**:
   - Buscar mensajes como `[test_el2008_machine] Setting LED X to Y`
   - Si hay errores, aparecerán en los logs de Rust

4. **Verificar conexión EtherCAT**:
   - El backend debe haber detectado correctamente los dispositivos EtherCAT
   - Buscar mensajes de inicialización en los logs

**Ejemplo de configuración correcta**:
```
1. EtherCAT configurado → Machine 1/54/1 creada
2. En StateChart → Máquina seleccionada: "Machine 1/54/1"
3. Importar traffic-light.json → Run in Backend → Las salidas deberían responder
```
