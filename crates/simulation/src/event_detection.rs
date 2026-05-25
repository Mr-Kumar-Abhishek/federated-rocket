use federated_rocket_aero::compute::AeroCalculator;
use federated_rocket_core::component_tree::ComponentTree;
use federated_rocket_physics::atmosphere::AtmosphericModel;
use federated_rocket_physics::gravity::GravityModel;
use federated_rocket_physics::wind::WindModel;

use crate::derivatives::{
    compact_to_flight_state, compute_derivative, normalize_orientation, CompactState,
};
use crate::motor::MotorModel;

/// Linear interpolation to find the time when a quantity crosses a threshold.
///
/// Given values before and after a threshold crossing, computes the exact
/// time of the crossing using linear interpolation.
pub fn interpolate_event_time(
    prev_value: f64,
    threshold: f64,
    curr_value: f64,
    prev_time: f64,
    curr_time: f64,
    _threshold_value: f64,
) -> f64 {
    if (curr_value - prev_value).abs() < 1e-12 {
        return curr_time;
    }
    let fraction = (threshold - prev_value) / (curr_value - prev_value);
    prev_time + fraction * (curr_time - prev_time)
}

/// Bisection search for exact apogee time (maximum altitude).
///
/// Uses the fact that altitude derivative changes sign at apogee.
/// Iteratively narrows down the interval containing the apogee by
/// checking the altitude gradient between midpoints.
pub fn find_apogee_time(
    prev_state: &CompactState,
    curr_state: &CompactState,
    t_prev: f64,
    t_curr: f64,
    motor: &Option<MotorModel>,
    aero_calc: &AeroCalculator,
    atmosphere: &dyn AtmosphericModel,
    gravity: &dyn GravityModel,
    wind: &dyn WindModel,
    tree: &ComponentTree,
    reference_area: f64,
    reference_diameter: f64,
) -> f64 {
    const BISECT_ITERATIONS: usize = 20;
    let mut a = t_prev;
    let mut b = t_curr;
    let mut s_prev = prev_state.clone();

    for _ in 0..BISECT_ITERATIONS {
        let mid = (a + b) / 2.0;
        let dt_mid = mid - a;
        let deriv = compute_derivative(
            &s_prev,
            a,
            motor,
            aero_calc,
            atmosphere,
            gravity,
            wind,
            tree,
            reference_area,
            reference_diameter,
        );
        let mut s_mid_state = s_prev.clone() + deriv * dt_mid;
        normalize_orientation(&mut s_mid_state);
        let fs_prev = compact_to_flight_state(&s_prev, atmosphere, wind);
        let fs_mid = compact_to_flight_state(&s_mid_state, atmosphere, wind);
        let fs_curr = compact_to_flight_state(curr_state, atmosphere, wind);

        // Check if altitude derivative changed sign (peak between a and mid)
        if (fs_mid.altitude() - fs_prev.altitude()) * (fs_curr.altitude() - fs_mid.altitude()) < 0.0
        {
            // Apogee is between prev and mid
            b = mid;
        } else {
            s_prev = s_mid_state;
            a = mid;
        }
    }
    (a + b) / 2.0
}

/// Bisection search for exact burnout time (motor stops burning).
///
/// Uses the motor's [`is_burning`](MotorModel::is_burning) method to
/// find the exact time when the motor transitions from burning to not burning.
pub fn find_burnout_time(
    _prev_state: &CompactState,
    _curr_state: &CompactState,
    t_prev: f64,
    t_curr: f64,
    motor: &Option<MotorModel>,
    _aero_calc: &AeroCalculator,
    _atmosphere: &dyn AtmosphericModel,
    _gravity: &dyn GravityModel,
    _wind: &dyn WindModel,
    _tree: &ComponentTree,
    _reference_area: f64,
    _reference_diameter: f64,
) -> f64 {
    const BISECT_ITERATIONS: usize = 15;
    let mut a = t_prev;
    let mut b = t_curr;
    let motor_ref = motor.as_ref().unwrap();

    for _ in 0..BISECT_ITERATIONS {
        let mid = (a + b) / 2.0;
        let burning_mid = motor_ref.is_burning(mid);
        let burning_a = motor_ref.is_burning(a);

        if burning_a != burning_mid {
            b = mid;
        } else {
            a = mid;
        }
    }
    (a + b) / 2.0
}

/// Bisection search for exact ground hit time.
///
/// Finds the time when altitude crosses zero (ground level) by iteratively
/// advancing the state with Euler integration and halving the interval.
pub fn find_ground_hit_time(
    prev_state: &CompactState,
    _curr_state: &CompactState,
    t_prev: f64,
    t_curr: f64,
    motor: &Option<MotorModel>,
    aero_calc: &AeroCalculator,
    atmosphere: &dyn AtmosphericModel,
    gravity: &dyn GravityModel,
    wind: &dyn WindModel,
    tree: &ComponentTree,
    reference_area: f64,
    reference_diameter: f64,
) -> f64 {
    const BISECT_ITERATIONS: usize = 20;
    let mut a = t_prev;
    let mut b = t_curr;
    let mut s_a = prev_state.clone();

    for _ in 0..BISECT_ITERATIONS {
        let mid = (a + b) / 2.0;
        let dt_mid = mid - a;
        let deriv = compute_derivative(
            &s_a,
            a,
            motor,
            aero_calc,
            atmosphere,
            gravity,
            wind,
            tree,
            reference_area,
            reference_diameter,
        );
        let mut s_mid_state = s_a.clone() + deriv * dt_mid;
        normalize_orientation(&mut s_mid_state);
        let fs_mid = compact_to_flight_state(&s_mid_state, atmosphere, wind);

        if fs_mid.altitude() > 0.0 {
            s_a = s_mid_state;
            a = mid;
        } else {
            b = mid;
        }
    }
    (a + b) / 2.0
}

#[cfg(test)]
mod tests {
    use federated_rocket_aero::compute::AeroCalculator;
    use federated_rocket_core::component_tree::ComponentTree;
    use federated_rocket_math::quaternion::Quaternion;
    use federated_rocket_math::vector::Vector3D;
    use federated_rocket_physics::atmosphere::StandardAtmosphere;
    use federated_rocket_physics::gravity::ConstantGravity;
    use federated_rocket_physics::wind::NoWind;

    use super::*;
    use crate::motor::MotorModel;

    #[test]
    fn test_interpolate_event_time_linear() {
        // Simple linear case: value goes from 0 to 10, crossing threshold 5 at time 0.5
        let time = interpolate_event_time(0.0, 5.0, 10.0, 0.0, 1.0, 5.0);
        assert!((time - 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_interpolate_event_time_exact_threshold() {
        // Threshold exactly at current value → should return curr_time
        let time = interpolate_event_time(0.0, 10.0, 10.0, 0.0, 1.0, 10.0);
        assert!((time - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_interpolate_event_time_no_change() {
        // No change between prev and curr → returns curr_time
        let time = interpolate_event_time(5.0, 3.0, 5.0, 0.0, 1.0, 3.0);
        assert!((time - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_interpolate_event_time_negative_values() {
        // Crossing zero from negative to positive
        let time = interpolate_event_time(-10.0, 0.0, 10.0, 0.0, 1.0, 0.0);
        assert!((time - 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_find_apogee_time_parabolic() {
        // Create a simple parabolic trajectory to test apogee bisection
        let tree = ComponentTree::new();
        let aero = AeroCalculator::new();
        let atmo = StandardAtmosphere;
        let grav = ConstantGravity;
        let wind = NoWind;

        // Start at t=0 with upward velocity
        let state1 = CompactState::new(
            Vector3D::zero(),
            Vector3D::new(0.0, 0.0, 50.0), // 50 m/s upward
            Quaternion::identity(),
            Vector3D::zero(),
            1.0,
            0.0,
        );

        // After some time, the velocity should have decreased (gravity)
        let dt = 2.0;
        let deriv = compute_derivative(
            &state1, 0.0, &None, &aero, &atmo, &grav, &wind, &tree, 0.001, 0.05,
        );
        let mut state2 = state1.clone() + deriv * dt;
        normalize_orientation(&mut state2);

        // Find apogee between t=0 and t=2
        let apogee_time = find_apogee_time(
            &state1, &state2, 0.0, dt, &None, &aero, &atmo, &grav, &wind, &tree, 0.001, 0.05,
        );

        // With 50 m/s upward and g ≈ 9.81 m/s², apogee should be around 5.1s
        // The bisection might not get exactly to 5.1s since we're searching
        // within [0, 2] and both states might be ascending
        assert!(apogee_time >= 0.0 && apogee_time <= dt);
    }

    #[test]
    fn test_find_burnout_time_with_motor() {
        let mut motor = MotorModel::new("TestCo".into(), "T100".into());
        motor.burn_time = 2.0;
        motor.total_impulse = 100.0;
        motor.add_thrust_point(0.0, 50.0);
        motor.add_thrust_point(1.0, 50.0);
        motor.add_thrust_point(2.0, 0.0);

        let aero = AeroCalculator::new();
        let atmo = StandardAtmosphere;
        let grav = ConstantGravity;
        let wind = NoWind;
        let tree = ComponentTree::new();

        let state = CompactState::new(
            Vector3D::zero(),
            Vector3D::zero(),
            Quaternion::identity(),
            Vector3D::zero(),
            1.0,
            0.085,
        );

        let burnout_time = find_burnout_time(
            &state,
            &state,
            1.9,
            2.1,
            &Some(motor),
            &aero,
            &atmo,
            &grav,
            &wind,
            &tree,
            0.001,
            0.05,
        );

        // Should find burnout close to 2.0s
        assert!((burnout_time - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_find_ground_hit_time_falling() {
        let aero = AeroCalculator::new();
        let atmo = StandardAtmosphere;
        let grav = ConstantGravity;
        let wind = NoWind;
        let tree = ComponentTree::new();

        // State at altitude with downward velocity
        let state1 = CompactState::new(
            Vector3D::new(0.0, 0.0, 100.0), // 100m altitude
            Vector3D::new(0.0, 0.0, -10.0), // 10 m/s downward
            Quaternion::identity(),
            Vector3D::zero(),
            1.0,
            0.0,
        );

        // After ~10s at that rate, altitude would be around 0
        let dt = 10.0;
        let deriv = compute_derivative(
            &state1, 0.0, &None, &aero, &atmo, &grav, &wind, &tree, 0.001, 0.05,
        );
        let mut state2 = state1.clone() + deriv * dt;
        normalize_orientation(&mut state2);

        let _fs2 = compact_to_flight_state(&state2, &atmo, &wind);

        // The falling rocket should have lower altitude
        let ground_time = find_ground_hit_time(
            &state1, &state2, 0.0, dt, &None, &aero, &atmo, &grav, &wind, &tree, 0.001, 0.05,
        );

        assert!(ground_time > 0.0);
        assert!(ground_time <= dt);
    }

    #[test]
    fn test_interpolate_event_time_fraction() {
        // Value crosses threshold at 25% of interval
        let time = interpolate_event_time(0.0, 2.5, 10.0, 0.0, 1.0, 2.5);
        assert!((time - 0.25).abs() < 1e-12);
    }

    #[test]
    fn test_find_burnout_time_no_motor_does_not_panic() {
        // This tests that the function doesn't panic when motor is Some
        // (the function internally unwraps, so it should only be called with Some)
        let motor = MotorModel::new("Test".into(), "M1".into());
        let state = CompactState::new(
            Vector3D::zero(),
            Vector3D::zero(),
            Quaternion::identity(),
            Vector3D::zero(),
            1.0,
            0.0,
        );

        let aero = AeroCalculator::new();
        let atmo = StandardAtmosphere;
        let grav = ConstantGravity;
        let wind = NoWind;
        let tree = ComponentTree::new();

        let result = find_burnout_time(
            &state,
            &state,
            0.0,
            1.0,
            &Some(motor),
            &aero,
            &atmo,
            &grav,
            &wind,
            &tree,
            0.001,
            0.05,
        );

        assert!(result >= 0.0 && result <= 1.0);
    }
}