use std::path::{Path, PathBuf};

use chat_core::RowID;
use sha1::{Digest, Sha1};

use crate::error::{AppError, AppResult};

pub struct ChatFile {
    pub ws_id: RowID,
    pub ext: String,
    pub hash: String,
}

impl ChatFile {
    pub fn new(ws_id: RowID, filename: &str, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        Self {
            ws_id,
            ext: filename.split('.').last().unwrap_or("txt").to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn url(&self) -> String {
        format!("/files/{}", self.hash_to_path())
    }

    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.hash_to_path())
    }

    fn hash_to_path(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!("{}/{}/{}/{}.{}", self.ws_id, part1, part2, part3, self.ext)
    }

    pub fn from_url(url: &str) -> AppResult<Self> {
        let mut parts = url.split('/');
        let Some(ws_id) = parts.next() else {
            return Err(AppError::invalid_input(
                "invalid file url format: missing ws_id",
            ));
        };

        let (hash, ext) = match (parts.next(), parts.next(), parts.next()) {
            (Some(part1), Some(part2), Some(part3)) => {
                let (part3, ext) = part3.split_once('.').unwrap_or((part3, "txt"));
                let hash = [part1, part2, part3].concat();
                (hash, ext)
            }
            _ => return Err(AppError::invalid_input("invalid file url format")),
        };

        Ok(Self {
            ws_id: ws_id.parse()?,
            ext: ext.to_string(),
            hash,
        })
    }
}
