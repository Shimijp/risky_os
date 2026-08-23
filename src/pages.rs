use crate::mem::FRAMES;
use core::arch::asm;

pub unsafe fn write_satp(val: u64) { unsafe {
    asm!("csrw satp, {}", in(reg) val);
    }
}

pub fn enable_paging(root_table_phys_addr: u64) {
    let ppn = root_table_phys_addr >> 12;

    let mode = 8u64 << 60;

    let satp_val = mode | ppn;

    unsafe {

        write_satp(satp_val);

        asm!("sfence.vma zero, zero");
    }
}
#[derive(Copy, Clone, Debug)]
pub struct Flags {
    pub is_valid: bool,
    pub is_read: bool,
    pub is_write: bool,
    pub is_exe: bool,
    pub is_user: bool,
}
impl Flags
{
    pub fn new(is_valid: bool,is_read: bool,is_write: bool, is_exe: bool,is_user: bool,)-> Self
    {
        Flags{
           is_valid,
            is_read,
            is_write,
            is_exe,
            is_user
        }
    }
}
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn new(phys_addr: u64, flags: Flags) -> Self {
        let mut entry_raw = 0x0000000;
        entry_raw |= (phys_addr >> 12) & 0xFFF_FFFF_FFFF;
        entry_raw = entry_raw << 10;

        let mut entry = PageTableEntry(entry_raw);
        entry.set_flags(flags);
        entry
    }
    pub fn set_flags(&mut self, flags: Flags) {
        self.set_valid(flags.is_valid);
        self.set_write(flags.is_write);
        self.set_exe(flags.is_exe);
        self.set_user(flags.is_user);
        self.set_read(flags.is_read);
    }
    pub fn is_valid(&self) -> bool {
        self.0 & 1 == 1
    }
    pub fn is_read(&self) -> bool {
        self.0 >> 1 & 1 == 1
    }
    pub fn is_write(&self) -> bool {
        self.0 >> 2 & 1 == 1
    }
    pub fn is_exe(&self) -> bool {
        self.0 >> 3 & 1 == 1
    }
    pub fn is_user(&self) -> bool {
        self.0 >> 4 & 1 == 1
    }
    pub fn is_leaf(&self) -> bool {
        self.0 & 0xe != 0
    }
    pub fn set_valid(&mut self, value: bool) {
        self.0 = (self.0 & !(1 << 0)) | ((value as u64) << 0);
    }
    pub fn set_read(&mut self, value: bool) {
        self.0 = (self.0 & !(1 << 1)) | ((value as u64) << 1);
    }
    pub fn set_write(&mut self, value: bool) {
        self.0 = (self.0 & !(1 << 2)) | ((value as u64) << 2);
    }
    pub fn set_exe(&mut self, value: bool) {
        self.0 = (self.0 & !(1 << 3)) | ((value as u64) << 3);
    }
    pub fn set_user(&mut self, value: bool) {
        self.0 = (self.0 & !(1 << 4)) | ((value as u64) << 4);
    }
    pub fn set_accessed(&mut self, value: bool) {
        self.0 = (self.0 & !(1 << 6)) | ((value as u64) << 6);
    }
    pub fn set_dirty(&mut self, value: bool) {
        self.0 = (self.0 & !(1 << 7)) | ((value as u64) << 7);
    }
    pub fn get_ppn(&self) -> u64 {
        (self.0 >> 10) & 0xFFF_FFFF_FFFF
    }
}
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    pub fn new() -> Self {
        let entry = PageTableEntry(0_u64);
        PageTable {
            entries: [entry; 512],
        }
    }
    pub fn map_page(&mut self, virt: u64, phys: u64, flags: Flags) {
        let first_index = virt >> 30 & 0x1FF;
        let mut level_3_entry = &mut self.entries[first_index as usize];
        let mut new_entry = PageTableEntry::new(phys, flags);
        new_entry.set_accessed(true);
        new_entry.set_dirty(flags.is_write);
        let table_flags = Flags {
            is_valid: true,
            is_read: false,
            is_write: false,
            is_exe: false,
            is_user: false,
        };
        if !level_3_entry.is_valid() {

            let new_frame = FRAMES.lock().alloc_frame();
            if new_frame.is_none()
            {
                panic!("oh hell nah");
            }
            let new_frame = new_frame.unwrap();

            self.entries[first_index as usize] = PageTableEntry::new(new_frame as u64, table_flags);
            level_3_entry = &mut self.entries[first_index as usize]

        }

        let phys = level_3_entry.get_ppn() << 12;
        let level_2_table_ptr = (phys as *mut u64) as *mut PageTable;
        let level_2_table = unsafe { &mut *level_2_table_ptr };
        let second_index = virt >> 21 & 0x1FF;
        let mut level_2_entry = &mut level_2_table.entries[second_index as usize];
        if !level_2_entry.is_valid() {
            let new_frame = FRAMES.lock().alloc_frame();
            if new_frame.is_none()
            {
                panic!("oh hell nah");
            }
            let new_frame = new_frame.unwrap();

            level_2_table.entries[second_index as usize] = PageTableEntry::new(new_frame as u64, table_flags);
            level_2_entry = &mut level_2_table.entries[second_index as usize]
        }

        let phys = level_2_entry.get_ppn() << 12;
        let level_1_table_ptr = (phys as *mut u64) as *mut PageTable;
        let level_1_table = unsafe { &mut *level_1_table_ptr };
        let third_index = virt >> 12 & 0x1FF;
        level_1_table.entries[third_index as usize] = new_entry;
        unsafe { asm!("sfence.vma {}, zero", in(reg) virt) };
    }
}
pub fn create_kernel_page_map(
    kernel_page_table: &mut PageTable,
    start_addr: u64,
    end_addr : u64,
    uart: u64
)
{
    let flags = Flags::new(true, true,true, true, false);

    let mut current = start_addr;
    while current < end_addr
    {

        kernel_page_table.map_page(current, current, flags);
       current += 4096;
    }
    let uart_flags = Flags::new(true, true,true, false, false);
    kernel_page_table.map_page(uart, uart, uart_flags);
}



