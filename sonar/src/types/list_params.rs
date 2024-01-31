#[derive(Debug, Default, Clone, Copy)]
pub struct ListParams {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

impl From<(Option<u32>, Option<u32>)> for ListParams {
    fn from((offset, limit): (Option<u32>, Option<u32>)) -> Self {
        Self { offset, limit }
    }
}

impl From<(u32, u32)> for ListParams {
    fn from((offset, limit): (u32, u32)) -> Self {
        Self {
            offset: Some(offset),
            limit: Some(limit),
        }
    }
}

impl ListParams {
    pub fn new(offset: u32, limit: u32) -> Self {
        Self {
            offset: Some(offset),
            limit: Some(limit),
        }
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub(crate) fn to_db_offset_limit(self) -> (i64, i64) {
        let limit = self.limit.map(|limit| limit as i64).unwrap_or(i64::MAX);
        let offset = self.offset.map(|offset| offset as i64).unwrap_or(0);
        (offset, limit)
    }
}
