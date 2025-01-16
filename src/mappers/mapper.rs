pub trait Mapper {
    fn cpu_map_read(&self, addr: u16) -> (u32, bool);
    fn cpu_map_write(&mut self, addr: u16, data: u8) -> (u32, bool);
    fn ppu_map_read(&self, addr: u16) -> (u32, bool);
    fn ppu_map_write(&mut self, addr: u16) -> (u32, bool);
    fn reset(&mut self);
}