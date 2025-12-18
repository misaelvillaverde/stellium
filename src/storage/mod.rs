//! Persistent storage for natal charts

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use directories::ProjectDirs;

use crate::models::NatalChart;

use serde::Serialize;

/// Chart info for listing
#[derive(Debug, Clone, Serialize)]
pub struct ChartInfo {
    pub name: String,
    pub birth_date: String,
    pub birth_location: String,
}

/// Storage backend for natal charts
/// Uses composite key (name + birth_date) to prevent duplicates
pub struct Storage {
    /// Charts stored by composite key: "{name}_{birth_date}"
    charts: RwLock<HashMap<String, NatalChart>>,
    storage_path: PathBuf,
}

impl Storage {
    /// Create composite key from name and birth_date
    fn make_key(name: &str, birth_date: &str) -> String {
        format!("{}_{}", name, birth_date)
    }

    /// Create a new storage instance
    pub fn new() -> Result<Self, String> {
        let storage_path = Self::get_storage_path()?;

        // Ensure directory exists
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create storage directory: {}", e))?;
        }

        let charts: HashMap<String, NatalChart> = if storage_path.exists() {
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

    /// Save a natal chart using composite key (name + birth_date)
    pub fn save_chart(&self, chart: NatalChart) -> Result<(), String> {
        let key = Self::make_key(&chart.name, &chart.birth_date);
        {
            let mut charts = self
                .charts
                .write()
                .map_err(|_| "Failed to acquire write lock")?;
            charts.insert(key, chart);
        }
        self.persist()?;
        Ok(())
    }

    /// Get a natal chart by name (returns first match if multiple with same name)
    pub fn get_chart(&self, name: &str) -> Option<NatalChart> {
        let charts = self.charts.read().ok()?;
        // Search for any chart with matching name
        charts.values().find(|c| c.name == name).cloned()
    }

    /// Get a natal chart by exact match of name AND birth_date
    pub fn get_chart_exact(&self, name: &str, birth_date: &str) -> Option<NatalChart> {
        let charts = self.charts.read().ok()?;
        // Search by values to handle both old (name-only) and new (composite) key formats
        charts
            .values()
            .find(|c| c.name == name && c.birth_date == birth_date)
            .cloned()
    }

    /// Get the default chart (first one stored, or None)
    pub fn get_default_chart(&self) -> Option<NatalChart> {
        let charts = self.charts.read().ok()?;
        charts.values().next().cloned()
    }

    /// List all stored charts with their info
    pub fn list_charts(&self) -> Vec<ChartInfo> {
        if let Ok(charts) = self.charts.read() {
            charts
                .values()
                .map(|c| ChartInfo {
                    name: c.name.clone(),
                    birth_date: c.birth_date.clone(),
                    birth_location: c.birth_location.clone(),
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// List chart names only (for backward compatibility)
    pub fn list_chart_names(&self) -> Vec<String> {
        if let Ok(charts) = self.charts.read() {
            charts.values().map(|c| c.name.clone()).collect()
        } else {
            Vec::new()
        }
    }

    /// Search charts by name (case-insensitive partial match)
    pub fn search_charts(&self, query: &str) -> Vec<ChartInfo> {
        let query_lower = query.to_lowercase();
        if let Ok(charts) = self.charts.read() {
            charts
                .values()
                .filter(|c| c.name.to_lowercase().contains(&query_lower))
                .map(|c| ChartInfo {
                    name: c.name.clone(),
                    birth_date: c.birth_date.clone(),
                    birth_location: c.birth_location.clone(),
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Delete a chart by name (deletes first match if multiple)
    pub fn delete_chart(&self, name: &str) -> Result<bool, String> {
        let key_to_remove = {
            let charts = self
                .charts
                .read()
                .map_err(|_| "Failed to acquire read lock")?;
            charts
                .iter()
                .find(|(_, c)| c.name == name)
                .map(|(k, _)| k.clone())
        };

        if let Some(key) = key_to_remove {
            let mut charts = self
                .charts
                .write()
                .map_err(|_| "Failed to acquire write lock")?;
            charts.remove(&key);
            drop(charts);
            self.persist()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Delete a chart by exact match of name AND birth_date
    pub fn delete_chart_exact(&self, name: &str, birth_date: &str) -> Result<bool, String> {
        // Find the key for the chart with matching name and birth_date
        let key_to_remove = {
            let charts = self
                .charts
                .read()
                .map_err(|_| "Failed to acquire read lock")?;
            charts
                .iter()
                .find(|(_, c)| c.name == name && c.birth_date == birth_date)
                .map(|(k, _)| k.clone())
        };

        if let Some(key) = key_to_remove {
            let mut charts = self
                .charts
                .write()
                .map_err(|_| "Failed to acquire write lock")?;
            charts.remove(&key);
            drop(charts);
            self.persist()?;
            Ok(true)
        } else {
            Ok(false)
        }
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
