#![no_std]
#![no_main]


mod interrupts;
mod mmio;
mod mem;
mod dtb;
mod pages;

use mmio::MMIO;
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use crate::dtb::cpy_dtb;
use crate::interrupts::{asm_trap_vector, handle_interrupt, timer_init};
use crate::mem::FRAMES;
use crate::mmio::{init_uart, read};

//pub static mut DTB : [u8;8192] = [0;8192];
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



    //init physical memory with info from dbt, will need to replace with multiple memory regions on a real machine
    let dbt: [u8; 8192] = [0; 8192];
    let dbt_ptr = &raw const dbt;
    cpy_dtb(dbt_ptr as *mut u8, dtb);
    let fdt = fdt::Fdt::new(&dbt)
        .expect("Failed to parse valid FDT header");

    let mem_region =  fdt.memory()
        .regions()
        .next()
        .unwrap();
    let start_addr = mem_region.starting_address.addr();
    let size = mem_region.size
        .unwrap();
    println!("mem starts at : {:0x}, and with total size of: {}", start_addr, size);

    //now init the frames
    let mut frames = FRAMES.lock();
    //will need to replace with calculated addr instead of hard coded
    frames.init(start_addr+size);

    if let Some(frame) = frames.alloc_frame()
    {
        let frame_addr = frame as *mut u8;
        unsafe {*frame_addr = 0x77};
        let check = unsafe {*frame_addr};
        println!("wrote byte to :0x{:0x}, value: 0x{:0x}", frame, check);
    }



    //activate traps and pass the trap handler function
    unsafe{
        asm!(
        "csrw stvec, {0}",
        "csrsi sstatus, 2",
        in(reg) asm_trap_vector as *const () as usize
        );
    }
    unsafe { asm!("ebreak") };

    //init specific hardware
    init_uart();
    timer_init();



    println!("heard id is {:x}", hartid);
    println!("DTB at: {:0x}", dtb);
    println!("hello from kernel");

    loop {
        read();
        core::hint::spin_loop();
    }
}
