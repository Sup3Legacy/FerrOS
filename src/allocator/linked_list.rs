use super::{align_up, Locked};

use alloc::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::ptr;

/// Implements the structure of a linked list.
///
/// Here, we consider a heap allocator that is a linked list of free heap segments.
///
/// Each node holds these values :
/// * `size` - the size in bytes if the free segment
/// * `previous` - static reference to the previous memory node
/// * `next` - static reference to the next memory node
#[derive(Debug)]
struct ListNode {
    size: usize,
    previous: Option<&'static mut ListNode>,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode {
            size,
            previous: None,
            next: None,
        }
    }
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

/// Implements the structure of a memory allocator based on a linked list
///
/// It holds a single value `head` which is the first `ListNode` of the associated linked list.
#[derive(Debug)]
pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }
    /// Adds a free region to the allocator. It works by placing a new `ListNode` at the front of the allocator with the given size.
    /// TODO : add the functionnality of list simplification by merging contiguous free regions.
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());
        let mut node = ListNode::new(size);
        node.next = self.head.next.take();

        // Deuxième tentative
        /*
        let mut current = &mut self.head;
        while current.start_addr() < addr {
            if let Some(ref mut next_region) = current.next {
                node.next = Some(next_region);
                current.next = Some(&mut node);
                current = next_region;
            } else {

            }
        }
        */

        // Première tentative
        /*
        while let Some(ref mut next_region) = current.next {
            if (current).start_addr() <= addr {
                //current.next = Some();
                break;
            }
            node.next = next_region.next.take();
            //node.previous =
            current = next_region;
        }
        */
        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr)
    }
    /// # Safety
    /// TODO
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size)
    }

    /// Find the first free region in the allocator that has a size at least equal to the requested one.
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        let mut current = &mut self.head;
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            } else {
                current = current.next.as_mut().unwrap();
            }
        }
        None
    }

    /// Checks whether a given region can hold a value of given size.
    ///
    /// By first comparing `alloc_end` and `region.end_addr()`, we make sure the region has enough space for the value.
    ///
    /// Then we check whether the excess size (i.e. the memory space that would be left if the allocator would take this segment)
    /// allows to put the remaining memory space into a new free node.
    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;
        if alloc_end > region.end_addr() {
            return Err(());
        }
        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            return Err(());
        }
        Ok(alloc_start)
    }
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // perform layout adjustments
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        //     println!("{:#?}", allocator.head);

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // perform layout adjustments
        let (size, _) = LinkedListAllocator::size_align(layout);

        self.lock().add_free_region(ptr as usize, size)
    }
}
