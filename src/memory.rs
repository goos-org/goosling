use core::num::NonZeroUsize;
use core::ops::Range;
use core::ptr::NonNull;

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

pub struct VirtualAllocator {
    first: NonNull<BuddyNode>,
    last: NonNull<BuddyNode>,
    hhdm_start: usize,
    blocks: NonNull<[Option<NonNull<[Option<BuddyNode>; 128]>>]>,
}
pub struct BuddyNode {
    prev: Option<NonNull<BuddyNode>>,
    next: Option<NonNull<BuddyNode>>,
    /// Allow niche optimization
    size: NonZeroUsize,
}
impl BuddyNode {
    pub fn join(
        &mut self,
        virtual_allocator: &mut VirtualAllocator,
        physical_allocator: &mut PhysicalAllocator,
    ) {
        let block = unsafe {
            &mut *((self.next.unwrap().as_ptr() as usize / 0x1000) as *mut [Option<BuddyNode>; 128])
        };
        let slot_index = (self.next.unwrap().as_ptr() as usize % 0x1000) / 32;
        self.next = block[slot_index].as_ref().unwrap().next;
        self.size = unsafe {
            NonZeroUsize::new_unchecked(
                self.size.get() + block[slot_index].as_ref().unwrap().size.get(),
            )
        };
        if block.iter().any(|slot| slot.is_none()) {
            let slot = unsafe {
                virtual_allocator
                    .blocks
                    .as_mut()
                    .iter_mut()
                    .filter(|slot| slot.is_none())
                    .next()
                    .unwrap()
            };
            *slot = Some(NonNull::new(block).unwrap());
        }
        block[slot_index] = None;
        physical_allocator
            .free(self.next.unwrap().as_ptr() as usize - virtual_allocator.hhdm_start);
    }
    /// Returns `None` if the node is of size 2 (the smallest possible size)
    pub fn split(
        &mut self,
        virtual_allocator: &mut VirtualAllocator,
        physical_allocator: &mut PhysicalAllocator,
    ) -> Option<NonNull<BuddyNode>> {
        if self.size.get() == 2 {
            return None;
        }
        let mut block_slot = &mut None;
        for (i, block) in unsafe { virtual_allocator.blocks.as_mut() }
            .iter_mut()
            .filter(|block| block.is_some())
            .enumerate()
        {
            for slot in unsafe { block.unwrap().as_mut().iter_mut() } {
                if slot.is_none()
                    && slot.as_ref().unwrap().size > unsafe { NonZeroUsize::new_unchecked(1) }
                {
                    block_slot = slot;
                    break;
                }
            }
            (unsafe { virtual_allocator.blocks.as_mut() })[i] = None;
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
                            as *mut [Option<BuddyNode>; 128],
                    )
                    .unwrap(),
                );
                block_slot = (unsafe {
                    virtual_allocator.blocks.as_mut()[i]
                        .as_mut()
                        .unwrap()
                        .as_mut()
                })
                .get_mut(0)
                .unwrap();
            }
        }
        *block_slot = Some(BuddyNode {
            prev: Some(NonNull::new(self).unwrap()),
            next: self.next,
            size: unsafe { NonZeroUsize::new_unchecked(self.size.get() / 2) },
        });
        self.next = Some(NonNull::from(block_slot.as_mut().unwrap()));
        self.size = block_slot.as_mut().unwrap().size;
        Some(NonNull::from(self))
    }
}
