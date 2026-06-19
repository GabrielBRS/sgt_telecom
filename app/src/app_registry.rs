// app/src/app_registry.rs
use shared::registry::RegistroModulo;

/// REGISTRO consolidado da aplicação: junta todos os grupos de módulos.
/// Hoje só `modules`; amanhã soma outros grupos aqui — o Application não muda.
pub fn registrar() -> Vec<RegistroModulo> {
    modules::registrar()
}