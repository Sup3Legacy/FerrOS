

#[alloc_error_handler]
fn alloc_error_handler(layout : alloc::alloc::Layout) -> ! {
    panic!("Allocation error : {:?}", layout)
}