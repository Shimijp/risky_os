#![no_std]
#![no_main]

unsafe extern "C" {
    static _kernel_start: u8;
    static _kernel_end: u8;
}
pub fn get_kernel_boundaries() -> (u64, u64) {
    let start_addr = unsafe { core::ptr::addr_of!(_kernel_start) as u64 };
    let end_addr = unsafe { core::ptr::addr_of!(_kernel_end) as u64 };

    (start_addr, end_addr)
}

mod interrupts;
mod mmio;
mod mem;
mod dtb;
mod pages;
mod alloc;

use mmio::MMIO;
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use crate::dtb::cpy_dtb;
use crate::interrupts::{asm_trap_vector, handle_interrupt, timer_init};
use crate::mem::FRAMES;
use crate::mmio::{init_uart, read};
use crate::pages::{create_kernel_page_map, enable_paging, Flags, PageTable};

global_asm!(include_str!("asm/boot.S"));

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        core::hint::spin_loop();
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn kmain(hartid: usize, dtb: usize) -> ! {

    //activate traps and pass the trap handler function
    unsafe{
        asm!(
        "csrw stvec, {0}",
        "csrsi sstatus, 2",
        in(reg) asm_trap_vector as *const () as usize
        );
    }

    //init specific hardware
    init_uart();

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
    frames.init(start_addr + size);

    let frame = frames.alloc_frame().unwrap();


    drop(frames);
    let kernel_page_table = unsafe {
        let kpt_ptr = frame as *mut u64 as *mut PageTable;
        &mut (*kpt_ptr)
    };

    let flags = Flags::new(true, true, false, false, false);
    let mut frames = FRAMES.lock();
    let new_frame = frames.alloc_frame();
    drop(frames);

    let (kernel_start, kernel_end) = get_kernel_boundaries();
    println!("kernel starts at: 0x{:x}, kernel end at: 0x{:x}", kernel_start, kernel_end);
    create_kernel_page_map(kernel_page_table, kernel_start, kernel_end, 0x10000000);
    println!("mapped kernel code");


    let test_virt_addr: u64 = 0xC0000;

    kernel_page_table.map_page(test_virt_addr, new_frame.unwrap() as u64, flags);

    enable_paging(frame as u64);

    unsafe {
        *(test_virt_addr as *mut u64) = 0xDEADBEEF;

        let val = *(test_virt_addr as *mut u64);
        println!("Successfully read from virtual memory! Value: 0x{:X}", val);
    }











    unsafe { asm!("ebreak") };

    //start timer interrupts
    timer_init();



    println!("heard id is {:x}", hartid);
    println!("DTB at: {:0x}", dtb);
    println!("hello from kernel");

    loop {
        read();
        core::hint::spin_loop();
    }
}
