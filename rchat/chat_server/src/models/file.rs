use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use crate::AppError;

pub struct ChatFile {
    // extract extension from filename or mime type
    pub ws_id: i64,
    pub ext: String,
    pub hash: String,
}

impl ChatFile {
    pub fn url(&self) -> String {
        format!("/files/{}/{}", self.ws_id, self.hash_to_path())
    }

    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.hash_to_path())
    }

    // split hash to 3 parts, first 2 with 3 chars
    fn hash_to_path(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!("{}/{}/{}/{}.{}", self.ws_id, part1, part2, part3, self.ext)
    }
    pub fn new(ws_id: i64, filename: &str, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        let ext = filename.split('.').next_back().unwrap_or("txt").to_string();
        Self {
            ws_id,
            ext,
            hash: hex::encode(hash),
        }
    }
}

impl FromStr for ChatFile {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(s) = s.strip_prefix("/files/") else {
            return Err(AppError::ChatFileError(
                "Invalid chat file path".to_string()
            ));
        };
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 4 {
            return Err(AppError::ChatFileError("File path does not valid".to_string()));
        };

        let Ok(ws_id) = parts[1].parse::<i64>() else {
            return Err(AppError::ChatFileError(format!(
                "Invalid workspace id: {}",
                parts[1]
            )));
        };

        let Some((part3, ext)) = parts[3].split_once('.') else {
            return Err(AppError::ChatFileError(format!(
                "Invalid file name: {}",
                parts[3]
            )));
        };

        let hash = format!(
            "{}{}{}",
            parts[1],
            parts[2],
            part3,
        );
        Ok(Self {
            ws_id,
            ext: ext.to_string(), hash
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chat_file_new_should_work() {
        let file = ChatFile::new(1,"test.txt", b"hello");
        assert_eq!(file.ws_id, 1);
        assert_eq!(file.ext, "txt");
        assert_eq!(file.hash, "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
    }
}
