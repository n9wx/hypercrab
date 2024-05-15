use crate::arch::page_table::{PhysAddress, PhysPageNum};
use crate::constants::MEMORY_END;
use crate::println;
use alloc::collections::LinkedList;
use alloc::vec::Vec;
use spin::{Mutex, Once};

/// frame tracker that has same life times as allocated page
#[derive(Debug, Clone)]
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    #[inline(always)]
    pub fn new(ppn: PhysPageNum) -> Self {
        Self { ppn }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn)
    }
}

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;

    /// alloc n contiguous pages
    fn alloc_n_pages(&mut self, order: usize) -> Option<Vec<PhysPageNum>>;

    fn dealloc(&mut self, ppn: PhysPageNum);
}

// todo 重新实现一个buddy page allocator，使用stack的分配方式会导致最后变成以单个页帧为单位的碎片
pub struct StackFrameAllocator {
    current_ppn: usize,
    end: usize,
    recycled: LinkedList<usize>,
}

impl StackFrameAllocator {
    pub fn init(&mut self, start: PhysPageNum, end: PhysPageNum) {
        self.current_ppn = start.0;
        self.end = end.0;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current_ppn: 0,
            end: 0,
            recycled: LinkedList::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop_back() {
            Some(PhysPageNum(ppn))
        } else if self.current_ppn != self.end {
            self.current_ppn += 1;
            Some(PhysPageNum(self.current_ppn - 1))
        } else {
            None
        }
    }

    // 确保能够分配连续的物理页帧
    fn alloc_n_pages(&mut self, order: usize) -> Option<Vec<PhysPageNum>> {
        let page_nums = 1 << order;
        if (self.end - self.current_ppn) < page_nums {
            return None;
        }

        let mut ret = Vec::with_capacity(page_nums);
        for _ in 0..page_nums {
            ret.push(PhysPageNum(self.current_ppn));
            self.current_ppn += 1;
        }
        Some(ret)
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        if ppn >= self.current_ppn || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }

        self.recycled.push_back(ppn);
    }
}

pub static mut FRAME_ALLOCATOR: Once<Mutex<StackFrameAllocator>> = Once::new();

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }

    unsafe {
        FRAME_ALLOCATOR.call_once(|| {
            let mut frame_allocator = StackFrameAllocator::new();
            frame_allocator.init(
                PhysAddress(ekernel as usize).next_page_number(),
                PhysAddress(MEMORY_END).current_page_number(),
            );
            Mutex::new(frame_allocator)
        });
    }
}

pub fn frame_alloc() -> Option<FrameTracker> {
    unsafe {
        let frame_allocator_ref = FRAME_ALLOCATOR.get_mut();
        let mut frame_allocator = frame_allocator_ref.unwrap().lock();
        frame_allocator.alloc().map(FrameTracker::new)
    }
}

pub fn n_frames_alloc(order: usize) -> Option<Vec<FrameTracker>> {
    unsafe {
        let frame_allocator_ref = FRAME_ALLOCATOR.get_mut();
        let mut frame_allocator = frame_allocator_ref.unwrap().lock();
        frame_allocator.alloc_n_pages(order).map(|vec| {
            vec.into_iter()
                .map(FrameTracker::new)
                .collect::<Vec<FrameTracker>>()
        })
    }
}

pub fn frame_dealloc(ppn: PhysPageNum) {
    unsafe {
        let mut frame_allocator_ref = FRAME_ALLOCATOR.get_mut();
        let mut frame_allocator = frame_allocator_ref.unwrap().lock();
        frame_allocator.dealloc(ppn);
    }
}

#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}
