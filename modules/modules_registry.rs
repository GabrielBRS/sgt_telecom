// modules/modules_registry.rs
use shared::registry::RegistroModulo;

/// O índice de TODOS os módulos. Só coleta — sem lógica, sem fiação.
pub fn registrar() -> Vec<RegistroModulo> {
    vec![
        email::registrar(),
        // sms::registrar(),
        // whatsapp::registrar(),
    ]
}