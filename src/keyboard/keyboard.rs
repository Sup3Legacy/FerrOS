use alloc::collections::BTreeMap;
/// simple keyboard event element for use in the keyboard queue
#[derive(Clone, Copy, Debug)]
pub struct KeyEvent {
    code : u8,
}

#[derive(Clone, Copy, Debug)]
pub struct Node<'a, T> {
    after : Option<&'a Node<'a, T>>,
    value : T
}

#[derive(Debug)]
pub struct Queue<'a, T> {
    begin : Option<Node<'a, T>>,
    end : Option<Node<'a, T>>,
    length : usize,
}

impl <'a, T> Queue<'a, T> {
    fn enqueue(&mut self, element : T) -> () {
        let new_node = Node{after : None, value : element};
        if let Some(end_node) = self.end {
            self.end = Some(new_node);
            end_node.after = Some(&new_node);
        } else {

        }
    }
}