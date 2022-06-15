use crate::arch::native::PageTable;

#[derive(Default)]
pub struct TaskFlags {
    data: u8,
}
impl TaskFlags {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_ring(&mut self, ring: u8) {
        self.data &= !0b11;
        self.data |= ring & 0b11;
    }
    pub fn get_ring(&self) -> u8 {
        self.data & 0b11
    }
}

pub struct Task {
    pub pid: usize,
    pub page_table: PageTable,
    pub flags: TaskFlags,
}
