#![no_std]
#![no_main]


mod interrupts;
mod mmio;
use mmio::MMIO;
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use crate::interrupts::{asm_trap_vector, handle_interrupt};
use crate::mmio::read;

global_asm!(include_str!("asm/boot.S"));

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        core::hint::spin_loop();
    }
}

/// Entered from `_start` in S-mode.
/// `hartid` and `dtb` are whatever OpenSBI passed in a0/a1.
#[unsafe(no_mangle)]
pub extern "C" fn kmain(hartid: usize, dtb: usize) -> ! {

    unsafe{
        asm!(
            "csrw stvec, {0}",
            "li   t0, 0x02  ",
            "csrs sstatus, t0",
            in(reg) asm_trap_vector as *const () as usize
        )
    }
    unsafe { asm!("ebreak") };
    println!("heard id is {}", hartid);
    println!("dtb at: {}", dtb);
    println!("hello from kernel");

    loop {
        read();
        core::hint::spin_loop();
    }
}
