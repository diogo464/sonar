#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ByteRange {
    pub offset: Option<u64>,
    pub length: Option<u64>,
}

impl ByteRange {
    pub fn new(offset: u64, length: u64) -> Self {
        Self {
            offset: Some(offset),
            length: Some(length),
        }
    }

    pub fn with_offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_length(mut self, length: u64) -> Self {
        self.length = Some(length);
        self
    }
}
