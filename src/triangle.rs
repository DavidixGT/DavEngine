#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub positions: [[f32; 2]; 3],
}

impl Triangle {
    pub fn new(positions: [[f32; 2]; 3]) -> Self {
        Self { positions }
    }
}
