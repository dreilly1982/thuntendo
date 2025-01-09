pub trait Mapper {
    fn cpu_map_read(&self, addr: u16) -> u32;
    fn cpu_map_write(&self, addr: u16) -> u32;
    fn ppu_map_read(&self, addr: u16) -> u32;
    fn ppu_map_write(&self, addr: u16) -> u32;
}