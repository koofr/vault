#[derive(Debug, Clone, PartialEq)]
pub struct NextId(pub u32);

impl NextId {
    pub fn next(&mut self) -> u32 {
        let id = self.0;

        self.0 += 1;

        id
    }
}

impl Default for NextId {
    fn default() -> Self {
        NextId(1)
    }
}
