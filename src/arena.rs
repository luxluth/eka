use std::collections::VecDeque;

#[derive(Debug)]
pub struct Arena {
    bump: bumpalo::Bump,
    deallocs: VecDeque<usize>,
    allocation_ptrs: Vec<Option<*mut std::os::raw::c_void>>,
}

impl Arena {
    pub fn new() -> Self {
        Self {
            bump: bumpalo::Bump::new(),
            deallocs: VecDeque::new(),
            allocation_ptrs: vec![],
        }
    }

    pub fn alloc<T>(&mut self, any: T) -> usize {
        use std::os::raw::c_void;
        let alloc = self.bump.alloc(any);
        let idx = {
            if !self.deallocs.is_empty() {
                self.deallocs.pop_front().unwrap()
            } else {
                self.allocation_ptrs.len()
            }
        };
        self.allocation_ptrs
            .push(Some(alloc as *mut _ as *mut c_void));
        return idx;
    }

    #[allow(unused)]
    pub fn dealloc(&mut self, id: usize) -> bool {
        if let Some(data_ptr) = self.allocation_ptrs.get(id) {
            if data_ptr.is_some() {
                self.deallocs.push_back(id);
                self.allocation_ptrs[id] = None;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get<T>(&self, index: usize) -> Option<&mut T> {
        if let Some(data_ptr) = self.allocation_ptrs.get(index) {
            if let Some(data) = *data_ptr {
                let typed = unsafe { &mut *(data as *mut T) };
                Some(typed)
            } else {
                None
            }
        } else {
            None
        }
    }
}
