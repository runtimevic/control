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

### Máquinas compatibles

- `test_el2008_machine`: Control de 8 LEDs (terminales EL2004/EL2008)
  - Mutaciones: `SetLed`, `SetAllLeds`, `SetMode`, `Start`, `Stop`
  - Ver: `docs/api-test/test_el2008_machine_api.http`

### Testing sin hardware

Si importas un state chart SIN seleccionar máquina (modo "Global (Testing)"):
- Las acciones se registran como mock (solo logs)
- Útil para diseñar y probar la lógica del state chart
