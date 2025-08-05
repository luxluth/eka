#[derive(Debug)]
pub struct Arena {
    bump: bumpalo::Bump,
    allocation_ptrs: Vec<*mut std::os::raw::c_void>,
}

impl Arena {
    pub fn new() -> Self {
        Self {
            bump: bumpalo::Bump::new(),
            allocation_ptrs: vec![],
        }
    }

    pub fn alloc<T>(&mut self, any: T) -> usize {
        use std::os::raw::c_void;
        let alloc = self.bump.alloc(any);
        let idx = self.allocation_ptrs.len();
        self.allocation_ptrs.push(alloc as *mut _ as *mut c_void);
        return idx;
    }

    pub fn get<T>(&self, index: usize) -> &mut T {
        let data_ptr = self.allocation_ptrs[index];
        unsafe { &mut *(data_ptr as *mut T) }
    }
}
