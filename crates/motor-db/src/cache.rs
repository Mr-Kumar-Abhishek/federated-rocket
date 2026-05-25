use crate::types::Motor;
use std::collections::HashMap;
use std::sync::Mutex;

/// In-memory cache for frequently accessed motors.
///
/// Thread-safe wrapper around a HashMap using `Mutex`.
/// Keys follow the format `"Manufacturer/Designation"`, e.g. `"Estes/C6-5"`.
pub struct MotorCache {
    motors: Mutex<HashMap<String, Motor>>,
}

impl MotorCache {
    /// Create a new empty motor cache.
    pub fn new() -> Self {
        MotorCache {
            motors: Mutex::new(HashMap::new()),
        }
    }

    /// Retrieve a motor from the cache by key.
    ///
    /// Returns `None` if the key is not present in the cache.
    ///
    /// # Key format
    /// `"Manufacturer/Designation"`, e.g. `"Estes/C6-5"`
    pub fn get(&self, key: &str) -> Option<Motor> {
        self.motors.lock().ok()?.get(key).cloned()
    }

    /// Insert a motor into the cache with the standard key format
    /// `"Manufacturer/Designation"`.
    pub fn set(&self, key: String, motor: Motor) {
        if let Ok(mut cache) = self.motors.lock() {
            cache.insert(key, motor);
        }
    }

    /// Insert a motor into the cache, deriving the key from its manufacturer
    /// and designation fields.
    pub fn set_motor(&self, motor: Motor) {
        let key = format!("{}/{}", motor.manufacturer, motor.designation);
        if let Ok(mut cache) = self.motors.lock() {
            cache.insert(key, motor);
        }
    }

    /// Clear all entries from the cache.
    pub fn clear(&self) {
        if let Ok(mut cache) = self.motors.lock() {
            cache.clear();
        }
    }

    /// Check if a key exists in the cache.
    pub fn contains(&self, key: &str) -> bool {
        self.motors.lock().ok().map_or(false, |cache| cache.contains_key(key))
    }

    /// Return the number of entries in the cache.
    pub fn len(&self) -> usize {
        self.motors.lock().ok().map_or(0, |cache| cache.len())
    }

    /// Returns `true` if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove a specific entry from the cache by key.
    pub fn remove(&self, key: &str) -> Option<Motor> {
        self.motors.lock().ok()?.remove(key)
    }

    /// Return all keys currently in the cache.
    pub fn keys(&self) -> Vec<String> {
        self.motors
            .lock()
            .ok()
            .map(|cache| cache.keys().cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for MotorCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{MotorType, ThrustPoint};

    fn sample_motor() -> Motor {
        Motor {
            id: None,
            manufacturer: "Estes".to_string(),
            manufacturer_abbrev: "EST".to_string(),
            designation: "C6-5".to_string(),
            motor_type: MotorType::Solid,
            diameter: 18.0,
            length: 70.0,
            total_impulse: 8.82,
            burn_time: 1.6,
            avg_thrust: 5.5,
            max_thrust: 12.0,
            propellant_mass: 10.5,
            dry_mass: 11.0,
            delay_time: 5.0,
            thrust_curve: vec![ThrustPoint { time: 0.0, thrust: 0.0 }],
            data_source: "test".to_string(),
        }
    }

    #[test]
    fn test_cache_new_empty() {
        let cache = MotorCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_set_and_get() {
        let cache = MotorCache::new();
        let motor = sample_motor();
        cache.set("Estes/C6-5".to_string(), motor.clone());

        assert!(cache.contains("Estes/C6-5"));
        assert_eq!(cache.len(), 1);

        let retrieved = cache.get("Estes/C6-5").unwrap();
        assert_eq!(retrieved.manufacturer, "Estes");
        assert_eq!(retrieved.designation, "C6-5");
    }

    #[test]
    fn test_cache_set_motor() {
        let cache = MotorCache::new();
        let motor = sample_motor();
        cache.set_motor(motor);

        assert!(cache.contains("Estes/C6-5"));
    }

    #[test]
    fn test_cache_get_missing() {
        let cache = MotorCache::new();
        assert!(cache.get("NonExistent").is_none());
    }

    #[test]
    fn test_cache_clear() {
        let cache = MotorCache::new();
        cache.set("key1".to_string(), sample_motor());
        cache.set("key2".to_string(), sample_motor());
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_remove() {
        let cache = MotorCache::new();
        cache.set("Estes/C6-5".to_string(), sample_motor());
        assert_eq!(cache.len(), 1);

        let removed = cache.remove("Estes/C6-5");
        assert!(removed.is_some());
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_keys() {
        let cache = MotorCache::new();
        cache.set("key1".to_string(), sample_motor());
        cache.set("key2".to_string(), sample_motor());

        let mut keys = cache.keys();
        keys.sort();
        assert_eq!(keys, vec!["key1", "key2"]);
    }

    #[test]
    fn test_cache_thread_safety() {
        use std::thread;
        let cache = std::sync::Arc::new(MotorCache::new());
        let cache_clone = cache.clone();

        let handle = thread::spawn(move || {
            cache_clone.set("thread_key".to_string(), sample_motor());
        });

        handle.join().unwrap();
        assert!(cache.contains("thread_key"));
    }
}