use core::fmt::Write;
use x86_64::instructions::port::PortWriteOnly;

pub struct SerialWriter {
    port: PortWriteOnly<u8>,
}
impl SerialWriter {
    pub const fn new() -> Self {
        Self {
            port: PortWriteOnly::new(0x3F8),
        }
    }
    pub fn write(&mut self, val: &str) {
        for byte in val.bytes() {
            unsafe {
                self.port.write(byte);
            }
        }
    }
}
impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s);
        Ok(())
    }
}
