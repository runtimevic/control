# Backend API Integration - Drive Control

## Overview

Documentación de la integración entre el frontend React (DriveControl) y el backend Rust (server).

## Architecture

```
┌─────────────────────────────────────────┐
│  React Frontend (Electron)              │
│  - DriveControl.tsx                     │
│  - useDriveConnection.ts                │
└──────────────┬──────────────────────────┘
               │
               │ HTTP/WebSocket
               │
┌──────────────▼──────────────────────────┐
│  Rust Backend (server)                  │
│  - REST API (/api/servo/*)              │
│  - Socket.io event handlers             │
│  - ServoAdapter<T>                      │
└──────────────┬──────────────────────────┘
               │
               │ EtherCAT
               │
┌──────────────▼──────────────────────────┐
│  Physical Servo/Drive                   │
│  - Lichuan LC10E (0x766:0x402)          │
│  - Mitsubishi MR-J4-TM (0xA1E:0x201)    │
└─────────────────────────────────────────┘
```

## REST API Endpoints

### GET /api/servo/:id/state

Obtiene el estado actual del servo.

**Request:**
```
GET /api/servo/axis1/state
```

**Response:**
```json
{
  "name": "Axis 1",
  "position": 123.4567,
  "setpointPosition": 123.4000,
  "velocity": 10.5,
  "errorCode": 0,
  "lagError": {
    "current": 0.0123,
    "min": -0.005,
    "max": 0.015
  },
  "actualVelocity": 10.2,
  "setpointVelocity": 10.0,
  "override": 10000,
  "outputPercent": 45.5,
  "controllerOutputPercent": 42.3,
  "ready": true,
  "calibrated": true,
  "hasJob": false,
  "notMoving": false,
  "movingForward": true,
  "movingBackward": false,
  "coupledMode": false,
  "inTargetPos": false,
  "inPosRange": true,
  "controllerEnabled": true,
  "feedFwEnabled": true,
  "feedBwEnabled": true
}
```

### POST /api/servo/:id/command

Envía un comando al servo.

**Request:**
```json
POST /api/servo/axis1/command
{
  "command": "jog_start",
  "params": {
    "direction": "fast+"
  }
}
```

**Response:**
```json
{
  "success": true,
  "timestamp": 1706630400000
}
```

### Comandos Disponibles

#### 1. jog_start
```json
{
  "command": "jog_start",
  "params": {
    "direction": "fast+" | "slow+" | "slow-" | "fast-"
  }
}
```

#### 2. jog_stop
```json
{
  "command": "jog_stop"
}
```

#### 3. start
```json
{
  "command": "start",
  "params": {
    "mode": "Absolute" | "Relative" | "JogPos" | ...,
    "targetPosition": 100.0,
    "targetVelocity": 50.0,
    "acceleration": 100.0,
    "deceleration": 100.0,
    "jerk": 1000.0
  }
}
```

#### 4. stop
```json
{
  "command": "stop"
}
```

#### 5. reset
```json
{
  "command": "reset"
}
```

#### 6. reference
```json
{
  "command": "reference"
}
```

#### 7. set_enabling
```json
{
  "command": "set_enabling",
  "params": {
    "controller": true,
    "feedFw": true,
    "feedBw": true,
    "override": 10000
  }
}
```

#### 8. download_config
```json
{
  "command": "download_kv_factor",
  "params": { "value": 1.5 }
}

{
  "command": "download_ref_velocity",
  "params": { "value": 100.0 }
}

{
  "command": "download_target_position",
  "params": { "value": 250.0 }
}
```

#### 9. raw_output
```json
{
  "command": "raw_output_start",
  "params": {
    "mode": "Torque" | "Velocity" | "Position",
    "value": 50.0
  }
}

{
  "command": "raw_output_stop"
}
```

#### 10. set_position
```json
{
  "command": "set_actual_position",
  "params": {
    "mode": "Absolute" | "Relative",
    "value": 100.0
  }
}

{
  "command": "set_target_position",
  "params": {
    "mode": "Absolute" | "Relative" | "Home",
    "value": 200.0
  }
}
```

## WebSocket Events (Optional)

Para actualizaciones en tiempo real más eficientes que polling HTTP:

### Server → Client

#### servo:state
Broadcast del estado cada 100ms (o cuando cambia)
```json
{
  "axisId": "axis1",
  "state": { /* DriveState completo */ }
}
```

#### servo:error
Notificación de errores
```json
{
  "axisId": "axis1",
  "errorCode": 8192,
  "errorMessage": "Following error",
  "timestamp": 1706630400000
}
```

### Client → Server

#### servo:subscribe
Suscribirse a updates de un eje
```json
{
  "axisId": "axis1"
}
```

#### servo:unsubscribe
Desuscribirse de updates
```json
{
  "axisId": "axis1"
}
```

## Implementation in Rust Backend

### 1. Crear módulo `servo_api.rs`

```rust
// server/src/api/servo_api.rs

use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize)]
pub struct DriveState {
    pub name: String,
    pub position: f64,
    pub setpoint_position: f64,
    pub velocity: f64,
    pub error_code: u16,
    // ... resto de campos
}

#[derive(Debug, Deserialize)]
pub struct ServoCommand {
    pub command: String,
    pub params: Option<serde_json::Value>,
}

pub async fn get_servo_state(
    path: web::Path<String>,
    app_state: web::Data<Arc<RwLock<AppState>>>,
) -> Result<HttpResponse> {
    let axis_id = path.into_inner();
    let state = app_state.read().await;
    
    // Obtener estado del ServoAdapter
    if let Some(servo) = state.servos.get(&axis_id) {
        let drive_state = servo.get_state().await?;
        Ok(HttpResponse::Ok().json(drive_state))
    } else {
        Ok(HttpResponse::NotFound().json(json!({
            "error": "Axis not found"
        })))
    }
}

pub async fn send_servo_command(
    path: web::Path<String>,
    command: web::Json<ServoCommand>,
    app_state: web::Data<Arc<RwLock<AppState>>>,
) -> Result<HttpResponse> {
    let axis_id = path.into_inner();
    let mut state = app_state.write().await;
    
    if let Some(servo) = state.servos.get_mut(&axis_id) {
        match command.command.as_str() {
            "jog_start" => {
                // Implementar JOG
                servo.jog_start(/* params */).await?;
            }
            "start" => {
                // Implementar START con modo
                servo.start_movement(/* params */).await?;
            }
            // ... resto de comandos
            _ => {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "error": "Unknown command"
                })));
            }
        }
        
        Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "timestamp": chrono::Utc::now().timestamp_millis()
        })))
    } else {
        Ok(HttpResponse::NotFound().json(json!({
            "error": "Axis not found"
        })))
    }
}
```

### 2. Registrar rutas en `main.rs`

```rust
// server/src/main.rs

use actix_web::{App, HttpServer, web};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/api/servo/{id}/state", web::get().to(get_servo_state))
            .route("/api/servo/{id}/command", web::post().to(send_servo_command))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### 3. Integrar con ServoAdapter

```rust
// Ejemplo de cómo usar ServoAdapter con la API

impl AppState {
    pub async fn initialize_servos(&mut self) -> Result<()> {
        // Lichuan LC10E
        let lichuan = LichuanServo::new(/* config */);
        let adapter = ServoAdapter::new(lichuan);
        self.servos.insert("axis1".to_string(), adapter);
        
        // Mitsubishi MR-J4-TM
        let mitsubishi = MitsubishiServo::new(/* config */);
        let adapter = ServoAdapter::new(mitsubishi);
        self.servos.insert("axis2".to_string(), adapter);
        
        Ok(())
    }
}
```

## Frontend Integration

### Usando useDriveConnection hook

```typescript
// En tu componente
import { useDriveConnection } from '@/components/drive/useDriveConnection';

function MyDrivePage() {
  const { state, isConnected, sendCommand } = useDriveConnection({
    axisName: 'axis1',
    updateInterval: 100,
  });

  const handleJogStart = async (direction: string) => {
    await sendCommand('jog_start', { direction });
  };

  return (
    <div>
      {isConnected ? (
        <DriveControl 
          initialState={state} 
          onCommand={sendCommand}
        />
      ) : (
        <p>Connecting...</p>
      )}
    </div>
  );
}
```

## Testing

### 1. Mock Server

Crear servidor mock para desarrollo:

```bash
cd server
cargo run --bin mock-servo-server
```

### 2. Frontend con datos reales

```typescript
// Cambiar en useDriveConnection.ts
const response = await fetch(`http://localhost:8080/api/servo/${axisName}/state`);
const data = await response.json();
setState(data);
```

## Error Handling

### Frontend
```typescript
try {
  await sendCommand('start', params);
} catch (error) {
  toast.error(`Failed to start axis: ${error.message}`);
}
```

### Backend
```rust
match servo.start_movement(params).await {
    Ok(_) => HttpResponse::Ok().json(/* success */),
    Err(e) => {
        error!("Failed to start movement: {:?}", e);
        HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        }))
    }
}
```

## Next Steps

1. [ ] Implementar endpoints REST en `server/src/api/servo_api.rs`
2. [ ] Conectar endpoints con ServoAdapter
3. [ ] Implementar Socket.io handlers (opcional)
4. [ ] Actualizar `useDriveConnection.ts` para usar API real
5. [ ] Agregar manejo de errores robusto
6. [ ] Implementar reconexión automática
7. [ ] Agregar logging y métricas
8. [ ] Testing de integración
