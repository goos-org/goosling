use crate::sync::Mutex;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ops::Range;
use core::ptr::{slice_from_raw_parts_mut, NonNull};
use limine::{LimineMemmapEntry, LimineMemoryMapEntryType};
use static_assertions::const_assert;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

#[global_allocator]
pub static mut GLOBAL_ALLOCATOR: HeapGlobalAlloc = HeapGlobalAlloc { val: None };

#[alloc_error_handler]
pub fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("alloc error: {:?}", layout);
}

pub struct HeapGlobalAlloc<'a> {
    pub val: Option<Mutex<UnsafeCell<Heap<'a>>>>,
}
unsafe impl Send for HeapGlobalAlloc<'_> {}
unsafe impl Sync for HeapGlobalAlloc<'_> {}

unsafe impl GlobalAlloc for HeapGlobalAlloc<'_> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let val = (*self.val.as_ref().unwrap().lock().get()).alloc(layout.size());
        val.unwrap().0 as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        (*self.val.as_ref().unwrap().lock().get()).free(ptr as usize, layout.size());
    }
}

pub struct Heap<'a> {
    physical_allocator: PhysicalAllocator,
    virtual_allocator: VirtualAllocator,
    page_table: &'a mut OffsetPageTable<'a>,
}
impl<'a> Heap<'a> {
    pub fn new(mmap: &[LimineMemmapEntry], page_table: &'a mut OffsetPageTable<'a>) -> Self {
        let hhdm_start = page_table.phys_offset().as_u64() as usize;
        let memory_len = mmap.last().unwrap().base + mmap.last().unwrap().len;
        let pa_len = memory_len / 4096 / 8;
        let mut pa_start = None;
        for entry in mmap
            .iter()
            .filter(|e| e.typ == LimineMemoryMapEntryType::Usable)
        {
            if entry.len >= pa_len {
                pa_start = Some(entry.base);
            }
        }
        if pa_start.is_none() {
            panic!("No memory available for physical allocator");
        }
        let pa_start = pa_start.unwrap();
        let bitmap = unsafe {
            core::slice::from_raw_parts_mut(
                (pa_start as usize + hhdm_start) as *mut u8,
                pa_len as usize,
            )
        };
        let bitmap = NonNull::from(bitmap);
        let mut pa = PhysicalAllocator { bitmap };
        for entry in mmap
            .iter()
            .filter(|e| e.typ != LimineMemoryMapEntryType::Usable)
        {
            pa.set_range_used(entry.base as usize..(entry.base + entry.len) as usize);
        }
        let va_blocks = (pa.alloc().unwrap() + hhdm_start)
            as *mut [Option<NonNull<[Option<BuddyNode>; 102]>>; 512];
        unsafe {
            *va_blocks = [None; 512];
        }
        let mut va_blocks = NonNull::new(va_blocks).unwrap();
        (unsafe { va_blocks.as_mut() })[0] = Some(
            NonNull::new((pa.alloc().unwrap() + hhdm_start) as *mut [Option<BuddyNode>; 102])
                .unwrap(),
        );
        const TMP: Option<BuddyNode> = None;
        *(unsafe { va_blocks.as_mut()[0].unwrap().as_mut() }) = [TMP; 102];
        (unsafe { va_blocks.as_mut()[0].unwrap().as_mut() })[0] = Some(BuddyNode {
            prev: None,
            next: None,
            size: usize::MAX - hhdm_start - memory_len as usize,
            base: hhdm_start + memory_len as usize,
            used: false,
        });
        let node = NonNull::from(
            (unsafe { va_blocks.as_mut()[0].unwrap().as_mut() })[0]
                .as_mut()
                .unwrap(),
        );
        let va = VirtualAllocator {
            first: node,
            last: node,
            hhdm_start,
            blocks: va_blocks,
        };
        Self {
            physical_allocator: pa,
            virtual_allocator: va,
            page_table,
        }
    }
    pub fn alloc(&mut self, size: usize) -> Option<(usize, usize)> {
        let virtual_addr = unsafe {
            self.virtual_allocator
                .alloc(size, &mut self.physical_allocator)?
        };
        let mut size = size.next_power_of_two();
        if size < 2 {
            size = 2;
        }
        for i in (0..size).step_by(4096) {
            let page = self.physical_allocator.alloc()?;
            let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(page as u64));
            let page_addr =
                Page::containing_address(VirtAddr::new((virtual_addr + (i * 4096)) as u64));
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            (unsafe {
                self.page_table
                    .map_to(page_addr, frame, flags, &mut self.physical_allocator)
            })
            .ok()?
            .flush();
        }
        Some((virtual_addr, size))
    }
    pub fn free(&mut self, addr: usize, size: usize) {
        unsafe {
            self.virtual_allocator
                .free(addr, &mut self.physical_allocator);
        }
        let mut size = size.next_power_of_two();
        if size < 2 {
            size = 2;
        }
        for i in (0..size).step_by(4096) {
            let page_addr =
                Page::<Size4KiB>::containing_address(VirtAddr::new((addr + (i * 4096)) as u64));
            let result = self.page_table.unmap(page_addr).unwrap();
            result.1.flush();
            self.physical_allocator
                .free(result.0.start_address().as_u64() as usize);
        }
    }
}

pub struct PhysicalAllocator {
    bitmap: NonNull<[u8]>,
}
impl PhysicalAllocator {
    pub fn alloc(&mut self) -> Option<usize> {
        for byte in unsafe { self.bitmap.as_mut() } {
            if *byte != 0xFF {
                for i in 0..8 {
                    if *byte & (1 << i) == 0 {
                        *byte |= 1 << i;
                        return Some((*byte as usize) * 8 + i);
                    }
                }
            }
        }
        None
    }
    pub fn set_used(&mut self, ptr: usize) {
        let byte = ptr / 8;
        let bit = ptr % 8;
        (unsafe { self.bitmap.as_mut() })[byte] |= 1 << bit;
    }
    pub fn set_range_used(&mut self, range: Range<usize>) {
        let start = (range.start + 7) / 8;
        let end = range.end / 8;
        unsafe {
            core::ptr::write_bytes(
                self.bitmap.as_mut().as_mut_ptr().add(start),
                0xFF,
                end - start,
            );
        }
        for i in range.start..start * 8 {
            self.set_used(i);
        }
        for i in end * 8..range.end {
            self.set_used(i);
        }
    }
    pub fn free(&mut self, ptr: usize) {
        let byte = ptr / 8;
        let bit = ptr % 8;
        (unsafe { self.bitmap.as_mut() })[byte] &= !(1 << bit);
    }
}
unsafe impl FrameAllocator<Size4KiB> for PhysicalAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.alloc()
            .map(|page| PhysFrame::containing_address(PhysAddr::new(page as u64)))
    }
}

pub struct VirtualAllocator {
    first: NonNull<BuddyNode>,
    last: NonNull<BuddyNode>,
    hhdm_start: usize,
    blocks: NonNull<[Option<NonNull<[Option<BuddyNode>; 102]>>]>,
}
impl VirtualAllocator {
    pub unsafe fn alloc(
        &mut self,
        size: usize,
        physical_allocator: &mut PhysicalAllocator,
    ) -> Option<usize> {
        let mut node = self.first.as_mut();
        let mut smallest_size = size.next_power_of_two();
        if smallest_size < 2 {
            smallest_size = 2;
        }
        loop {
            if node.used || node.size < smallest_size {
                node = node.next?.as_mut();
                continue;
            }
            while node.size > smallest_size {
                node = node.split(self, physical_allocator)?.as_mut();
            }
            return Some(node.base);
        }
    }
    pub unsafe fn free(&mut self, ptr: usize, physical_allocator: &mut PhysicalAllocator) {
        let mut node = self.first.as_mut();
        while node.base != ptr {
            node = node
                .next
                .unwrap_or_else(|| panic!("No block with address {}", ptr))
                .as_mut();
        }
        node.used = false;
        node.try_join(self, physical_allocator);
    }
}

const_assert!(core::mem::size_of::<BuddyNode>() == 40);
const_assert!(core::mem::size_of::<Option<BuddyNode>>() == 40);
pub struct BuddyNode {
    prev: Option<NonNull<BuddyNode>>,
    next: Option<NonNull<BuddyNode>>,
    size: usize,
    base: usize,
    used: bool,
}
impl BuddyNode {
    pub unsafe fn can_join(&self) -> bool {
        if let Some(next) = self.next {
            let next = next.as_ref();
            if !next.used && self.size == next.size {
                return true;
            }
        }
        false
    }
    pub unsafe fn can_join_prev(&self) -> bool {
        if let Some(prev) = self.prev {
            let prev = prev.as_ref();
            if !prev.used && self.size == prev.size {
                return true;
            }
        }
        false
    }
    pub unsafe fn try_join_prev(
        &mut self,
        virtual_allocator: &mut VirtualAllocator,
        physical_allocator: &mut PhysicalAllocator,
    ) {
        if self.can_join_prev() {
            let block = unsafe {
                &mut *((self.prev.unwrap().as_ptr() as usize / 0x1000)
                    as *mut [Option<BuddyNode>; 102])
            };
            let slot_index = (self.prev.unwrap().as_ptr() as usize % 0x1000) / 32;
            self.prev = block[slot_index].as_ref().unwrap().next;
            self.size += block[slot_index].as_ref().unwrap().size;
            if block.iter().any(|slot| slot.is_none()) {
                let slot = unsafe {
                    virtual_allocator
                        .blocks
                        .as_mut()
                        .iter_mut()
                        .find(|slot| slot.is_none())
                        .unwrap()
                };
                *slot = Some(NonNull::new(block).unwrap());
            }
            block[slot_index] = None;
            physical_allocator
                .free(self.prev.unwrap().as_ptr() as usize - virtual_allocator.hhdm_start);
        } else if let Some(mut prev) = self.prev {
            prev.as_mut()
                .try_join_prev(virtual_allocator, physical_allocator);
            if self.can_join_prev() {
                self.try_join_prev(virtual_allocator, physical_allocator);
            }
        }
    }
    pub unsafe fn try_join(
        &mut self,
        virtual_allocator: &mut VirtualAllocator,
        physical_allocator: &mut PhysicalAllocator,
    ) {
        if self.can_join() {
            let block = unsafe {
                &mut *((self.next.unwrap().as_ptr() as usize / 0x1000)
                    as *mut [Option<BuddyNode>; 102])
            };
            let slot_index = (self.next.unwrap().as_ptr() as usize % 0x1000) / 32;
            self.next = block[slot_index].as_ref().unwrap().next;
            self.size += block[slot_index].as_ref().unwrap().size;
            if block.iter().any(|slot| slot.is_none()) {
                let slot = unsafe {
                    virtual_allocator
                        .blocks
                        .as_mut()
                        .iter_mut()
                        .find(|slot| slot.is_none())
                        .unwrap()
                };
                *slot = Some(NonNull::new(block).unwrap());
            }
            block[slot_index] = None;
            physical_allocator
                .free(self.next.unwrap().as_ptr() as usize - virtual_allocator.hhdm_start);
        } else if let Some(mut next) = self.next {
            if next.as_mut().can_join() {
                next.as_mut()
                    .try_join(virtual_allocator, physical_allocator);
                if self.can_join() {
                    self.try_join(virtual_allocator, physical_allocator);
                }
            }
        } else {
            self.try_join_prev(virtual_allocator, physical_allocator);
        }
    }
    /// Returns `None` if the node is of size 2 (the smallest possible size)
    pub fn split(
        &mut self,
        virtual_allocator: &mut VirtualAllocator,
        physical_allocator: &mut PhysicalAllocator,
    ) -> Option<NonNull<BuddyNode>> {
        if self.size == 2 {
            return None;
        }
        let mut block_slot = None;
        for block in unsafe { virtual_allocator.blocks.as_mut() }
            .iter_mut()
            .filter(|block| block.is_some())
        {
            for slot in unsafe { block.unwrap().as_mut().iter_mut() } {
                if slot.is_none() {
                    block_slot = Some(slot);
                    break;
                }
            }
            if block_slot.is_some() {
                break;
            }
        }
        if block_slot.is_none() {
            for (i, _) in unsafe { virtual_allocator.blocks.as_mut() }
                .iter_mut()
                .filter(|block| block.is_none())
                .enumerate()
            {
                (unsafe { virtual_allocator.blocks.as_mut() })[i] = Some(
                    NonNull::new(
                        (physical_allocator.alloc().unwrap() + virtual_allocator.hhdm_start)
                            as *mut [Option<BuddyNode>; 102],
                    )
                    .unwrap(),
                );
                block_slot = Some(
                    (unsafe {
                        virtual_allocator.blocks.as_mut()[i]
                            .as_mut()
                            .unwrap()
                            .as_mut()
                    })
                    .get_mut(0)
                    .unwrap(),
                );
            }
        }
        let block_slot = block_slot.unwrap();
        *block_slot = Some(BuddyNode {
            prev: Some(NonNull::new(self).unwrap()),
            next: self.next,
            size: self.size / 2,
            base: self.base + self.size / 2,
            used: false,
        });
        self.next = Some(NonNull::from(block_slot.as_mut().unwrap()));
        self.size = block_slot.as_mut().unwrap().size;
        if virtual_allocator.last.as_ptr().eq(&(self as *mut _)) {
            virtual_allocator.last = self.next.unwrap();
        }
        Some(NonNull::from(self))
    }
}
