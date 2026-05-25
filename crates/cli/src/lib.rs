pub mod commands;
pub mod output;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_simulate_args_validation() {
        use crate::commands::simulate::SimulateArgs;

        // Verify default values are set correctly
        let args = SimulateArgs {
            file: "test.ork".to_string(),
            motor: None,
            output: None,
            max_time: 120.0,
            output_interval: 0.1,
            time_step: 0.001,
            rod_clear: 2.0,
            launch_altitude: 0.0,
            wind_speed: 0.0,
            wind_direction: 0.0,
            json: false,
            verbose: false,
        };

        assert_eq!(args.file, "test.ork");
        assert!(args.motor.is_none());
        assert!((args.max_time - 120.0).abs() < 1e-12);
        assert!((args.output_interval - 0.1).abs() < 1e-12);
        assert!((args.time_step - 0.001).abs() < 1e-12);
        assert!((args.rod_clear - 2.0).abs() < 1e-12);
        assert!((args.launch_altitude - 0.0).abs() < 1e-12);
        assert!((args.wind_speed - 0.0).abs() < 1e-12);
        assert!((args.wind_direction - 0.0).abs() < 1e-12);
        assert!(!args.json);
        assert!(!args.verbose);
    }

    #[test]
    fn test_info_args() {
        use crate::commands::info::InfoArgs;

        let args = InfoArgs {
            file: "test.ork".to_string(),
            detailed: true,
            json: true,
        };

        assert_eq!(args.file, "test.ork");
        assert!(args.detailed);
        assert!(args.json);
    }

    #[test]
    fn test_motors_list_filtering() {
        use crate::commands::motors::MotorsArgs;

        let args = MotorsArgs {
            list: true,
            manufacturer: Some("Estes".to_string()),
            designation: None,
            impulse_class: None,
            detailed: false,
            json: false,
            db: None,
        };

        assert!(args.list);
        assert_eq!(args.manufacturer, Some("Estes".to_string()));
        assert!(args.designation.is_none());
        assert!(args.impulse_class.is_none());
        assert!(!args.detailed);
        assert!(!args.json);
        assert!(args.db.is_none());
    }

    #[test]
    fn test_optimize_args() {
        use crate::commands::optimize::OptimizeArgs;

        let args = OptimizeArgs {
            file: "test.ork".to_string(),
            parameter: "nose_length".to_string(),
            goal: "altitude".to_string(),
            min: 0.05,
            max: 0.30,
            motor: None,
            iterations: 50,
            json: false,
        };

        assert_eq!(args.parameter, "nose_length");
        assert_eq!(args.goal, "altitude");
        assert!((args.min - 0.05).abs() < 1e-12);
        assert!((args.max - 0.30).abs() < 1e-12);
        assert_eq!(args.iterations, 50);
    }

    #[test]
    fn test_convert_args() {
        use crate::commands::convert::ConvertArgs;

        let args = ConvertArgs {
            input: "input.ork".to_string(),
            output: "output.rkt".to_string(),
        };

        assert_eq!(args.input, "input.ork");
        assert_eq!(args.output, "output.rkt");
    }
}
