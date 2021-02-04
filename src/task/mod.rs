use core::{future::Future, pin::Pin, task::{Context, Poll}};
use alloc::boxed::Box;

pub mod executor;
pub struct Task {
    future : Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future : impl Future<Output = ()> + 'static) -> Self {
        Task {future : Box::pin(future),}
    }
    fn poll(&mut self, context : &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}