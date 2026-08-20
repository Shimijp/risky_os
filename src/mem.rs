use lazy_static::lazy_static;
use spin::Mutex;

unsafe extern  "C" {
    static _kernel_end: u8;
}
pub fn get_free_memory_start() -> usize {
    let end_address = unsafe { core::ptr::addr_of!(_kernel_end) as usize };

    end_address
}
pub struct UsableFrames
{
    current : usize,
    next : *mut usize
}

impl UsableFrames
{

    pub fn init(&mut self, end_addr: usize)
    {
        let addr = get_free_memory_start();
        let page_size = 4096;
        let aligned_addr = (addr + page_size - 1) & !(page_size - 1);
        self.current = aligned_addr;
        let mut current_ptr = aligned_addr;
        while current_ptr + 4096 < end_addr {

            unsafe {
                (*(current_ptr as *mut UsableFrames)).next = (current_ptr + 4096) as *mut usize
            }
            current_ptr += 4096;
        }
        unsafe {
            (*(current_ptr as *mut UsableFrames)).next = core::ptr::null_mut();
        }
    }
    pub  fn alloc_frame(&mut self) -> Option<usize>
    {
        let current_addr = self.current;
        if current_addr == 0 {return None}
        let next = unsafe{(*(self.current as *mut UsableFrames)).next};
        self.current = next as usize;
        Some(current_addr)
    }
    pub fn dealloc_frame(&mut self, frame: usize)
    {

        let frame_ptr = frame as *mut UsableFrames;
        unsafe {(*frame_ptr).next = self.current as *mut usize};


        self.current = frame

    }
}
unsafe impl Send for UsableFrames {}
lazy_static!{
    pub static ref FRAMES : Mutex<UsableFrames> = {
        let frames = UsableFrames{
            current : 0,
            next: core::ptr::null_mut()
        };
        Mutex::new(frames)
    };
}



