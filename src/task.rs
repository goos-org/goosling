use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::cell::{Cell, UnsafeCell};
use core::future::Future;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};
use static_assertions::const_assert_eq;

#[cfg(target_pointer_width = "64")]
#[repr(C, packed)]
pub struct TaskId {
    core_id: u16,
    reserved: u32,
    index: u16,
}
#[cfg(target_pointer_width = "32")]
#[repr(C, packed)]
pub struct TaskId {
    core_id: u16,
    index: u16,
}
impl From<usize> for TaskId {
    fn from(value: usize) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}
impl From<TaskId> for usize {
    fn from(value: TaskId) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}

pub trait Awaitable {
    fn finished(&self) -> bool;
}

pub enum TaskStatus {
    Running,
    Waiting(Box<dyn Awaitable>),
}

pub struct Task {
    pub id: TaskId,
    pub status: TaskStatus,
}

const_assert_eq!(
    core::mem::size_of::<TaskId>(),
    core::mem::size_of::<usize>()
);

pub struct Scheduler {
    processes: VecDeque<Task>,
}
