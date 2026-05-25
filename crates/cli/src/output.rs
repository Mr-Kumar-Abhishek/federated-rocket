/// Display formatting utilities for CLI output.
/// All functions are public for downstream use.
/// Format altitude for display
#[allow(dead_code)]
pub fn format_altitude(meters: f64) -> String {
    format!("{:.1} m ({:.1} ft)", meters, meters * 3.28084)
}

/// Format velocity for display
#[allow(dead_code)]
pub fn format_velocity(mps: f64) -> String {
    format!("{:.1} m/s ({:.1} mph)", mps, mps * 2.23694)
}

/// Format acceleration for display
#[allow(dead_code)]
pub fn format_acceleration(mps2: f64) -> String {
    format!("{:.1} m/s² ({:.1} G)", mps2, mps2 / 9.80665)
}

/// Format a duration for display
#[allow(dead_code)]
pub fn format_duration(seconds: f64) -> String {
    if seconds >= 60.0 {
        format!("{:.0}m {:.1}s", seconds / 60.0, seconds % 60.0)
    } else {
        format!("{:.2}s", seconds)
    }
}
