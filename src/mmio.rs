use core::fmt::{Error, Write};
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
     pub static ref  MMIO : Mutex<Uart> = {
        let   uart = Uart::new(0x1000_0000);
        Mutex::new(uart)
    };
}
pub struct Uart {
    base_address: usize,
}

impl Uart {
    pub fn new(base_address: usize) -> Self {
        Uart {
            // Since our parameter is also named the same as the member
            // variable, we can just label it by name.
            base_address
        }
    }

    fn put(&self, c : u8)
    {
        unsafe {
            (self.base_address as *mut u8).write_volatile(c);
        }
    }
    pub fn read_byte(&mut self) -> u8
    {
        unsafe {
            (self.base_address as *mut u8).read_volatile()
        }
    }
}
pub fn read()
{
    let mut  mmio = MMIO.lock();
    let byte =  mmio.read_byte();
    if byte == b'\r'
    {
        mmio.put(b'\n');
    }
    /* backspace */
    else if byte == 0x08 || byte == 127 {
        mmio.put(0x08);
        mmio.put(0x20);
        mmio.put(0x08);
        return;
    }
    mmio.put(byte);
}
pub fn init_uart()
{
    unsafe {
        *(0x1000_0000 as *mut u8) = 1;
    }
}
impl Write for Uart {
    // The trait Write expects us to write the function write_str
    // which looks like:
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.bytes() {
            if c == b'\n'
            {
                self.put(b'\r');
            }
            self.put(c);
        }
        // Return that we succeeded.
        Ok(())
    }
}
// ///////////////////////////////////
// / RUST MACROS
// ///////////////////////////////////
#[macro_export]
macro_rules! print
{
	($($args:tt)+) => ({
			use core::fmt::Write;
            let mut mmio = MMIO.lock();
			let _ = write!(mmio, $($args)+);
	});
}
#[macro_export]
macro_rules! println
{
	() => ({
		print!("\r\n")
	});
	($fmt:expr) => ({
		print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		print!(concat!($fmt, "\r\n"), $($args)+)
	});
}

