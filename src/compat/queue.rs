const QUEUE_SIZE: usize = 10;

pub struct SimpleQueue<T> {
    data: [Option<T>; QUEUE_SIZE],
    read_index: usize,
    write_index: usize,
}

impl<T> SimpleQueue<T> {
    pub const fn new() -> SimpleQueue<T> {
        SimpleQueue {
            data: [None, None, None, None, None, None, None, None, None, None],
            read_index: 0,
            write_index: 0,
        }
    }

    pub fn enqueue(&mut self, e: T) {
        self.data[self.write_index] = Some(e);

        self.write_index += 1;
        self.write_index %= QUEUE_SIZE;

        if self.write_index == self.read_index {
            panic!("Queue overflow");
        }
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.write_index == self.read_index {
            None
        } else {
            let result = self.data[self.read_index].take();
            self.read_index += 1;
            self.read_index %= QUEUE_SIZE;
            result
        }
    }

    pub fn is_empty(&self) -> bool {
        self.read_index == self.write_index
    }

    pub fn is_full(&self) -> bool {
        let mut next_write = self.read_index + 1;
        next_write %= QUEUE_SIZE;

        next_write == self.read_index
    }
}
