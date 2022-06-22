use crate::arch::{CpuException, Error, InterruptHandler};
use crate::InterruptTable;

pub trait PagingManagerTrait {
    type PageTable: PageTableTrait;
    fn set_page_table(page_table: &Self::PageTable) -> Result<(), Error>;
    fn get_page_table<'a>() -> Result<&'a mut Self::PageTable, Error>;
}

pub trait PageTableTrait {
    fn map_page(&mut self, virtual_addr: usize, physical_addr: usize);
    fn unmap_page(&mut self, virtual_addr: usize);
    fn get_physical_addr(&self, virtual_addr: usize) -> Option<usize>;
}

pub trait UtilTrait {
    fn init() -> Result<(), Error>;
    fn halt_loop() -> !;
}

pub trait InterruptInfoTrait {
    fn interrupt_num(&self) -> usize;
    fn error_code(&self) -> usize;
}

pub trait InterruptTableTrait {
    /// # Safety
    /// Dereferences the `handler` pointer in order to make a `NonNull` reference
    unsafe fn set_interrupt_handler(
        &mut self,
        interrupt_num: usize,
        handler: *const dyn FnMut(Option<usize>, usize, &'static mut usize),
    );
    fn new() -> Self;
}

pub trait InterruptManagerTrait {
    type InterruptTable: InterruptTableTrait;
    fn set_interrupt_table(interrupt_table: &mut Self::InterruptTable) -> Result<(), Error>;
    fn get_interrupt_table<'a>() -> Result<&'a mut Self::InterruptTable, Error>;
    fn enable_interrupts();
}

pub trait ExceptionHandlerTrait {
    fn new(handler: *const dyn Fn(CpuException) -> bool) -> Self;
    fn set_handler(&mut self, handler: *const dyn Fn(CpuException) -> bool);
    fn write(&self, table: &mut InterruptTable);
}
