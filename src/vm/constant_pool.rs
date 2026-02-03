pub struct ConstantPool {
    ints: Vec<i64>,
}

impl ConstantPool {
    pub fn new() -> Self {
        Self { ints: Vec::new() }
    }

    pub fn add_int(&mut self, v: i64) -> u32 {
        let id = self.ints.len() as u32;
        self.ints.push(v);
        id
    }

    #[inline(always)]
    pub fn get_int(&self, id: u32) -> i64 {
        self.ints
            .get(id as usize)
            .copied()
            .expect("constant pool: índice inválido")
    }

    pub fn len(&self) -> usize {
        self.ints.len()
    }
}
