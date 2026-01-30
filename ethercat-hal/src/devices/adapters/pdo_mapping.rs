//! Funciones de mapeo entre BitSlice (PDO) y estructuras de datos
//!
//! Este módulo proporciona conversiones entre el formato BitSlice usado por
//! ethercrab y las estructuras de datos tipadas para servos CiA402.

use super::{Cia402RxPdo, Cia402TxPdo};
use anyhow::{Result, anyhow};
use bitvec::{order::Lsb0, slice::BitSlice, field::BitField};

/// Lee un RxPDO desde un BitSlice
///
/// # Argumentos
///
/// * `input` - BitSlice con los datos del RxPDO
///
/// # Retorna
///
/// Estructura `Cia402RxPdo` con los datos parseados
///
/// # Errores
///
/// Retorna error si el tamaño del input no coincide con el esperado
pub fn read_rx_pdo(input: &BitSlice<u8, Lsb0>) -> Result<Cia402RxPdo> {
    if input.len() < Cia402RxPdo::SIZE_BITS {
        return Err(anyhow!(
            "RxPDO input too small: {} bits, expected at least {}",
            input.len(),
            Cia402RxPdo::SIZE_BITS
        ));
    }
    
    // Convertir BitSlice a bytes usando conversión segura
    // Creamos un array temporal para leer los bytes
    let mut bytes = [0u8; Cia402RxPdo::SIZE_BYTES];
    for (i, byte) in bytes.iter_mut().enumerate() {
        if i * 8 < input.len() {
            *byte = input[i * 8..(i * 8 + 8).min(input.len())].load_le();
        }
    }
    
    let mut pdo = Cia402RxPdo::default();
    
    // Control word (offset 0, 2 bytes)
    pdo.control_word = u16::from_le_bytes([bytes[0], bytes[1]]);
    
    // Target position (offset 2, 4 bytes)
    pdo.target_position = i32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
    
    // Target velocity (offset 6, 4 bytes)
    pdo.target_velocity = i32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
    
    // Target torque (offset 10, 2 bytes)
    pdo.target_torque = i16::from_le_bytes([bytes[10], bytes[11]]);
    
    // Mode of operation (offset 12, 1 byte)
    pdo.mode_of_operation = bytes[12] as i8;
    
    Ok(pdo)
}

/// Escribe un TxPDO a un BitSlice
///
/// # Argumentos
///
/// * `output` - BitSlice mutable donde escribir los datos
/// * `pdo` - Estructura con los datos a escribir
///
/// # Errores
///
/// Retorna error si el tamaño del output no coincide con el esperado
pub fn write_tx_pdo(output: &mut BitSlice<u8, Lsb0>, pdo: &Cia402TxPdo) -> Result<()> {
    if output.len() < Cia402TxPdo::SIZE_BITS {
        return Err(anyhow!(
            "TxPDO output too small: {} bits, expected at least {}",
            output.len(),
            Cia402TxPdo::SIZE_BITS
        ));
    }
    
    // Preparar bytes para escribir
    let mut bytes = [0u8; Cia402TxPdo::SIZE_BYTES];
    
    // Status word (offset 0, 2 bytes)
    let status_bytes = pdo.status_word.to_le_bytes();
    bytes[0] = status_bytes[0];
    bytes[1] = status_bytes[1];
    
    // Position actual (offset 2, 4 bytes)
    let pos_bytes = pdo.position_actual.to_le_bytes();
    bytes[2] = pos_bytes[0];
    bytes[3] = pos_bytes[1];
    bytes[4] = pos_bytes[2];
    bytes[5] = pos_bytes[3];
    
    // Velocity actual (offset 6, 4 bytes)
    let vel_bytes = pdo.velocity_actual.to_le_bytes();
    bytes[6] = vel_bytes[0];
    bytes[7] = vel_bytes[1];
    bytes[8] = vel_bytes[2];
    bytes[9] = vel_bytes[3];
    
    // Torque actual (offset 10, 2 bytes)
    let torque_bytes = pdo.torque_actual.to_le_bytes();
    bytes[10] = torque_bytes[0];
    bytes[11] = torque_bytes[1];
    
    // Mode of operation display (offset 12, 1 byte)
    bytes[12] = pdo.mode_of_operation_display as u8;
    
    // Error code (offset 13, 2 bytes)
    let error_bytes = pdo.error_code.to_le_bytes();
    bytes[13] = error_bytes[0];
    bytes[14] = error_bytes[1];
    
    // Padding (offset 15-16)
    bytes[15] = 0;
    bytes[16] = 0;
    
    // Escribir los bytes de vuelta al BitSlice
    for (i, &byte) in bytes.iter().enumerate() {
        if i * 8 < output.len() {
            let bit_start = i * 8;
            let bit_end = (bit_start + 8).min(output.len());
            output[bit_start..bit_end].store_le(byte);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::prelude::*;

    #[test]
    fn test_rx_pdo_roundtrip() {
        let mut buffer = [0u8; Cia402RxPdo::SIZE_BYTES];
        
        // Escribir datos de prueba
        buffer[0..2].copy_from_slice(&0x1234u16.to_le_bytes());  // control_word
        buffer[2..6].copy_from_slice(&0x56789ABCi32.to_le_bytes());  // target_position
        buffer[6..10].copy_from_slice(&0x11223344i32.to_le_bytes()); // target_velocity
        buffer[10..12].copy_from_slice(&0x5678i16.to_le_bytes());    // target_torque
        buffer[12] = 8; // mode_of_operation (CSP)
        
        let bits = buffer.view_bits::<Lsb0>();
        let pdo = read_rx_pdo(bits).unwrap();
        
        assert_eq!(pdo.control_word, 0x1234);
        assert_eq!(pdo.target_position, 0x56789ABC);
        assert_eq!(pdo.target_velocity, 0x11223344);
        assert_eq!(pdo.target_torque, 0x5678);
        assert_eq!(pdo.mode_of_operation, 8);
    }

    #[test]
    fn test_tx_pdo_write() {
        let mut buffer = [0u8; Cia402TxPdo::SIZE_BYTES];
        let bits = buffer.view_bits_mut::<Lsb0>();
        
        let pdo = Cia402TxPdo {
            status_word: 0xABCD,
            position_actual: 0x12345678,
            velocity_actual: 0x9ABCDEF0u32 as i32,
            torque_actual: 0x1122,
            mode_of_operation_display: 8,
            error_code: 0x4455,
        };
        
        write_tx_pdo(bits, &pdo).unwrap();
        
        assert_eq!(u16::from_le_bytes([buffer[0], buffer[1]]), 0xABCD);
        assert_eq!(i32::from_le_bytes([buffer[2], buffer[3], buffer[4], buffer[5]]), 0x12345678);
        assert_eq!(buffer[12], 8);
    }
}
