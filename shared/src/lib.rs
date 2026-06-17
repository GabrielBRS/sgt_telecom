//! Crate shared — folha do grafo. Utils transversais + contrato de eventos.

pub fn mascarar_destino(destino: &str) -> String {
    let chars: Vec<char> = destino.chars().collect();
    if chars.len() <= 4 {
        return "*".repeat(chars.len());
    }
    let inicio: String = chars[..2].iter().collect();
    let fim: String = chars[chars.len() - 2..].iter().collect();
    format!("{inicio}{}{fim}", "*".repeat(chars.len() - 4))
}

pub trait Evento: std::fmt::Debug + Send + Sync {
    fn nome(&self) -> &'static str;
}

pub trait Handler: Send + Sync {
    fn lidar(&self, evento: &dyn Evento);
}

#[derive(Default)]
pub struct BarramentoEventos {
    handlers: Vec<Box<dyn Handler>>,
}

impl BarramentoEventos {
    pub fn novo() -> Self {
        Self { handlers: Vec::new() }
    }
    pub fn registrar(&mut self, handler: Box<dyn Handler>) {
        self.handlers.push(handler);
    }
    pub fn emitir(&self, evento: &dyn Evento) {
        println!("[evento] emitido: {}", evento.nome());
        for h in &self.handlers {
            h.lidar(evento);
        }
    }
}