//! Synchronous input traversal policy. Numeric native owner tokens are copied
//! opaque identities, never dereferenced. State owns no geometry or allocation.
pub const MAX_DEPTH: usize = 64;
pub const MAX_NODES: usize = 16_384;
pub const MAX_BYTES: usize = 64 * 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum TreeError {
    State = 1,
    Depth = 2,
    Nodes = 3,
    Bytes = 4,
    Cycle = 5,
    Identity = 6,
    Stack = 7,
}
#[repr(C)]
pub struct Gate {
    pub magic: u32,
    pub failure: u32,
    pub depth: usize,
    pub nodes: usize,
    pub bytes: usize,
    pub path: [u64; MAX_DEPTH],
}
impl Default for Gate {
    fn default() -> Self {
        Self::new()
    }
}
impl Gate {
    pub fn new() -> Self {
        Self {
            magic: 0x4f4d3954,
            failure: 0,
            depth: 0,
            nodes: 0,
            bytes: 0,
            path: [0; MAX_DEPTH],
        }
    }
    fn ready(&self) -> Result<(), TreeError> {
        if self.magic != 0x4f4d3954
            || self.depth > MAX_DEPTH
            || self.nodes > MAX_NODES
            || self.nodes < self.depth
            || self.bytes > MAX_BYTES
        {
            return Err(TreeError::State);
        }
        if self.path[self.depth..].iter().any(|&id| id != 0)
            || self.path[..self.depth]
                .iter()
                .enumerate()
                .any(|(i, &id)| id == 0 || self.path[..i].contains(&id))
        {
            return Err(TreeError::State);
        }
        match self.failure {
            0 => Ok(()),
            1 => Err(TreeError::State),
            2 => Err(TreeError::Depth),
            3 => Err(TreeError::Nodes),
            4 => Err(TreeError::Bytes),
            5 => Err(TreeError::Cycle),
            6 => Err(TreeError::Identity),
            7 => Err(TreeError::Stack),
            _ => Err(TreeError::State),
        }
    }
    fn reject(&mut self, error: TreeError) -> Result<(), TreeError> {
        self.failure = error as u32;
        Err(error)
    }
    pub fn enter(&mut self, identity: u64) -> Result<(), TreeError> {
        self.ready()?;
        if identity == 0 {
            return self.reject(TreeError::Identity);
        }
        if self.path[..self.depth].contains(&identity) {
            return self.reject(TreeError::Cycle);
        }
        if self.depth == MAX_DEPTH {
            return self.reject(TreeError::Depth);
        }
        if self.nodes == MAX_NODES {
            return self.reject(TreeError::Nodes);
        }
        self.path[self.depth] = identity;
        self.depth += 1;
        self.nodes += 1;
        Ok(())
    }
    pub fn leave(&mut self, identity: u64) -> Result<(), TreeError> {
        self.ready()?;
        if self.depth == 0 || self.path[self.depth - 1] != identity {
            return self.reject(TreeError::Stack);
        }
        self.depth -= 1;
        self.path[self.depth] = 0;
        Ok(())
    }
    /// Reserve no nodes: reject an impossible immediate child loop before the
    /// native caller reads it. Each actual enter accounts one visited owner.
    pub fn children(&mut self, count: usize) -> Result<(), TreeError> {
        self.ready()?;
        if self.nodes.checked_add(count).is_none_or(|v| v > MAX_NODES) {
            return self.reject(TreeError::Nodes);
        }
        Ok(())
    }
    /// Logical extracted NURBS payload, not a global allocator budget.
    pub fn account(&mut self, bytes: usize) -> Result<(), TreeError> {
        self.ready()?;
        match self.bytes.checked_add(bytes) {
            Some(total) if total <= MAX_BYTES => {
                self.bytes = total;
                Ok(())
            }
            _ => self.reject(TreeError::Bytes),
        }
    }
    pub fn finished(&mut self) -> Result<(), TreeError> {
        self.ready()?;
        if self.depth != 0 {
            return self.reject(TreeError::Stack);
        }
        Ok(())
    }
}
