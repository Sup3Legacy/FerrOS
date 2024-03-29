//! Here we define a basic bounded queue (fixed size) supporting creation, push and pop

/// Maximam size of a queue
const MAX_SIZE: usize = 32;

/// Queue error.
///
/// Possible meanings :
/// * Overflow: This means the program attempted to push an element
/// into a full `Queue`
/// * Underflow : This means the program attempted to pop an element
/// from an empty `Queue`
#[derive(Debug)]
pub enum Error {
    Overflow,
    Underflow,
}

/// Bounded queue abstraction
///
/// It has O(1) `push` and `pop` operations
pub struct Queue<T> {
    data: [Option<T>; MAX_SIZE],
    pushing: usize, // the next element will be put in data[min]
    poping: usize,  // the next element to pop is in data[max]
    empty: bool,    // to distinguish empty from full
}
impl<T> Queue<T>
where
    Option<T>: Copy,
{
    /// Returns a new bounded queue, freshly initialized
    pub const fn new() -> Self {
        Queue {
            data: [None; MAX_SIZE],
            pushing: 0,
            poping: 0,
            empty: true,
        }
    }

    pub fn is_empty(&self) -> bool {
        //! Returns true iff the queue is empty
        self.empty
    }

    pub fn is_full(&self) -> bool {
        //! Returns true iff the queue is full
        !self.is_empty() && self.poping == self.pushing
    }

    pub fn push(&mut self, elt: T) -> Result<(), Error> {
        //! Adds `elt` to the queue. Will return `Err(Overflow)` if the queue is full
        if self.is_full() {
            Err(Error::Overflow)
        } else {
            self.data[self.pushing] = Some(elt);
            self.pushing = (self.pushing + 1) % MAX_SIZE;
            self.empty = false;
            Ok(())
        }
    }

    pub fn pop(&mut self) -> Result<T, Error> {
        //! Removes and returns the last element of the queue. Will return `Err(Underflow)` if the queue is empty.
        if self.is_empty() {
            Err(Error::Underflow)
        } else {
            let res = self.data[self.poping].unwrap();
            self.data[self.poping] = None;
            self.poping = (self.poping + 1) % MAX_SIZE;
            if self.poping == self.pushing {
                self.empty = true;
            }
            Ok(res)
        }
    }
}
impl<T> Default for Queue<T>
where
    Option<T>: Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

/*
#[cfg(test)]
mod test {
    #[test]
    fn queue_creation_ok() -> Queue<usize> {
        Queue();
    }
    #[test]
    fn queue_underflow() {
        let queue: Queue<usize> = Queue();
        assert_eq!(queue.pop(), Err(Error::Underflow));
    }
    fn queue_overflow() {
        let queue: Queue<usize> = Queue();
        for i in (0..MAX_SIZE - 1) {
            queue.push(0).unwrap();
        }
        assert_eq!(queue.push(0), Err(Error::Overflow));
    }
    fn queue_push_pop() {
        let queue: Queue<usize> = Queue();
        queue.push(0);
        assert_eq!(0, queue.pop());
    }
}
*/
