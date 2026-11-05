//! Small helpers for the JSON files kept in the app's local data directory.

use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

/// Serializes read-modify-write cycles on the data files. Tauri runs async
/// commands on a thread pool, so two quick clicks could otherwise interleave
/// and one update would overwrite the other.
static DATA_LOCK: Mutex<()> = Mutex::new(());

pub fn lock_data() -> MutexGuard<'static, ()> {
    // A panic while holding the lock cannot leave the files half-written
    // (writes are atomic), so a poisoned lock is safe to reuse.
    DATA_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Reads a JSON file, returning `T::default()` if it does not exist yet.
///
/// A file that exists but cannot be parsed is reported as an error instead of
/// being treated as empty: silently falling back to an empty value would make
/// the next save overwrite the user's data.
pub fn read_json_or_default<T: DeserializeOwned + Default>(path: &Path) -> Result<T, String> {
    match fs::read_to_string(path) {
        Ok(content) if content.trim().is_empty() => Ok(T::default()),
        Ok(content) => serde_json::from_str(&content).map_err(|e| {
            format!(
                "Could not read {} ({}). The file has been left untouched.",
                path.display(),
                e
            )
        }),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(T::default()),
        Err(e) => Err(format!("Failed to read {}: {}", path.display(), e)),
    }
}

/// Writes `value` as pretty JSON. The data goes to a temporary file first and
/// is then renamed over the target, so a crash mid-write can't leave a
/// truncated file behind.
pub fn write_json_atomic<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
    }

    let content = serde_json::to_string_pretty(value)
        .map_err(|e| format!("Failed to serialize {}: {}", path.display(), e))?;

    let mut tmp_name = path.as_os_str().to_owned();
    tmp_name.push(".tmp");
    let tmp_path = Path::new(&tmp_name);

    fs::write(tmp_path, content)
        .map_err(|e| format!("Failed to write {}: {}", tmp_path.display(), e))?;
    fs::rename(tmp_path, path)
        .map_err(|e| format!("Failed to replace {}: {}", path.display(), e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "offline-nihongo-test-{}-{}",
            name,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn missing_file_reads_as_default() {
        let dir = temp_dir("missing");
        let value: HashMap<String, u64> = read_json_or_default(&dir.join("nope.json")).unwrap();
        assert!(value.is_empty());
    }

    #[test]
    fn round_trips_and_leaves_no_temp_file() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("nested").join("data.json");
        let mut value = HashMap::new();
        value.insert("a".to_string(), 1u64);
        write_json_atomic(&path, &value).unwrap();

        let read: HashMap<String, u64> = read_json_or_default(&path).unwrap();
        assert_eq!(read, value);
        assert!(!dir.join("nested").join("data.json.tmp").exists());
    }

    #[test]
    fn corrupted_file_is_an_error_and_is_not_modified() {
        let dir = temp_dir("corrupt");
        let path = dir.join("data.json");
        fs::write(&path, "{ not json").unwrap();

        let result: Result<HashMap<String, u64>, String> = read_json_or_default(&path);
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "{ not json");
    }
}
