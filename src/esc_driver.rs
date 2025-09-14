pub trait EscDriver {
    fn init(&mut self);
    fn reset(&mut self);
    fn write(&mut self, address: u16, buf: &[u8]);
    fn read(&mut self, address: u16, buf: &mut [u8]);
}

//TODO: create a trait for initialized driver and use it for the slave obj
