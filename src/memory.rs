use crate::arch::Cpu;
use crate::PageTable;
use alloc::collections::LinkedList;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::slice_from_raw_parts_mut;

pub static mut ALLOCATOR: Option<BitmapAllocator> = None;

pub struct BitmapAllocator {
    bitmap: *mut [u8],
    free: usize,
}
impl BitmapAllocator {
    pub fn new(bitmap: *mut u8, size: usize) -> Self {
        unsafe {
            core::ptr::write_bytes(bitmap as *mut u8, 0xFF, size as usize);
        }
        BitmapAllocator {
            bitmap: slice_from_raw_parts_mut(bitmap, size),
            free: 0,
        }
    }
    pub fn alloc(&mut self) -> Option<&mut u8> {
        let bitmap = unsafe { &mut *self.bitmap };
        for (i, item) in bitmap.iter_mut().enumerate() {
            if *item != 0xff {
                for j in 0..8 {
                    if *item & (1 << j) == 0 {
                        *item |= 1 << j;
                        self.free -= 1;
                        return Some(unsafe { &mut *(((i * 8 + j) * 4096) as *mut u8) });
                    }
                }
            }
        }
        None
    }
    pub fn free(&mut self, mut address: usize) {
        address /= 4096;
        let item = (unsafe { &mut *self.bitmap }).get_mut(address / 8).unwrap();
        if *item & (1 << (address % 8)) == 1 {
            self.free += 1;
        }
        *item &= !(1 << (address % 8));
    }
    pub fn free_range(&mut self, mut begin: usize, mut end: usize) {
        begin /= 4096;
        end /= 4096;
        self.free += end - begin;
        let begin_index = begin / 8;
        let end_index = end / 8;
        let begin_offset = begin % 8;
        let end_offset = end % 8;
        if begin_offset == 0 {
            if end_offset == 0 {
                unsafe {
                    core::ptr::write_bytes(
                        (self.bitmap as *mut u8).add(begin_index),
                        0,
                        end_index - begin_index,
                    );
                }
            } else {
                unsafe {
                    core::ptr::write_bytes(
                        (self.bitmap as *mut u8).add(begin_index),
                        0,
                        end_index - begin_index - 1,
                    );
                }
                *(unsafe { &mut *self.bitmap })
                    .get_mut(end_index - 1)
                    .unwrap() &= !(0xFF << end_offset);
            }
        } else if end_offset == 0 {
            *(unsafe { &mut *self.bitmap }).get_mut(begin_index).unwrap() &=
                !(0xFF >> begin_offset);
            unsafe {
                core::ptr::write_bytes(
                    (self.bitmap as *mut u8).add(begin_index + 1),
                    0,
                    end_index - begin_index,
                );
            }
        } else {
            unsafe {
                core::ptr::write_bytes(
                    (self.bitmap as *mut u8).add(begin_index + 1),
                    0,
                    end_index - begin_index - 1,
                );
            }
            *(unsafe { &mut *self.bitmap }).get_mut(end_index).unwrap() &= !(0xFF << end_offset);
        }
    }
    pub fn get_free(&self) -> usize {
        self.free * 4096
    }
    pub fn get_free_actual(&self) -> usize {
        let mut free = 0;
        for item in unsafe { &mut *self.bitmap } {
            if *item != 0xff {
                for j in 0..8 {
                    if *item & (1 << j) == 0 {
                        free += 1;
                    }
                }
            }
        }
        free * 4096
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator {};

pub struct GlobalAllocator {}
unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() > 4096 {
            panic!("alloc size too large");
        } else {
            let allocator = ALLOCATOR.as_mut().unwrap();
            allocator.alloc().unwrap()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let allocator = ALLOCATOR.as_mut().unwrap();
        allocator.free(ptr as usize);
    }
}

#[alloc_error_handler]
fn error_handler(_: Layout) -> ! {
    panic!("Allocator error");
}

pub type KernelHeap = Heap<0xFFFF_8000_0000_0000, 0xFFFF_FFFF_FFFF_FFFF>;

pub struct Heap<const BASE: usize, const LEN: usize> {
    heap_len: usize,
    allocator: BitmapAllocator,
    page_table: PageTable,
}
impl<const BASE: usize, const LEN: usize> Heap<BASE, LEN> {
    pub fn new(allocator: BitmapAllocator, page_table: PageTable) -> Self {
        Heap {
            heap_len: 0,
            allocator,
            page_table,
        }
    }
    pub fn alloc<T>(&mut self) -> Option<&mut T> {
        let begin = self.heap_len + BASE;
        let size = (core::mem::size_of::<T>() + 4095) / 4096 * 4096; // Align to a page, rounding up
        let end = begin + size;
        if end > LEN {
            return None;
        }
        self.heap_len += size;
        for i in begin / 4096..end / 4096 {
            let page = self.allocator.alloc().unwrap() as *mut _ as usize;
            self.page_table.map_page(i * 4096, page);
        }
        Some(unsafe { &mut *(begin as *mut T) })
    }
    pub fn free<T>(&mut self, addr: &mut T) {
        let begin = addr as *mut _ as usize;
        let size = (core::mem::size_of::<T>() + 4095) / 4096 * 4096; // Align to a page, rounding up
        let end = begin + size;
        if end > LEN {
            return;
        }
        for i in begin / 4096..end / 4096 {
            self.allocator
                .free(self.page_table.get_physical_addr(i * 4096).unwrap());
            self.page_table.unmap_page(i * 4096);
        }
    }
}
