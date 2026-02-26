//! Config Dirs - утилиты директорий
//!
//! Аналог util/config.go из Go версии (часть 9: директории)

use std::path::{Path, PathBuf};
use std::fs;
use crate::error::{Error, Result};

/// Очищает директорию, удаляя все файлы и поддиректории
pub fn clear_dir<P: AsRef<Path>>(dir: P, preserve_files: bool, prefix: &str) -> Result<()> {
    let dir = dir.as_ref();
    
    if !dir.exists() {
        return Ok(());
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // Проверяем префикс если указан
        if !prefix.is_empty() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !name.starts_with(prefix) {
                    continue;
                }
            }
        }
        
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else if !preserve_files {
            fs::remove_file(&path)?;
        }
    }
    
    Ok(())
}

/// Создаёт директорию если она не существует
pub fn ensure_dir_exists<P: AsRef<Path>>(dir: P) -> Result<()> {
    let dir = dir.as_ref();
    
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    
    Ok(())
}

/// Получает временную директорию проекта
pub fn get_project_tmp_dir<P: AsRef<Path>>(tmp_base: P, project_id: i32) -> PathBuf {
    tmp_base.as_ref().join(format!("project_{}", project_id))
}

/// Очищает временную директорию проекта
pub fn clear_project_tmp_dir<P: AsRef<Path>>(tmp_base: P, project_id: i32) -> Result<()> {
    let project_tmp_dir = get_project_tmp_dir(tmp_base, project_id);
    
    if project_tmp_dir.exists() {
        fs::remove_dir_all(&project_tmp_dir)?;
    }
    
    Ok(())
}

/// Создаёт временную директорию проекта
pub fn create_project_tmp_dir<P: AsRef<Path>>(tmp_base: P, project_id: i32) -> Result<PathBuf> {
    let project_tmp_dir = get_project_tmp_dir(tmp_base, project_id);
    ensure_dir_exists(&project_tmp_dir)?;
    Ok(project_tmp_dir)
}

/// Получает или создаёт временную директорию проекта
pub fn get_or_create_project_tmp_dir<P: AsRef<Path>>(tmp_base: P, project_id: i32) -> Result<PathBuf> {
    let project_tmp_dir = get_project_tmp_dir(tmp_base, project_id);
    ensure_dir_exists(&project_tmp_dir)?;
    Ok(project_tmp_dir)
}

/// Проверяет что путь является безопасным (не выходит за пределы базовой директории)
pub fn is_safe_path<P: AsRef<Path>, B: AsRef<Path>>(path: P, base: B) -> bool {
    let path = path.as_ref();
    let base = base.as_ref();
    
    // Нормализуем пути
    if let (Ok(path_canonical), Ok(base_canonical)) = (fs::canonicalize(path), fs::canonicalize(base)) {
        path_canonical.starts_with(&base_canonical)
    } else {
        // Если не удалось канонизировать, проверяем строковое представление
        path.starts_with(base)
    }
}

/// Создаёт временную директорию с уникальным именем
pub fn create_unique_tmp_dir<P: AsRef<Path>>(base: P, prefix: &str) -> Result<PathBuf> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let base = base.as_ref();
    ensure_dir_exists(base)?;
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos();
    
    let dir_name = format!("{}_{}", prefix, timestamp);
    let dir_path = base.join(&dir_name);
    
    ensure_dir_exists(&dir_path)?;
    
    Ok(dir_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_ensure_dir_exists() {
        let temp_dir = env::temp_dir().join("test_ensure_dir");
        ensure_dir_exists(&temp_dir).unwrap();
        assert!(temp_dir.exists());
        assert!(temp_dir.is_dir());
        
        // Cleanup
        let _ = fs::remove_dir(&temp_dir);
    }

    #[test]
    fn test_get_project_tmp_dir() {
        let base = PathBuf::from("/tmp");
        let project_dir = get_project_tmp_dir(base, 123);
        assert_eq!(project_dir, PathBuf::from("/tmp/project_123"));
    }

    #[test]
    fn test_create_project_tmp_dir() {
        let temp_base = env::temp_dir().join("test_project_tmp");
        let project_dir = create_project_tmp_dir(&temp_base, 456).unwrap();
        
        assert!(project_dir.exists());
        assert_eq!(project_dir, temp_base.join("project_456"));
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_base);
    }

    #[test]
    fn test_clear_project_tmp_dir() {
        let temp_base = env::temp_dir().join("test_clear_tmp");
        let project_dir = create_project_tmp_dir(&temp_base, 789).unwrap();
        
        // Создаём тестовый файл
        let test_file = project_dir.join("test.txt");
        fs::write(&test_file, "test").unwrap();
        assert!(test_file.exists());
        
        // Очищаем
        clear_project_tmp_dir(&temp_base, 789).unwrap();
        assert!(!project_dir.exists());
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_base);
    }

    #[test]
    fn test_is_safe_path() {
        let base = PathBuf::from("/tmp");
        let safe_path = PathBuf::from("/tmp/subdir/file.txt");
        let unsafe_path = PathBuf::from("/etc/passwd");
        
        assert!(is_safe_path(&safe_path, &base));
        assert!(!is_safe_path(&unsafe_path, &base));
    }

    #[test]
    fn test_create_unique_tmp_dir() {
        let base = env::temp_dir();
        let dir = create_unique_tmp_dir(&base, "test").unwrap();
        
        assert!(dir.exists());
        assert!(dir.file_name().unwrap().to_str().unwrap().starts_with("test_"));
        
        // Cleanup
        let _ = fs::remove_dir(&dir);
    }
}
