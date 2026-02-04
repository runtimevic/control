# API REST - Documentación y Testing

Esta carpeta contiene archivos `.http` para probar las APIs REST de las diferentes máquinas del sistema de control.

## Requisitos

### Extensión REST Client para VS Code

Instala la extensión **REST Client** de Huachao Mao:

```bash
code --install-extension humao.rest-client
```

O desde VS Code:
1. `Ctrl+Shift+X` para abrir extensiones
2. Buscar "REST Client"
3. Instalar la extensión de Huachao Mao

## Cómo usar

1. **Inicia el servidor backend:**
   ```bash
   cargo run --features mock-machine
   ```
   
   El servidor por defecto corre en `http://localhost:3000`

2. **Abre un archivo `.http`** de esta carpeta en VS Code

3. **Verás enlaces "Send Request"** sobre cada petición HTTP

4. **Haz clic en "Send Request"** para ejecutar la petición

5. **La respuesta aparecerá** en un panel lateral con formato JSON

## Estructura de las peticiones

Todas las peticiones siguen el formato:

```http
POST http://localhost:3000/api/v1/machine/mutate
Content-Type: application/json

{
  "machine_identification_unique": "nombre_de_la_maquina",
  "data": {
    "action": "NombreDeLaAccion",
    "value": { /* parámetros opcionales */ }
  }
}
```

## Variables

Los archivos `.http` usan variables para facilitar la configuración:

```http
### Variables
@baseUrl = http://localhost:3000
@machineId = test_el2008_machine
```

Para cambiar el puerto o el ID de la máquina, simplemente edita estas variables al inicio del archivo.

## Respuestas

### Respuesta exitosa:
```json
{
  "success": true,
  "error": null
}
```

### Respuesta con error:
```json
{
  "success": false,
  "error": "Descripción del error"
}
```

## Archivos disponibles

- **`test_el2008_machine_api.http`** - Control de salidas digitales (8 LEDs) con módulos Beckhoff EL2004/EL2008
  - Control individual de LEDs
  - Control de todos los LEDs
  - Modos: Manual, Home, Automatic
  - Configuración de delays

## Características de REST Client

### Separación de requests
Usa `###` para separar múltiples requests en el mismo archivo:

```http
### Request 1
POST {{baseUrl}}/api/v1/machine/mutate
...

### Request 2
POST {{baseUrl}}/api/v1/machine/mutate
...
```

### Variables de entorno
Puedes crear diferentes entornos (desarrollo, producción):

```http
@baseUrl = http://localhost:3000    # Desarrollo
# @baseUrl = http://192.168.1.100:3000  # Producción
```

### Comentarios
Usa `#` para comentarios en línea:

```http
# Esto es un comentario
POST {{baseUrl}}/api/v1/machine/mutate
```

### Ejecución secuencial
Puedes ejecutar múltiples requests en secuencia seleccionándolos y usando `Ctrl+Alt+R` (o `Cmd+Alt+R` en Mac)

## Tips

1. **Usa nombres descriptivos** en los separadores `###` para identificar rápidamente cada request
2. **Agrupa requests relacionados** en secciones del archivo
3. **Documenta casos de uso** con comentarios antes de cada request
4. **Versiona estos archivos** en Git para compartir con el equipo
5. **Crea secuencias de prueba** para testing automatizado

## Troubleshooting

### El servidor no responde
- Verifica que el servidor esté corriendo: `ps aux | grep cargo`
- Comprueba el puerto en `server/config.toml`
- Verifica que no haya firewall bloqueando el puerto 3000

### Error "Machine not found"
- Asegúrate de que `machine_identification_unique` coincida con el ID configurado
- Verifica que la máquina esté registrada en el servidor

### Error de sintaxis JSON
- Valida el JSON en [jsonlint.com](https://jsonlint.com)
- Verifica que todas las comillas sean dobles `"`
- Comprueba que no falten comas entre propiedades

## Recursos adicionales

- [Documentación REST Client](https://marketplace.visualstudio.com/items?itemName=humao.rest-client)
- [API Backend Documentation](../architecture-overview.md)
- [Código de las máquinas](../../machines/src/)

## Contribuir

Al agregar nuevas máquinas:

1. Crea un archivo `{nombre_maquina}_api.http` en esta carpeta
2. Documenta todas las acciones disponibles
3. Agrupa las peticiones por categorías
4. Incluye ejemplos de casos de uso comunes
5. Actualiza este README con el nuevo archivo
