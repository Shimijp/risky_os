#![no_std]
#![no_main]
extern crate alloc;

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
mod kalloc;
mod task;

use ::alloc::vec::Vec;
use mmio::MMIO;
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use crate::kalloc::{map_heap, ALLOCATOR, HEAP_SIZE};
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


fn init_traps() {
    unsafe{
        asm!(
        "csrw stvec, {0}",
        "csrsi sstatus, 2",
        in(reg) asm_trap_vector as *const () as usize
        );
    }
}

fn get_memory_region(dtb: usize) -> (usize, usize) {
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

    (start_addr, size)
}

fn init_frames(start_addr: usize, size: usize) -> usize {
    let mut frames = FRAMES.lock();
    frames.init(start_addr + size);

    let frame = frames.alloc_frame().unwrap();

    drop(frames);

    frame
}

fn root_page_table(frame: usize) -> &'static mut PageTable {
    unsafe {
        let kpt_ptr = frame as *mut u64 as *mut PageTable;
        &mut (*kpt_ptr)
    }
}

fn alloc_frame() -> Option<usize> {
    let mut frames = FRAMES.lock();
    let new_frame = frames.alloc_frame();
    drop(frames);

    new_frame
}

fn map_kernel(kernel_page_table: &mut PageTable) {
    let (kernel_start, kernel_end) = get_kernel_boundaries();
    println!("kernel starts at: 0x{:x}, kernel end at: 0x{:x}", kernel_start, kernel_end);
    create_kernel_page_map(kernel_page_table, kernel_start, kernel_end, 0x10000000);
    println!("mapped kernel code");
}

fn init_allocator() {
    unsafe
        {
            ALLOCATOR.lock().init(0x4000_0000 as *mut u8, HEAP_SIZE)
        }
}

fn test_virtual_memory(test_virt_addr: u64) {
    unsafe {
        *(test_virt_addr as *mut u64) = 0xDEADBEEF;

        let val = *(test_virt_addr as *mut u64);
        println!("Successfully read from virtual memory! Value: 0x{:X}", val);
    }
}

fn test_heap() {
    let mut vec = Vec::new();
    for _ in 0..5
    {
        vec.push(5);
    }
    for i in 0 .. 5
    {
        println!("{}", vec[i]);
    }
}

fn print_boot_info(hartid: usize, dtb: usize) {
    println!("heard id is {:x}", hartid);
    println!("DTB at: {:0x}", dtb);
    println!("hello from kernel");
}

fn idle_loop() -> ! {
    loop {
        read();
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain(hartid: usize, dtb: usize) -> ! {

    //activate traps and pass the trap handler function
    init_traps();

    //init specific hardware
    init_uart();

    //init physical memory with info from dbt, will need to replace with multiple memory regions on a real machine
    let (start_addr, size) = get_memory_region(dtb);

    //now init the frames
    let frame = init_frames(start_addr, size);

    let kernel_page_table = root_page_table(frame);

    let flags = Flags::new(true, true, false, false, false);
    let new_frame = alloc_frame();

    map_kernel(kernel_page_table);


    let test_virt_addr: u64 = 0xC0000;

    kernel_page_table.map_page(test_virt_addr, new_frame.unwrap() as u64, flags);
    map_heap(kernel_page_table);
    enable_paging(frame as u64);
    init_allocator();

    test_virtual_memory(test_virt_addr);

    //unsafe { asm!("ebreak") };

    //start timer interrupts
    timer_init();

    test_heap();

    print_boot_info(hartid, dtb);

    idle_loop()
}
