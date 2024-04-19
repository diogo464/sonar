use crate::Genre;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ListKind {
    #[default]
    Unspecified,
    ByGenre(Genre),
}

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

    pub(crate) fn sql_display(self) -> impl std::fmt::Display {
        struct ListParamsSqlDisplay(ListParams);
        impl std::fmt::Display for ListParamsSqlDisplay {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    " LIMIT {} OFFSET {} ",
                    self.0.limit.unwrap_or(u32::MAX),
                    self.0.offset.unwrap_or(0)
                )
            }
        }
        ListParamsSqlDisplay(self)
    }
}
