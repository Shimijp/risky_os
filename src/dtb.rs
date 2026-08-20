

pub fn cpy_dtb(buff: *mut u8, start_addr : usize)
{
    let start_ptr =  start_addr as * const u8;
    let  size_ptr = start_ptr.wrapping_add(4);
    let size  = u32::from_be(unsafe {core::ptr::read_unaligned(size_ptr as * const u32)});
    unsafe
        {
            core::ptr::copy_nonoverlapping(start_ptr as * const u8, buff, size as usize);
        }



}