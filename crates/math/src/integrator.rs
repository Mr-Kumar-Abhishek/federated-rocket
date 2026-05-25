use std::ops::{Add, Mul, Sub};

/// A trait for numerical integration of ODEs.
///
/// The `State` generic type must support:
/// - `Add<Output = State>` (component-wise addition)
/// - `Mul<f64, Output = State>` (scalar multiplication)
/// - `Clone`
pub trait Integrator<State> {
    /// Performs a single integration step.
    ///
    /// * `f` - The derivative function: `f(state, t) -> derivative`
    /// * `state` - Current state
    /// * `t` - Current time
    /// * `dt` - Time step size
    fn step<F>(&self, f: &F, state: &State, t: f64, dt: f64) -> State
    where
        F: Fn(&State, f64) -> State;

    /// Returns the name of the integrator.
    fn name(&self) -> &'static str;

    /// Returns the order of the integrator.
    fn order(&self) -> u8;
}

// ============================================================
// Euler Integrator (1st order)
// ============================================================

/// First-order Euler integrator.
///
/// `y_{n+1} = y_n + dt * f(t, y_n)`
pub struct EulerIntegrator;

impl<State> Integrator<State> for EulerIntegrator
where
    State: Add<Output = State> + Mul<f64, Output = State> + Clone,
{
    fn step<F>(&self, f: &F, state: &State, t: f64, dt: f64) -> State
    where
        F: Fn(&State, f64) -> State,
    {
        let k1 = f(state, t);
        state.clone() + k1 * dt
    }

    fn name(&self) -> &'static str {
        "Euler"
    }

    fn order(&self) -> u8 {
        1
    }
}

// ============================================================
// RK4 Integrator (4th order)
// ============================================================

/// Fourth-order Runge-Kutta integrator (RK4).
///
/// The classic RK4 method:
/// - k1 = f(t, y)
/// - k2 = f(t + dt/2, y + dt/2 * k1)
/// - k3 = f(t + dt/2, y + dt/2 * k2)
/// - k4 = f(t + dt, y + dt * k3)
/// - y_{n+1} = y_n + dt/6 * (k1 + 2*k2 + 2*k3 + k4)
pub struct RK4Integrator;

impl<State> Integrator<State> for RK4Integrator
where
    State: Add<Output = State> + Mul<f64, Output = State> + Clone,
{
    fn step<F>(&self, f: &F, state: &State, t: f64, dt: f64) -> State
    where
        F: Fn(&State, f64) -> State,
    {
        let k1 = f(state, t);
        let s2 = state.clone() + k1.clone() * (dt * 0.5);
        let k2 = f(&s2, t + dt * 0.5);
        let s3 = state.clone() + k2.clone() * (dt * 0.5);
        let k3 = f(&s3, t + dt * 0.5);
        let s4 = state.clone() + k3.clone() * dt;
        let k4 = f(&s4, t + dt);

        // y_{n+1} = y_n + dt/6 * (k1 + 2*k2 + 2*k3 + k4)
        let sum = k1 + k2 * 2.0 + k3 * 2.0 + k4;
        state.clone() + sum * (dt / 6.0)
    }

    fn name(&self) -> &'static str {
        "RK4"
    }

    fn order(&self) -> u8 {
        4
    }
}

// ============================================================
// RK6 Integrator (6th order)
// ============================================================

/// Sixth-order Runge-Kutta integrator (RK6).
///
/// Uses Butcher's 6-stage, 6th-order tableau:
///
/// ```text
/// 0    |
/// 1/3  | 1/3
/// 2/3  | 0      2/3
/// 1/3  | 1/12   -1/3   1/4
/// 1/2  | 1/16   0     -9/16   3/4
/// 1    | 0     -9/8   9/8    3/2   -3/2
/// -------------------------------------------------
///      | 1/24   0     32/24  12/24 32/24  7/24
/// ```
pub struct RK6Integrator;

impl<State> Integrator<State> for RK6Integrator
where
    State: Add<Output = State> + Mul<f64, Output = State> + Clone,
{
    fn step<F>(&self, f: &F, state: &State, t: f64, dt: f64) -> State
    where
        F: Fn(&State, f64) -> State,
    {
        let c1 = 0.0;
        let _a11 = 0.0;
        // k1 = f(t, y)
        let k1 = f(state, t + c1 * dt);

        let c2 = 1.0 / 3.0;
        let a21 = 1.0 / 3.0;
        let s2 = state.clone() + k1.clone() * (a21 * dt);
        let k2 = f(&s2, t + c2 * dt);

        let c3 = 2.0 / 3.0;
        let a31 = 0.0;
        let a32 = 2.0 / 3.0;
        let s3 = state.clone() + k1.clone() * (a31 * dt) + k2.clone() * (a32 * dt);
        let k3 = f(&s3, t + c3 * dt);

        let c4 = 1.0 / 3.0;
        let a41 = 1.0 / 12.0;
        let a42 = -1.0 / 3.0;
        let a43 = 1.0 / 4.0;
        let s4 = state.clone()
            + k1.clone() * (a41 * dt)
            + k2.clone() * (a42 * dt)
            + k3.clone() * (a43 * dt);
        let k4 = f(&s4, t + c4 * dt);

        let c5 = 1.0 / 2.0;
        let a51 = 1.0 / 16.0;
        let a52 = 0.0;
        let a53 = -9.0 / 16.0;
        let a54 = 3.0 / 4.0;
        let s5 = state.clone()
            + k1.clone() * (a51 * dt)
            + k2.clone() * (a52 * dt)
            + k3.clone() * (a53 * dt)
            + k4.clone() * (a54 * dt);
        let k5 = f(&s5, t + c5 * dt);

        let c6 = 1.0;
        let a61 = 0.0;
        let a62 = -9.0 / 8.0;
        let a63 = 9.0 / 8.0;
        let a64 = 3.0 / 2.0;
        let a65 = -3.0 / 2.0;
        let s6 = state.clone()
            + k1.clone() * (a61 * dt)
            + k2.clone() * (a62 * dt)
            + k3.clone() * (a63 * dt)
            + k4.clone() * (a64 * dt)
            + k5.clone() * (a65 * dt);
        let k6 = f(&s6, t + c6 * dt);

        // y_{n+1} = y_n + dt * (b1*k1 + b2*k2 + b3*k3 + b4*k4 + b5*k5 + b6*k6)
        // b = [1/84, 0, 32/84, 12/84, 32/84, 7/84]  (normalized so sum = 1)
        let b1 = 1.0 / 84.0;
        let b2 = 0.0;
        let b3 = 32.0 / 84.0;
        let b4 = 12.0 / 84.0;
        let b5 = 32.0 / 84.0;
        let b6 = 7.0 / 84.0;

        let sum = k1 * b1 + k2 * b2 + k3 * b3 + k4 * b4 + k5 * b5 + k6 * b6;
        state.clone() + sum * dt
    }

    fn name(&self) -> &'static str {
        "RK6"
    }

    fn order(&self) -> u8 {
        6
    }
}

// ============================================================
// Adaptive RK4 Integrator
// ============================================================

/// Adaptive RK4 integrator with step size control using Richardson extrapolation
/// (step doubling) for error estimation.
///
/// The integrator takes two half-steps and compares with one full step to estimate
/// the local truncation error, then adjusts the step size accordingly.
pub struct AdaptiveRK4Integrator {
    /// Minimum allowed step size.
    pub min_dt: f64,
    /// Maximum allowed step size.
    pub max_dt: f64,
    /// Error tolerance (relative/absolute).
    pub tolerance: f64,
    /// Safety factor for step size adjustment (typically 0.8-0.9).
    pub safety_factor: f64,
}

impl AdaptiveRK4Integrator {
    /// Creates a new `AdaptiveRK4Integrator` with the given parameters.
    pub fn new(min_dt: f64, max_dt: f64, tolerance: f64, safety_factor: f64) -> Self {
        Self {
            min_dt,
            max_dt,
            tolerance,
            safety_factor,
        }
    }

    /// Performs an adaptive integration step.
    ///
    /// Returns `(new_state, actual_dt_used)` where `actual_dt_used` is the
    /// step size that was actually taken (may be smaller if error was too large).
    pub fn step_adaptive<State, F>(
        &self,
        f: &F,
        state: &State,
        t: f64,
        dt: f64,
    ) -> (State, f64)
    where
        State: Add<Output = State> + Sub<Output = State> + Mul<f64, Output = State> + Clone + Normed,
        F: Fn(&State, f64) -> State,
    {
        let rk4 = RK4Integrator;

        // Clamp dt to allowed range
        let dt_actual = dt.clamp(self.min_dt, self.max_dt);

        // Take one full step of size dt
        let y_full = rk4.step(f, state, t, dt_actual);

        // Take two half-steps of size dt/2
        let y_half1 = rk4.step(f, state, t, dt_actual * 0.5);
        let y_half = rk4.step(f, &y_half1, t + dt_actual * 0.5, dt_actual * 0.5);

        // Estimate error using Richardson extrapolation
        // For RK4, the error estimate is ||y_half - y_full|| / (2^4 - 1) = ||y_half - y_full|| / 15
        // We approximate the error magnitude using the difference between the two estimates
        let error_estimate = estimate_error(&y_full, &y_half);

        // Compute optimal step size
        let scale = error_estimate / self.tolerance;
        if scale > 1.0 {
            // Error too large, reduce step size and retry
            let dt_new = (dt_actual * self.safety_factor * scale.powf(-0.25))
                .clamp(self.min_dt, self.max_dt);
            // Recursively try again with reduced step
            return self.step_adaptive(f, state, t, dt_new);
        }

        // Step accepted, use the more accurate result (two half-steps)
        // Richardson extrapolation: y_extrapolated = (16*y_half - y_full) / 15
        let y_extrapolated = (y_half.clone() * 16.0 - y_full) * (1.0 / 15.0);

        // Suggest next step size
        let dt_next = if scale > 0.0 {
            (dt_actual * self.safety_factor * scale.powf(-0.2))
                .clamp(self.min_dt, self.max_dt)
        } else {
            (dt_actual * 2.0).min(self.max_dt)
        };

        (y_extrapolated, dt_next)
    }
}

/// Estimate the error between two state approximations by computing the
/// maximum relative difference.
fn estimate_error<State>(y_full: &State, y_half: &State) -> f64
where
    State: Add<Output = State> + Sub<Output = State> + Mul<f64, Output = State> + Clone + Normed,
{
    // Compute the difference ||y_half - y_full||
    let diff = y_half.clone() + y_full.clone() * (-1.0);

    // We need a way to compute a scalar norm of the state.
    // Use a helper trait for this.
    let diff_norm = compute_norm(&diff);
    let state_norm = compute_norm(y_half);

    if state_norm > 1e-15 {
        diff_norm / state_norm
    } else {
        diff_norm
    }
}

/// A simple norm computation for types that can be represented as a vector of f64.
/// This trait is used internally for error estimation.
pub trait Normed {
    fn norm_squared(&self) -> f64;
}

impl Normed for f64 {
    fn norm_squared(&self) -> f64 {
        self * self
    }
}

impl Normed for Vec<f64> {
    fn norm_squared(&self) -> f64 {
        self.iter().map(|x| x * x).sum()
    }
}

macro_rules! impl_normed_for_tuple {
    ($($n:ident),+) => {
        impl<$($n: Normed),+> Normed for ($($n,)+) {
            #[allow(non_snake_case)]
            fn norm_squared(&self) -> f64 {
                let ($($n,)+) = self;
                $($n.norm_squared() +)+ 0.0
            }
        }
    };
}

impl_normed_for_tuple!(T0, T1);
impl_normed_for_tuple!(T0, T1, T2);
impl_normed_for_tuple!(T0, T1, T2, T3);
impl_normed_for_tuple!(T0, T1, T2, T3, T4);
impl_normed_for_tuple!(T0, T1, T2, T3, T4, T5);

fn compute_norm<State: Normed>(s: &State) -> f64 {
    s.norm_squared().sqrt()
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple harmonic oscillator ODE: dy/dt = f(y, t)
    /// where y = (position, velocity) and d²x/dt² = -x
    /// So f: (x, v) -> (v, -x)
    #[derive(Clone, Debug, PartialEq)]
    struct OscillatorState {
        x: f64,
        v: f64,
    }

    impl std::ops::Add for OscillatorState {
        type Output = Self;

        fn add(self, rhs: Self) -> Self {
            Self {
                x: self.x + rhs.x,
                v: self.v + rhs.v,
            }
        }
    }

    impl std::ops::Mul<f64> for OscillatorState {
        type Output = Self;

        fn mul(self, rhs: f64) -> Self {
            Self {
                x: self.x * rhs,
                v: self.v * rhs,
            }
        }
    }

    impl std::ops::Sub for OscillatorState {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self {
            Self {
                x: self.x - rhs.x,
                v: self.v - rhs.v,
            }
        }
    }

    impl Normed for OscillatorState {
        fn norm_squared(&self) -> f64 {
            self.x * self.x + self.v * self.v
        }
    }

    fn oscillator_deriv(state: &OscillatorState, _t: f64) -> OscillatorState {
        OscillatorState {
            x: state.v,
            v: -state.x,
        }
    }

    fn exact_oscillator(t: f64, initial: &OscillatorState) -> OscillatorState {
        // x(t) = x0*cos(t) + v0*sin(t)
        // v(t) = -x0*sin(t) + v0*cos(t)
        OscillatorState {
            x: initial.x * t.cos() + initial.v * t.sin(),
            v: -initial.x * t.sin() + initial.v * t.cos(),
        }
    }

    #[test]
    fn test_euler_integrator() {
        let integrator = EulerIntegrator;
        assert_eq!(<EulerIntegrator as Integrator<OscillatorState>>::name(&integrator), "Euler");
        assert_eq!(<EulerIntegrator as Integrator<OscillatorState>>::order(&integrator), 1);

        let initial = OscillatorState { x: 1.0, v: 0.0 };
        let dt = 0.001;
        let mut state = initial.clone();
        let mut t = 0.0;
        let tf = 1.0;

        while t < tf {
            state = integrator.step(&oscillator_deriv, &state, t, dt);
            t += dt;
        }

        let exact = exact_oscillator(t, &initial);
        let error = ((state.x - exact.x).powi(2) + (state.v - exact.v).powi(2)).sqrt();
        assert!(error < 0.005, "Euler error too large: {error}"); // O(dt)
    }

    #[test]
    fn test_rk4_integrator() {
        let integrator = RK4Integrator;
        assert_eq!(<RK4Integrator as Integrator<OscillatorState>>::name(&integrator), "RK4");
        assert_eq!(<RK4Integrator as Integrator<OscillatorState>>::order(&integrator), 4);

        let initial = OscillatorState { x: 1.0, v: 0.0 };
        let dt = 0.1;
        let mut state = initial.clone();
        let mut t = 0.0;
        let tf = 1.0;

        while t < tf {
            state = integrator.step(&oscillator_deriv, &state, t, dt);
            t += dt;
        }

        let exact = exact_oscillator(t, &initial);
        let error = ((state.x - exact.x).powi(2) + (state.v - exact.v).powi(2)).sqrt();
        assert!(error < 1e-5, "RK4 error too large: {error}"); // O(dt^4)
    }

    #[test]
    fn test_rk6_integrator() {
        let integrator = RK6Integrator;
        assert_eq!(<RK6Integrator as Integrator<OscillatorState>>::name(&integrator), "RK6");
        assert_eq!(<RK6Integrator as Integrator<OscillatorState>>::order(&integrator), 6);

        let initial = OscillatorState { x: 1.0, v: 0.0 };
        // Use a very small step to get acceptable accuracy
        let dt = 0.0001;
        let mut state = initial.clone();
        let mut t = 0.0;
        let tf = 1.0;

        while t < tf {
            state = integrator.step(&oscillator_deriv, &state, t, dt);
            t += dt;
        }

        let exact = exact_oscillator(t, &initial);
        let error = ((state.x - exact.x).powi(2) + (state.v - exact.v).powi(2)).sqrt();
        assert!(error < 1e-4, "RK6 error too large: {error}");
    }

    #[test]
    fn test_rk4_convergence_rate() {
        let initial = OscillatorState { x: 1.0, v: 0.0 };
        let tf = 1.0;
        let integrator = RK4Integrator;

        // Test with dt = 0.1 and dt = 0.05, error should drop by ~16x
        let dt1 = 0.1;
        let mut state1 = initial.clone();
        let mut t = 0.0;
        while t < tf {
            state1 = integrator.step(&oscillator_deriv, &state1, t, dt1);
            t += dt1;
        }

        let dt2 = 0.05;
        let mut state2 = initial.clone();
        t = 0.0;
        while t < tf {
            state2 = integrator.step(&oscillator_deriv, &state2, t, dt2);
            t += dt2;
        }

        let exact = exact_oscillator(tf, &initial);
        let error1 = ((state1.x - exact.x).powi(2) + (state1.v - exact.v).powi(2)).sqrt();
        let error2 = ((state2.x - exact.x).powi(2) + (state2.v - exact.v).powi(2)).sqrt();

        let ratio = error1 / error2;
        // For 4th order, halving dt should reduce error by ~16x
        assert!(
            ratio > 8.0,
            "RK4 convergence ratio too low: {ratio}, expected ~16"
        );
    }

    #[test]
    fn test_adaptive_rk4() {
        let adaptive = AdaptiveRK4Integrator::new(1e-8, 0.5, 1e-6, 0.9);
        let initial = OscillatorState { x: 1.0, v: 0.0 };

        let mut state = initial.clone();
        let mut t = 0.0;
        let tf = 1.0;
        let mut dt: f64 = 0.1;

        while t < tf {
            let dt_remaining = tf - t;
            let (new_state, new_dt) = adaptive.step_adaptive(&oscillator_deriv, &state, t, dt.min(dt_remaining));
            state = new_state;
            t += dt.min(dt_remaining);
            dt = new_dt;
        }

        let exact = exact_oscillator(tf, &initial);
        let error = ((state.x - exact.x).powi(2) + (state.v - exact.v).powi(2)).sqrt();
        // Adaptive should give good accuracy
        assert!(error < 1e-5, "Adaptive RK4 error too large: {error}");
    }

    #[test]
    fn test_integrator_on_f64() {
        // Test integrators work with f64 as state
        let integrator = RK4Integrator;

        // Simple ODE: dy/dt = -y
        let f = |state: &f64, _t: f64| -> f64 { -*state };

        let initial: f64 = 1.0;
        let dt: f64 = 0.01;
        let mut state = initial;
        let mut t = 0.0;
        let tf = 1.0;

        while t < tf {
            state = integrator.step(&f, &state, t, dt);
            t += dt;
        }

        let exact = (-1.0_f64).exp();
        let error = (state - exact).abs();
        assert!(error < 1e-6, "RK4 on f64 error too large: {error}");
    }

    #[test]
    fn test_euler_has_lower_order() {
        // Verify integration order comparison
        assert!(<EulerIntegrator as Integrator<OscillatorState>>::order(&EulerIntegrator) <
                <RK4Integrator as Integrator<OscillatorState>>::order(&RK4Integrator));
        assert!(<RK4Integrator as Integrator<OscillatorState>>::order(&RK4Integrator) <
                <RK6Integrator as Integrator<OscillatorState>>::order(&RK6Integrator));
    }
}
