use std::any::Any;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Allocator {
    slots: Vec<Option<Box<dyn Any>>>,
    free_list: VecDeque<usize>,
}

impl Allocator {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_list: VecDeque::new(),
        }
    }

    pub fn alloc<T: 'static + Any>(&mut self, data: T) -> usize {
        let boxed_data = Box::new(data);

        if let Some(recycled_id) = self.free_list.pop_front() {
            self.slots[recycled_id] = Some(boxed_data);
            recycled_id
        } else {
            let new_id = self.slots.len();
            self.slots.push(Some(boxed_data));
            new_id
        }
    }

    pub fn dealloc(&mut self, id: usize) -> bool {
        if let Some(slot) = self.slots.get_mut(id) {
            if slot.is_some() {
                // This is the magic. Taking the `Option` out
                // and replacing it with `None` drops the `Box<dyn Any>`,
                // which frees the memory.
                *slot = None;
                self.free_list.push_back(id);
                true
            } else {
                false // Already deallocated
            }
        } else {
            false // Invalid ID
        }
    }

    pub fn get<T: 'static + Any>(&self, id: usize) -> Option<&T> {
        if let Some(Some(boxed_data)) = self.slots.get(id) {
            // runtime type-check.
            boxed_data.downcast_ref::<T>()
        } else {
            None
        }
    }

    pub fn get_mut<T: 'static + Any>(&mut self, id: usize) -> Option<&mut T> {
        if let Some(Some(boxed_data)) = self.slots.get_mut(id) {
            // runtime type-check.
            boxed_data.downcast_mut::<T>()
        } else {
            None
        }
    }
}
