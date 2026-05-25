use crate::types::{ImpulseClass, Motor, MotorType, ThrustPoint};
use rusqlite::{params, Connection, Result as SqlResult};
use std::path::Path;

/// Motor database backed by SQLite.
pub struct MotorDatabase {
    conn: Connection,
}

/// Errors that can occur during database operations.
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Motor not found: {0}")]
    NotFound(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

/// Search query parameters for finding motors.
#[derive(Debug, Default)]
pub struct SearchQuery {
    /// Filter by manufacturer name (partial match)
    pub manufacturer: Option<String>,
    /// Filter by designation (partial match)
    pub designation: Option<String>,
    /// Filter by impulse class
    pub impulse_class: Option<ImpulseClass>,
    /// Minimum motor diameter in mm
    pub diameter_min: Option<f64>,
    /// Maximum motor diameter in mm
    pub diameter_max: Option<f64>,
    /// Minimum motor length in mm
    pub length_min: Option<f64>,
    /// Maximum motor length in mm
    pub length_max: Option<f64>,
    /// Exact delay time in seconds
    pub delay_time: Option<f64>,
    /// Filter by data source
    pub data_source: Option<String>,
    /// Maximum number of results to return
    pub limit: Option<usize>,
}

impl MotorDatabase {
    /// Create a new motor database.
    ///
    /// If `path` is `None`, an in-memory database is created.
    /// Tables are created automatically if they don't exist.
    pub fn new(path: Option<&Path>) -> Self {
        let conn = match path {
            Some(p) => Connection::open(p).expect("Failed to open SQLite database"),
            None => Connection::open_in_memory().expect("Failed to create in-memory database"),
        };

        // Enable WAL mode for better concurrent performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .expect("Failed to set WAL mode");

        // Create tables
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS motors (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                manufacturer TEXT NOT NULL,
                manufacturer_abbrev TEXT NOT NULL,
                designation TEXT NOT NULL,
                motor_type TEXT NOT NULL DEFAULT 'Solid',
                diameter REAL NOT NULL,
                length REAL NOT NULL,
                total_impulse REAL NOT NULL,
                burn_time REAL NOT NULL,
                avg_thrust REAL NOT NULL,
                max_thrust REAL NOT NULL,
                propellant_mass REAL NOT NULL,
                dry_mass REAL NOT NULL,
                delay_time REAL NOT NULL DEFAULT 0,
                data_source TEXT NOT NULL DEFAULT 'embedded',
                UNIQUE(manufacturer, designation)
            );

            CREATE TABLE IF NOT EXISTS thrust_points (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                motor_id INTEGER NOT NULL,
                time REAL NOT NULL,
                thrust REAL NOT NULL,
                FOREIGN KEY (motor_id) REFERENCES motors(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_motors_manufacturer ON motors(manufacturer);
            CREATE INDEX IF NOT EXISTS idx_motors_designation ON motors(designation);
            CREATE INDEX IF NOT EXISTS idx_motors_impulse ON motors(total_impulse);
            CREATE INDEX IF NOT EXISTS idx_thrust_points_motor ON thrust_points(motor_id);
            ",
        )
        .expect("Failed to create database tables");

        MotorDatabase { conn }
    }

    /// Add a motor to the database and return its assigned ID.
    /// Also inserts all thrust curve points.
    pub fn add_motor(&mut self, motor: &Motor) -> Result<i64, DatabaseError> {
        let tx = self.conn.transaction()?;

        tx.execute(
            "INSERT INTO motors (manufacturer, manufacturer_abbrev, designation, motor_type,
             diameter, length, total_impulse, burn_time, avg_thrust, max_thrust,
             propellant_mass, dry_mass, delay_time, data_source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                motor.manufacturer,
                motor.manufacturer_abbrev,
                motor.designation,
                motor.motor_type.as_str(),
                motor.diameter,
                motor.length,
                motor.total_impulse,
                motor.burn_time,
                motor.avg_thrust,
                motor.max_thrust,
                motor.propellant_mass,
                motor.dry_mass,
                motor.delay_time,
                motor.data_source,
            ],
        )?;

        let motor_id = tx.last_insert_rowid();

        // Insert thrust curve points
        let mut stmt =
            tx.prepare("INSERT INTO thrust_points (motor_id, time, thrust) VALUES (?1, ?2, ?3)")?;
        for pt in &motor.thrust_curve {
            stmt.execute(params![motor_id, pt.time, pt.thrust])?;
        }
        drop(stmt);

        tx.commit()?;

        Ok(motor_id)
    }

    /// Retrieve a motor by its database ID, including thrust curve data.
    pub fn get_motor(&self, id: i64) -> Result<Motor, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, manufacturer, manufacturer_abbrev, designation, motor_type,
                 diameter, length, total_impulse, burn_time, avg_thrust, max_thrust,
                 propellant_mass, dry_mass, delay_time, data_source
                 FROM motors WHERE id = ?1",
            )
            .map_err(DatabaseError::Sqlite)?;

        let motor_result = stmt.query_row(params![id], |row| {
            let motor_type_str: String = row.get(4)?;
            Ok(Motor {
                id: Some(row.get(0)?),
                manufacturer: row.get(1)?,
                manufacturer_abbrev: row.get(2)?,
                designation: row.get(3)?,
                motor_type: MotorType::from_str(&motor_type_str).ok_or_else(|| {
                    rusqlite::Error::InvalidParameterName(format!(
                        "Invalid motor_type: {}",
                        motor_type_str
                    ))
                })?,
                diameter: row.get(5)?,
                length: row.get(6)?,
                total_impulse: row.get(7)?,
                burn_time: row.get(8)?,
                avg_thrust: row.get(9)?,
                max_thrust: row.get(10)?,
                propellant_mass: row.get(11)?,
                dry_mass: row.get(12)?,
                delay_time: row.get(13)?,
                thrust_curve: vec![], // will be filled below
                data_source: row.get(14)?,
            })
        });

        let mut motor = match motor_result {
            Ok(m) => m,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(DatabaseError::NotFound(format!(
                    "Motor ID {} not found",
                    id
                )));
            }
            Err(e) => return Err(DatabaseError::Sqlite(e)),
        };

        // Load thrust curve
        motor.thrust_curve = self.get_thrust_curve(id)?;

        Ok(motor)
    }

    /// Retrieve a motor by manufacturer and designation.
    pub fn get_motor_by_designation(
        &self,
        manufacturer: &str,
        designation: &str,
    ) -> Result<Motor, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM motors WHERE manufacturer = ?1 AND designation = ?2")
            .map_err(DatabaseError::Sqlite)?;

        let id: i64 = stmt
            .query_row(params![manufacturer, designation], |row| row.get(0))
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => DatabaseError::NotFound(format!(
                    "Motor '{} {}' not found",
                    manufacturer, designation
                )),
                other => DatabaseError::Sqlite(other),
            })?;

        self.get_motor(id)
    }

    /// Search for motors using the given query filters.
    ///
    /// Builds a dynamic SQL query based on which filters are active.
    pub fn search_motors(&self, query: &SearchQuery) -> Result<Vec<Motor>, DatabaseError> {
        let mut sql = String::from("SELECT id FROM motors WHERE 1=1");
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref mfr) = query.manufacturer {
            param_values.push(Box::new(mfr.clone()));
            sql.push_str(&format!(" AND manufacturer LIKE ?{}", param_values.len()));
        }

        if let Some(ref desig) = query.designation {
            param_values.push(Box::new(desig.clone()));
            sql.push_str(&format!(" AND designation LIKE ?{}", param_values.len()));
        }

        if let Some(ref ic) = query.impulse_class {
            let (min_impulse, max_impulse) = ic.impulse_range();
            param_values.push(Box::new(min_impulse));
            sql.push_str(&format!(" AND total_impulse >= ?{}", param_values.len()));
            param_values.push(Box::new(max_impulse));
            sql.push_str(&format!(" AND total_impulse <= ?{}", param_values.len()));
        }

        if let Some(d) = query.diameter_min {
            param_values.push(Box::new(d));
            sql.push_str(&format!(" AND diameter >= ?{}", param_values.len()));
        }

        if let Some(d) = query.diameter_max {
            param_values.push(Box::new(d));
            sql.push_str(&format!(" AND diameter <= ?{}", param_values.len()));
        }

        if let Some(l) = query.length_min {
            param_values.push(Box::new(l));
            sql.push_str(&format!(" AND length >= ?{}", param_values.len()));
        }

        if let Some(l) = query.length_max {
            param_values.push(Box::new(l));
            sql.push_str(&format!(" AND length <= ?{}", param_values.len()));
        }

        if let Some(d) = query.delay_time {
            param_values.push(Box::new(d));
            sql.push_str(&format!(" AND delay_time = ?{}", param_values.len()));
        }

        if let Some(ref ds) = query.data_source {
            param_values.push(Box::new(ds.clone()));
            sql.push_str(&format!(" AND data_source = ?{}", param_values.len()));
        }

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql).map_err(DatabaseError::Sqlite)?;
        let ids: Vec<i64> = stmt
            .query_map(params_refs.as_slice(), |row| row.get(0))
            .map_err(DatabaseError::Sqlite)?
            .collect::<SqlResult<Vec<i64>>>()
            .map_err(DatabaseError::Sqlite)?;

        let mut motors = Vec::with_capacity(ids.len());
        for id in ids {
            motors.push(self.get_motor(id)?);
        }

        Ok(motors)
    }

    /// List all unique manufacturer names in the database.
    pub fn list_manufacturers(&self) -> Result<Vec<String>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT manufacturer FROM motors ORDER BY manufacturer")
            .map_err(DatabaseError::Sqlite)?;

        let manufacturers = stmt
            .query_map([], |row| row.get(0))
            .map_err(DatabaseError::Sqlite)?
            .collect::<SqlResult<Vec<String>>>()
            .map_err(DatabaseError::Sqlite)?;

        Ok(manufacturers)
    }

    /// List all motors in the database.
    pub fn list_motors(&self) -> Result<Vec<Motor>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM motors ORDER BY manufacturer, designation")
            .map_err(DatabaseError::Sqlite)?;

        let ids: Vec<i64> = stmt
            .query_map([], |row| row.get(0))
            .map_err(DatabaseError::Sqlite)?
            .collect::<SqlResult<Vec<i64>>>()
            .map_err(DatabaseError::Sqlite)?;

        let mut motors = Vec::with_capacity(ids.len());
        for id in ids {
            motors.push(self.get_motor(id)?);
        }

        Ok(motors)
    }

    /// Delete a motor and its thrust curve points from the database.
    pub fn delete_motor(&mut self, id: i64) -> Result<(), DatabaseError> {
        let tx = self.conn.transaction()?;

        // Delete thrust points first (cascade should handle this, but be explicit)
        tx.execute("DELETE FROM thrust_points WHERE motor_id = ?1", params![id])?;

        let affected = tx.execute("DELETE FROM motors WHERE id = ?1", params![id])?;

        if affected == 0 {
            tx.commit()?; // commit anyway, consistent state
            return Err(DatabaseError::NotFound(format!(
                "Motor ID {} not found for deletion",
                id
            )));
        }

        tx.commit()?;
        Ok(())
    }

    /// Get the total number of motors in the database.
    pub fn motor_count(&self) -> Result<i64, DatabaseError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM motors", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Retrieve thrust curve points for a given motor ID.
    pub fn get_thrust_curve(&self, motor_id: i64) -> Result<Vec<ThrustPoint>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare("SELECT time, thrust FROM thrust_points WHERE motor_id = ?1 ORDER BY time")
            .map_err(DatabaseError::Sqlite)?;

        let points = stmt
            .query_map(params![motor_id], |row| {
                Ok(ThrustPoint {
                    time: row.get(0)?,
                    thrust: row.get(1)?,
                })
            })
            .map_err(DatabaseError::Sqlite)?
            .collect::<SqlResult<Vec<ThrustPoint>>>()
            .map_err(DatabaseError::Sqlite)?;

        Ok(points)
    }

    /// Import motors from CSV data.
    ///
    /// Expected CSV format (header row required):
    /// manufacturer,manufacturer_abbrev,designation,motor_type,diameter,length,
    /// total_impulse,burn_time,avg_thrust,max_thrust,propellant_mass,dry_mass,
    /// delay_time,data_source
    ///
    /// Thrust curve data is not included in CSV import; imported motors will
    /// have an empty thrust curve.
    pub fn import_from_csv(&mut self, csv_data: &str) -> Result<usize, DatabaseError> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(csv_data.as_bytes());

        let mut count = 0;

        for result in reader.records() {
            let record = result
                .map_err(|e| DatabaseError::InvalidData(format!("CSV parse error: {}", e)))?;

            if record.len() < 14 {
                return Err(DatabaseError::InvalidData(format!(
                    "CSV row has {} fields, expected at least 14",
                    record.len()
                )));
            }

            let motor_type_str = record.get(3).unwrap_or("Solid");
            let motor_type = MotorType::from_str(motor_type_str).unwrap_or(MotorType::Solid);

            let motor =
                Motor {
                    id: None,
                    manufacturer: record.get(0).unwrap_or_default().to_string(),
                    manufacturer_abbrev: record.get(1).unwrap_or_default().to_string(),
                    designation: record.get(2).unwrap_or_default().to_string(),
                    motor_type,
                    diameter: record.get(4).unwrap_or("0").parse().map_err(|e| {
                        DatabaseError::InvalidData(format!("Invalid diameter: {}", e))
                    })?,
                    length: record.get(5).unwrap_or("0").parse().map_err(|e| {
                        DatabaseError::InvalidData(format!("Invalid length: {}", e))
                    })?,
                    total_impulse: record.get(6).unwrap_or("0").parse().map_err(|e| {
                        DatabaseError::InvalidData(format!("Invalid total_impulse: {}", e))
                    })?,
                    burn_time: record.get(7).unwrap_or("0").parse().map_err(|e| {
                        DatabaseError::InvalidData(format!("Invalid burn_time: {}", e))
                    })?,
                    avg_thrust: record.get(8).unwrap_or("0").parse().map_err(|e| {
                        DatabaseError::InvalidData(format!("Invalid avg_thrust: {}", e))
                    })?,
                    max_thrust: record.get(9).unwrap_or("0").parse().map_err(|e| {
                        DatabaseError::InvalidData(format!("Invalid max_thrust: {}", e))
                    })?,
                    propellant_mass: record.get(10).unwrap_or("0").parse().map_err(|e| {
                        DatabaseError::InvalidData(format!("Invalid propellant_mass: {}", e))
                    })?,
                    dry_mass: record.get(11).unwrap_or("0").parse().map_err(|e| {
                        DatabaseError::InvalidData(format!("Invalid dry_mass: {}", e))
                    })?,
                    delay_time: record.get(12).unwrap_or("0").parse().map_err(|e| {
                        DatabaseError::InvalidData(format!("Invalid delay_time: {}", e))
                    })?,
                    thrust_curve: vec![],
                    data_source: record.get(13).unwrap_or("user").to_string(),
                };

            self.add_motor(&motor)?;
            count += 1;
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{MotorType, ThrustPoint};

    fn create_test_motor() -> Motor {
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
            thrust_curve: vec![
                ThrustPoint {
                    time: 0.0,
                    thrust: 0.0,
                },
                ThrustPoint {
                    time: 0.1,
                    thrust: 12.0,
                },
                ThrustPoint {
                    time: 0.5,
                    thrust: 8.0,
                },
                ThrustPoint {
                    time: 1.0,
                    thrust: 5.0,
                },
                ThrustPoint {
                    time: 1.6,
                    thrust: 0.0,
                },
            ],
            data_source: "test".to_string(),
        }
    }

    fn create_another_motor() -> Motor {
        Motor {
            id: None,
            manufacturer: "Aerotech".to_string(),
            manufacturer_abbrev: "AT".to_string(),
            designation: "G25-10".to_string(),
            motor_type: MotorType::Solid,
            diameter: 29.0,
            length: 120.0,
            total_impulse: 120.0,
            burn_time: 4.8,
            avg_thrust: 25.0,
            max_thrust: 40.0,
            propellant_mass: 55.0,
            dry_mass: 45.0,
            delay_time: 10.0,
            thrust_curve: vec![
                ThrustPoint {
                    time: 0.0,
                    thrust: 0.0,
                },
                ThrustPoint {
                    time: 0.2,
                    thrust: 35.0,
                },
                ThrustPoint {
                    time: 1.0,
                    thrust: 28.0,
                },
                ThrustPoint {
                    time: 3.0,
                    thrust: 22.0,
                },
                ThrustPoint {
                    time: 4.8,
                    thrust: 0.0,
                },
            ],
            data_source: "test".to_string(),
        }
    }

    #[test]
    fn test_create_in_memory_database() {
        let db = MotorDatabase::new(None);
        assert!(db.motor_count().is_ok());
        assert_eq!(db.motor_count().unwrap(), 0);
    }

    #[test]
    fn test_add_and_get_motor() {
        let mut db = MotorDatabase::new(None);
        let motor = create_test_motor();
        let id = db.add_motor(&motor).unwrap();
        assert!(id > 0);

        let retrieved = db.get_motor(id).unwrap();
        assert_eq!(retrieved.manufacturer, "Estes");
        assert_eq!(retrieved.designation, "C6-5");
        assert_eq!(retrieved.thrust_curve.len(), 5);
        assert_eq!(retrieved.id, Some(id));
    }

    #[test]
    fn test_get_motor_by_designation() {
        let mut db = MotorDatabase::new(None);
        let motor = create_test_motor();
        let id = db.add_motor(&motor).unwrap();

        let retrieved = db.get_motor_by_designation("Estes", "C6-5").unwrap();
        assert_eq!(retrieved.id, Some(id));
        assert_eq!(retrieved.manufacturer, "Estes");
    }

    #[test]
    fn test_get_motor_not_found() {
        let db = MotorDatabase::new(None);
        let result = db.get_motor(999);
        assert!(matches!(result, Err(DatabaseError::NotFound(_))));
    }

    #[test]
    fn test_search_motors() {
        let mut db = MotorDatabase::new(None);
        db.add_motor(&create_test_motor()).unwrap();
        db.add_motor(&create_another_motor()).unwrap();

        // Search by manufacturer
        let query = SearchQuery {
            manufacturer: Some("Estes".to_string()),
            ..Default::default()
        };
        let results = db.search_motors(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].manufacturer, "Estes");

        // Search by impulse class (C is 5.01-10.00 N·s, Estes C6-5 has 8.82)
        let query = SearchQuery {
            impulse_class: Some(ImpulseClass::C),
            ..Default::default()
        };
        let results = db.search_motors(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].designation, "C6-5");

        // Search by diameter range
        let query = SearchQuery {
            diameter_min: Some(20.0),
            diameter_max: Some(35.0),
            ..Default::default()
        };
        let results = db.search_motors(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].manufacturer, "Aerotech");
    }

    #[test]
    fn test_list_manufacturers() {
        let mut db = MotorDatabase::new(None);
        db.add_motor(&create_test_motor()).unwrap();
        db.add_motor(&create_another_motor()).unwrap();

        let manufacturers = db.list_manufacturers().unwrap();
        assert!(manufacturers.contains(&"Estes".to_string()));
        assert!(manufacturers.contains(&"Aerotech".to_string()));
    }

    #[test]
    fn test_list_motors() {
        let mut db = MotorDatabase::new(None);
        db.add_motor(&create_test_motor()).unwrap();
        db.add_motor(&create_another_motor()).unwrap();

        let motors = db.list_motors().unwrap();
        assert_eq!(motors.len(), 2);
    }

    #[test]
    fn test_delete_motor() {
        let mut db = MotorDatabase::new(None);
        let id = db.add_motor(&create_test_motor()).unwrap();
        assert_eq!(db.motor_count().unwrap(), 1);

        db.delete_motor(id).unwrap();
        assert_eq!(db.motor_count().unwrap(), 0);
        assert!(matches!(db.get_motor(id), Err(DatabaseError::NotFound(_))));
    }

    #[test]
    fn test_motor_count() {
        let mut db = MotorDatabase::new(None);
        assert_eq!(db.motor_count().unwrap(), 0);
        db.add_motor(&create_test_motor()).unwrap();
        assert_eq!(db.motor_count().unwrap(), 1);
        db.add_motor(&create_another_motor()).unwrap();
        assert_eq!(db.motor_count().unwrap(), 2);
    }

    #[test]
    fn test_get_thrust_curve() {
        let mut db = MotorDatabase::new(None);
        let id = db.add_motor(&create_test_motor()).unwrap();

        let curve = db.get_thrust_curve(id).unwrap();
        assert_eq!(curve.len(), 5);
        assert!((curve[0].time - 0.0).abs() < 1e-10);
        assert!((curve[4].time - 1.6).abs() < 1e-10);
        assert!((curve[4].thrust - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_import_from_csv() {
        let mut db = MotorDatabase::new(None);
        let csv_data = "manufacturer,manufacturer_abbrev,designation,motor_type,diameter,length,total_impulse,burn_time,avg_thrust,max_thrust,propellant_mass,dry_mass,delay_time,data_source\nEstes,EST,C6-5,Solid,18.0,70.0,8.82,1.6,5.5,12.0,10.5,11.0,5.0,user\nAerotech,AT,G25-10,Solid,29.0,120.0,120.0,4.8,25.0,40.0,55.0,45.0,10.0,user";

        let count = db.import_from_csv(csv_data).unwrap();
        assert_eq!(count, 2);
        assert_eq!(db.motor_count().unwrap(), 2);
    }

    #[test]
    fn test_import_from_csv_invalid_data() {
        let mut db = MotorDatabase::new(None);
        let csv_data = "manufacturer,manufacturer_abbrev,designation,motor_type,diameter\nEstes,EST,C6-5,Solid,not_a_number";
        let result = db.import_from_csv(csv_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_nonexistent_motor() {
        let mut db = MotorDatabase::new(None);
        let result = db.delete_motor(999);
        assert!(matches!(result, Err(DatabaseError::NotFound(_))));
    }
}
