use crate::database::MotorDatabase;
use crate::database::SearchQuery;
use crate::types::{Motor, MotorType, ThrustPoint};
use reqwest::blocking::Client;
use std::collections::HashMap;

/// ThrustCurve.org API client for fetching motor data from the web.
pub struct ThrustCurveApi {
    client: Client,
    base_url: String,
}

/// Errors that can occur during ThrustCurve API operations.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API returned error: {0}")]
    Api(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

/// Response structure from the ThrustCurve API motor search endpoint.
#[derive(Debug, serde::Deserialize)]
struct ApiSearchResponse {
    results: Vec<ApiMotorSummary>,
}

/// Summary motor data from ThrustCurve API search results.
#[derive(Debug, serde::Deserialize)]
struct ApiMotorSummary {
    #[serde(default)]
    motor_id: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    manufacturer: Option<String>,
    #[serde(default)]
    manufacturer_abbrev: Option<String>,
    #[serde(default)]
    designation: Option<String>,
    #[serde(rename = "type", default)]
    motor_type: Option<String>,
    #[serde(default)]
    diameter: Option<f64>,
    #[serde(default)]
    length: Option<f64>,
    #[serde(default)]
    total_impulse: Option<f64>,
    #[serde(default)]
    burn_time: Option<f64>,
    #[serde(default)]
    avg_thrust: Option<f64>,
    #[serde(default)]
    max_thrust: Option<f64>,
    #[serde(default)]
    propellant_mass: Option<f64>,
    #[serde(default)]
    dry_mass: Option<f64>,
    #[serde(default)]
    delay: Option<f64>,
}

/// Detailed motor data from ThrustCurve API.
#[derive(Debug, serde::Deserialize)]
struct ApiMotorDetail {
    #[serde(default)]
    motor_id: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    manufacturer: Option<String>,
    #[serde(default)]
    manufacturer_abbrev: Option<String>,
    #[serde(default)]
    designation: Option<String>,
    #[serde(rename = "type", default)]
    motor_type: Option<String>,
    #[serde(default)]
    diameter: Option<f64>,
    #[serde(default)]
    length: Option<f64>,
    #[serde(default)]
    total_impulse: Option<f64>,
    #[serde(default)]
    burn_time: Option<f64>,
    #[serde(default)]
    avg_thrust: Option<f64>,
    #[serde(default)]
    max_thrust: Option<f64>,
    #[serde(default)]
    propellant_mass: Option<f64>,
    #[serde(default)]
    dry_mass: Option<f64>,
    #[serde(default)]
    data: Option<ApiData>,
}

#[derive(Debug, serde::Deserialize)]
struct ApiData {
    #[serde(default)]
    delay: Option<f64>,
    #[serde(default)]
    dry_mass: Option<f64>,
    #[allow(dead_code)]
    total_mass: Option<f64>,
}

impl ThrustCurveApi {
    /// Create a new ThrustCurve API client.
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("federated-rocket/0.1.0")
            .build()
            .expect("Failed to build HTTP client");

        ThrustCurveApi {
            client,
            base_url: "https://www.thrustcurve.org/api/v1".to_string(),
        }
    }

    /// Get the effective motor ID from an API response.
    fn get_motor_id(id: &Option<String>, alt_id: &Option<String>) -> Option<String> {
        id.clone().or_else(|| alt_id.clone())
    }

    /// Convert an API motor summary to a local Motor struct.
    fn api_to_motor(summary: &ApiMotorSummary) -> Option<Motor> {
        let _motor_id = Self::get_motor_id(&summary.motor_id, &summary.id)?;

        Some(Motor {
            id: None,
            manufacturer: summary.manufacturer.clone().unwrap_or_default(),
            manufacturer_abbrev: summary.manufacturer_abbrev.clone().unwrap_or_default(),
            designation: summary.designation.clone().unwrap_or_default(),
            motor_type: match summary.motor_type.as_deref() {
                Some("solid") => MotorType::Solid,
                Some("hybrid") => MotorType::Hybrid,
                Some("liquid") => MotorType::Liquid,
                Some("electric") => MotorType::Electric,
                _ => MotorType::Solid,
            },
            diameter: summary.diameter.unwrap_or(0.0),
            length: summary.length.unwrap_or(0.0),
            total_impulse: summary.total_impulse.unwrap_or(0.0),
            burn_time: summary.burn_time.unwrap_or(0.0),
            avg_thrust: summary.avg_thrust.unwrap_or(0.0),
            max_thrust: summary.max_thrust.unwrap_or(0.0),
            propellant_mass: summary.propellant_mass.unwrap_or(0.0),
            dry_mass: summary.dry_mass.unwrap_or(0.0),
            delay_time: summary.delay.unwrap_or(0.0),
            thrust_curve: vec![],
            data_source: "thrustcurve".to_string(),
        })
    }

    /// Convert API motor detail to a local Motor struct.
    fn api_detail_to_motor(detail: &ApiMotorDetail) -> Option<Motor> {
        let _motor_id = Self::get_motor_id(&detail.motor_id, &detail.id)?;

        let dry_mass = detail
            .data
            .as_ref()
            .and_then(|d| d.dry_mass)
            .or(detail.dry_mass)
            .unwrap_or(0.0);

        let delay_time = detail.data.as_ref().and_then(|d| d.delay).unwrap_or(0.0);

        Some(Motor {
            id: None,
            manufacturer: detail.manufacturer.clone().unwrap_or_default(),
            manufacturer_abbrev: detail.manufacturer_abbrev.clone().unwrap_or_default(),
            designation: detail.designation.clone().unwrap_or_default(),
            motor_type: match detail.motor_type.as_deref() {
                Some("solid") => MotorType::Solid,
                Some("hybrid") => MotorType::Hybrid,
                Some("liquid") => MotorType::Liquid,
                Some("electric") => MotorType::Electric,
                _ => MotorType::Solid,
            },
            diameter: detail.diameter.unwrap_or(0.0),
            length: detail.length.unwrap_or(0.0),
            total_impulse: detail.total_impulse.unwrap_or(0.0),
            burn_time: detail.burn_time.unwrap_or(0.0),
            avg_thrust: detail.avg_thrust.unwrap_or(0.0),
            max_thrust: detail.max_thrust.unwrap_or(0.0),
            propellant_mass: detail.propellant_mass.unwrap_or(0.0),
            dry_mass,
            delay_time,
            thrust_curve: vec![],
            data_source: "thrustcurve".to_string(),
        })
    }

    /// Search for motors using the ThrustCurve API.
    ///
    /// Maps the local `SearchQuery` to API query parameters.
    pub fn search_motors(&self, query: &SearchQuery) -> Result<Vec<Motor>, ApiError> {
        let mut params: HashMap<String, String> = HashMap::new();

        if let Some(ref mfr) = query.manufacturer {
            params.insert("manufacturer".to_string(), mfr.clone());
        }
        if let Some(ref desig) = query.designation {
            params.insert("designation".to_string(), desig.clone());
        }
        if let Some(ref ic) = query.impulse_class {
            let (min_impulse, max_impulse) = ic.impulse_range();
            params.insert("impulse_class_min".to_string(), min_impulse.to_string());
            params.insert("impulse_class_max".to_string(), max_impulse.to_string());
        }
        if let Some(d) = query.diameter_min {
            params.insert("diameter_min".to_string(), d.to_string());
        }
        if let Some(d) = query.diameter_max {
            params.insert("diameter_max".to_string(), d.to_string());
        }
        if let Some(ref ds) = query.data_source {
            params.insert("data_source".to_string(), ds.clone());
        }

        let url = format!("{}/motor/search", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .map_err(ApiError::Http)?;

        if !response.status().is_success() {
            return Err(ApiError::Api(format!(
                "API returned status {}",
                response.status()
            )));
        }

        // The API might return JSON or an error page; try to parse
        let body = response.text().map_err(ApiError::Http)?;

        // Try to parse as the expected search response
        let search_response: ApiSearchResponse = serde_json::from_str(&body).map_err(|e| {
            ApiError::Parse(format!(
                "Failed to parse search response: {} (body: {}...)",
                e,
                &body[..body.len().min(200)]
            ))
        })?;

        let motors: Vec<Motor> = search_response
            .results
            .iter()
            .filter_map(Self::api_to_motor)
            .collect();

        // Apply limit if specified
        let motors = if let Some(limit) = query.limit {
            motors.into_iter().take(limit).collect()
        } else {
            motors
        };

        Ok(motors)
    }

    /// Get detailed information about a specific motor from the API.
    pub fn get_motor_detail(&self, motor_id: &str) -> Result<Motor, ApiError> {
        let url = format!("{}/motor/{}", self.base_url, motor_id);
        let response = self.client.get(&url).send().map_err(ApiError::Http)?;

        if !response.status().is_success() {
            return Err(ApiError::Api(format!(
                "API returned status {} for motor {}",
                response.status(),
                motor_id
            )));
        }

        let body = response.text().map_err(ApiError::Http)?;

        let detail: ApiMotorDetail = serde_json::from_str(&body).map_err(|e| {
            ApiError::Parse(format!(
                "Failed to parse motor detail: {} (body: {}...)",
                e,
                &body[..body.len().min(200)]
            ))
        })?;

        let mut motor = Self::api_detail_to_motor(&detail).ok_or_else(|| {
            ApiError::Parse(format!(
                "Motor detail response missing motor_id for motor {}",
                motor_id
            ))
        })?;

        // Try to fetch thrust curve
        match self.get_thrust_curve(motor_id) {
            Ok(curve) => motor.thrust_curve = curve,
            Err(e) => {
                // If thrust curve fails, motor is still usable without it
                eprintln!(
                    "Warning: Failed to fetch thrust curve for {}: {}",
                    motor_id, e
                );
            }
        }

        Ok(motor)
    }

    /// Download a thrust curve CSV from the ThrustCurve API and parse it.
    pub fn get_thrust_curve(&self, motor_id: &str) -> Result<Vec<ThrustPoint>, ApiError> {
        let url = format!("{}/motor/{}/thrust_curve", self.base_url, motor_id);
        let response = self.client.get(&url).send().map_err(ApiError::Http)?;

        if !response.status().is_success() {
            return Err(ApiError::Api(format!(
                "API returned status {} for thrust curve of motor {}",
                response.status(),
                motor_id
            )));
        }

        let body = response.text().map_err(ApiError::Http)?;

        // Parse CSV data
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(body.as_bytes());

        let mut points = Vec::new();
        for result in reader.records() {
            let record = result
                .map_err(|e| ApiError::Parse(format!("CSV parse error in thrust curve: {}", e)))?;

            if record.len() < 2 {
                continue;
            }

            let time: f64 = record.get(0).unwrap_or("0").parse().map_err(|e| {
                ApiError::Parse(format!("Invalid time value in thrust curve: {}", e))
            })?;
            let thrust: f64 = record.get(1).unwrap_or("0").parse().map_err(|e| {
                ApiError::Parse(format!("Invalid thrust value in thrust curve: {}", e))
            })?;

            points.push(ThrustPoint { time, thrust });
        }

        Ok(points)
    }

    /// Download multiple motors from ThrustCurve and store them in the local database.
    pub fn download_to_database(
        &self,
        db: &mut MotorDatabase,
        motor_ids: &[&str],
    ) -> Result<usize, ApiError> {
        let mut count = 0;

        for motor_id in motor_ids {
            match self.get_motor_detail(motor_id) {
                Ok(motor) => match db.add_motor(&motor) {
                    Ok(_) => count += 1,
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to store motor {} in database: {}",
                            motor_id, e
                        );
                    }
                },
                Err(e) => {
                    eprintln!("Warning: Failed to fetch motor {}: {}", motor_id, e);
                }
            }
        }

        Ok(count)
    }

    /// Search ThrustCurve API and automatically import results into the local database.
    pub fn search_and_import(
        &self,
        db: &mut MotorDatabase,
        query: &SearchQuery,
    ) -> Result<usize, ApiError> {
        let motors = self.search_motors(query)?;
        let mut count = 0;

        for motor in &motors {
            match db.add_motor(motor) {
                Ok(_) => count += 1,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to import motor '{} {}': {}",
                        motor.manufacturer, motor.designation, e
                    );
                }
            }
        }

        Ok(count)
    }
}

impl Default for ThrustCurveApi {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ImpulseClass;

    #[test]
    fn test_api_creation() {
        let api = ThrustCurveApi::new();
        assert_eq!(api.base_url, "https://www.thrustcurve.org/api/v1");
    }

    #[test]
    fn test_api_to_motor_conversion() {
        let summary = ApiMotorSummary {
            motor_id: Some("123".to_string()),
            id: None,
            manufacturer: Some("Estes".to_string()),
            manufacturer_abbrev: Some("EST".to_string()),
            designation: Some("C6-5".to_string()),
            motor_type: Some("solid".to_string()),
            diameter: Some(18.0),
            length: Some(70.0),
            total_impulse: Some(8.82),
            burn_time: Some(1.6),
            avg_thrust: Some(5.5),
            max_thrust: Some(12.0),
            propellant_mass: Some(10.5),
            dry_mass: Some(11.0),
            delay: Some(5.0),
        };

        let motor = ThrustCurveApi::api_to_motor(&summary).unwrap();
        assert_eq!(motor.manufacturer, "Estes");
        assert_eq!(motor.designation, "C6-5");
        assert_eq!(motor.diameter, 18.0);
        assert_eq!(motor.total_impulse, 8.82);
        assert_eq!(motor.data_source, "thrustcurve");
    }

    #[test]
    fn test_missing_motor_id_returns_none() {
        let summary = ApiMotorSummary {
            motor_id: None,
            id: None,
            manufacturer: Some("Estes".to_string()),
            manufacturer_abbrev: Some("EST".to_string()),
            designation: Some("C6-5".to_string()),
            motor_type: Some("solid".to_string()),
            diameter: Some(18.0),
            length: Some(70.0),
            total_impulse: Some(8.82),
            burn_time: Some(1.6),
            avg_thrust: Some(5.5),
            max_thrust: Some(12.0),
            propellant_mass: Some(10.5),
            dry_mass: Some(11.0),
            delay: Some(5.0),
        };

        assert!(ThrustCurveApi::api_to_motor(&summary).is_none());
    }

    #[test]
    fn test_impulse_class_query_params() {
        let _query = SearchQuery {
            impulse_class: Some(ImpulseClass::C),
            ..Default::default()
        };

        // Just verify the impulse class range is used correctly
        let (min_i, max_i) = ImpulseClass::C.impulse_range();
        assert!((min_i - 5.01).abs() < 1e-10);
        assert!((max_i - 10.00).abs() < 1e-10);
    }
}
