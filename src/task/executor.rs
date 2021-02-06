use super::{Task, TaskId};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::task::{Waker, Context, Poll};
use crossbeam_queue::ArrayQueue;
use x86_64::structures::paging::frame::PhysFrame;
use x86_64::registers::control::Cr3Flags;
use x86_64::addr::VirtAddr;

#[repr(C)]
pub struct Executor {
    tasks : BTreeMap<TaskId, Task>,
    task_queue : Arc<ArrayQueue<TaskId>>,
    waker_cache : BTreeMap<TaskId, Waker>,
}

struct TaskWaker {
    task_id : TaskId,
    task_queue : Arc<ArrayQueue<TaskId>>,
}

pub struct Status {
    pub ip : VirtAddr,
    pub cs : u64,
    pub cf : u64,
    pub sp : VirtAddr,
    pub ss : u64,
    pub cr3 : (PhysFrame, Cr3Flags),
}


impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full.");
    }
}

impl Wake for TaskWaker {
    fn wake(self : Arc<Self>) {
        self.wake_task();
    }
    fn wake_by_ref(self : &Arc<Self>) {
        self.wake_task();
    }
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks : BTreeMap::new(),
            task_queue : Arc::new(ArrayQueue::new(100)),
            waker_cache : BTreeMap::new(),
        }
    }
    pub fn spawn(&mut self, task : Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("Task with same ID already in task BTreeMap.");
        }
        self.task_queue.push(task_id).expect("Executor queue full.");
    }

    pub fn run_tasks(&mut self) {
        let Self {tasks, task_queue, waker_cache} = self;

        while let Ok(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                },
                Poll::Pending => {}  
            }
        }
    }
    pub fn run(&mut self) -> ! {
        loop {
            self.run_tasks();
            self.sleep_if_idle();
        }
    }
    fn sleep_if_idle(&mut self) {
        if self.task_queue.is_empty() {
            x86_64::instructions::hlt();
        }
    }
}

pub fn next_task(t : Status) -> Status {
    t
}