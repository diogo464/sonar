use crate::{external::ExternalMediaId, UserId};

#[derive(Debug, Clone)]
pub struct ExternalSubscription {
    pub user: UserId,
    pub external_id: String,
    pub external_service: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExternalSubscriptionCreate {
    pub user: UserId,
    pub external_id: String,
    pub external_service: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExternalSubscriptionDelete {
    pub user: UserId,
    pub external_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalDownloadStatus {
    Downloading,
    Complete,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ExternalDownload {
    pub external_id: ExternalMediaId,
    pub status: ExternalDownloadStatus,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ExternalDownloadRequest {
    pub user_id: UserId,
    pub external_id: ExternalMediaId,
}

#[derive(Debug, Clone)]
pub struct ExternalDownloadDelete {
    pub user_id: UserId,
    pub external_id: ExternalMediaId,
}
