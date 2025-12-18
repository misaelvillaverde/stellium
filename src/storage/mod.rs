//! Persistent storage for natal charts

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use directories::ProjectDirs;

use crate::models::NatalChart;

/// Storage backend for natal charts
pub struct Storage {
    charts: RwLock<HashMap<String, NatalChart>>,
    storage_path: PathBuf,
}

impl Storage {
    /// Create a new storage instance
    pub fn new() -> Result<Self, String> {
        let storage_path = Self::get_storage_path()?;

        // Ensure directory exists
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create storage directory: {}", e))?;
        }

        let charts = if storage_path.exists() {
            let data = fs::read_to_string(&storage_path)
                .map_err(|e| format!("Failed to read storage file: {}", e))?;
            serde_json::from_str(&data)
                .map_err(|e| format!("Failed to parse storage file: {}", e))?
        } else {
            HashMap::new()
        };

        Ok(Self {
            charts: RwLock::new(charts),
            storage_path,
        })
    }

    /// Get the storage file path
    fn get_storage_path() -> Result<PathBuf, String> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "stellium", "stellium") {
            let data_dir = proj_dirs.data_dir();
            Ok(data_dir.join("natal_charts.json"))
        } else {
            // Fallback to current directory
            Ok(PathBuf::from("natal_charts.json"))
        }
    }

    /// Save a natal chart
    pub fn save_chart(&self, chart: NatalChart) -> Result<(), String> {
        {
            let mut charts = self
                .charts
                .write()
                .map_err(|_| "Failed to acquire write lock")?;
            charts.insert(chart.name.clone(), chart);
        }
        self.persist()?;
        Ok(())
    }

    /// Get a natal chart by name
    pub fn get_chart(&self, name: &str) -> Option<NatalChart> {
        let charts = self.charts.read().ok()?;
        charts.get(name).cloned()
    }

    /// Get the default chart (first one stored, or None)
    pub fn get_default_chart(&self) -> Option<NatalChart> {
        let charts = self.charts.read().ok()?;
        charts.values().next().cloned()
    }

    /// List all stored chart names
    pub fn list_charts(&self) -> Vec<String> {
        if let Ok(charts) = self.charts.read() {
            charts.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Delete a chart by name
    pub fn delete_chart(&self, name: &str) -> Result<bool, String> {
        let removed = {
            let mut charts = self
                .charts
                .write()
                .map_err(|_| "Failed to acquire write lock")?;
            charts.remove(name).is_some()
        };
        if removed {
            self.persist()?;
        }
        Ok(removed)
    }

    /// Persist charts to disk
    fn persist(&self) -> Result<(), String> {
        let charts = self
            .charts
            .read()
            .map_err(|_| "Failed to acquire read lock")?;
        let data = serde_json::to_string_pretty(&*charts)
            .map_err(|e| format!("Failed to serialize charts: {}", e))?;
        fs::write(&self.storage_path, data)
            .map_err(|e| format!("Failed to write storage file: {}", e))?;
        Ok(())
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new().expect("Failed to initialize storage")
    }
}
