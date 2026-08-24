use linked_list_allocator::LockedHeap;
use crate::mem::FRAMES;
use crate::pages::{Flags, PageTable};
pub const HEAP_SIZE : usize = 0x100000;
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn map_heap(kpage_table: &mut PageTable)
{
    let start: usize = 0x4000_0000;

    let mut current = start;

    let flags = Flags::new(true, true, true, false, false);
    while current < start + HEAP_SIZE
    {

        let mut frame_lock = FRAMES.lock();
        let frame = if let Some(frame_addr) = frame_lock.alloc_frame()
        {
            frame_addr
        }
        else {
            panic!("Ohh shit")
        };
        drop(frame_lock);
        kpage_table.map_page(current as u64, frame as u64, flags);
        current += 4096
    }

}