use alloc::string::String;
use crate::pages::Flags;

pub const STACK_SIZE: usize = 4096 * 4; // 16KB stack size

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TaskState {
    New,
    Ready,
    Running,
    Waiting,
    Suspended,
    Zombie,
    Terminated,
    Blocked,
}


#[repr(C)]
pub struct TaskContext {
    pub ra: u64, // x1 - Return Address
    pub sp: u64, // x2 - Stack Pointer
    pub s0: u64, // x8 / fp - Saved register / Frame pointer
    pub s1: u64, // x9 - Saved register
    pub s2: u64, // x18 - Saved registers start
    pub s3: u64, // x19
    pub s4: u64, // x20
    pub s5: u64, // x21
    pub s6: u64, // x22
    pub s7: u64, // x23
    pub s8: u64, // x24
    pub s9: u64, // x25
    pub s10: u64, // x26
    pub s11: u64, // x27 - Saved registers end
}
pub struct Task
{
    id: usize,
    name: String,
    page_table : usize,
    exit_code : i32


}