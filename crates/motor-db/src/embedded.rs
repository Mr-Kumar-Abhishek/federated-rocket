use crate::types::{Motor, MotorType, ThrustPoint};

/// Returns the list of embedded (built-in) motors that ship with the application.
pub fn embedded_motors() -> Vec<Motor> {
    vec![
        // Estes motors
        motor_estes_a8_3(),
        motor_estes_b4_4(),
        motor_estes_b6_4(),
        motor_estes_c6_5(),
        motor_estes_c6_7(),
        motor_estes_d12_5(),
        motor_estes_e12_4(),
        motor_estes_f15_6(),
        // Aerotech motors
        motor_aerotech_e23_5(),
        motor_aerotech_f24_7(),
        motor_aerotech_g25_10(),
        motor_aerotech_h13_15(),
        motor_aerotech_i161_14(),
        motor_aerotech_j250_15(),
        // Quest motors
        motor_quest_b4_4(),
        motor_quest_c6_5(),
        motor_quest_d16_6(),
        // Additional common motors
        motor_estes_b6_6(),
        motor_estes_d12_3(),
        motor_aerotech_f35_6(),
        motor_quest_c12_7(),
        motor_estes_c11_3(),
    ]
}

// ─── Estes Motors ───────────────────────────────────────────────────────────

fn motor_estes_a8_3() -> Motor {
    Motor {
        id: None,
        manufacturer: "Estes".to_string(),
        manufacturer_abbrev: "EST".to_string(),
        designation: "A8-3".to_string(),
        motor_type: MotorType::Solid,
        diameter: 18.0,
        length: 70.0,
        total_impulse: 2.50,
        burn_time: 0.75,
        avg_thrust: 3.33,
        max_thrust: 9.0,
        propellant_mass: 3.4,
        dry_mass: 4.0,
        delay_time: 3.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.04, thrust: 7.0 },
            ThrustPoint { time: 0.10, thrust: 9.0 },
            ThrustPoint { time: 0.20, thrust: 7.5 },
            ThrustPoint { time: 0.35, thrust: 4.0 },
            ThrustPoint { time: 0.50, thrust: 3.0 },
            ThrustPoint { time: 0.65, thrust: 2.0 },
            ThrustPoint { time: 0.75, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_estes_b4_4() -> Motor {
    Motor {
        id: None,
        manufacturer: "Estes".to_string(),
        manufacturer_abbrev: "EST".to_string(),
        designation: "B4-4".to_string(),
        motor_type: MotorType::Solid,
        diameter: 18.0,
        length: 70.0,
        total_impulse: 4.40,
        burn_time: 1.10,
        avg_thrust: 4.0,
        max_thrust: 11.5,
        propellant_mass: 6.5,
        dry_mass: 7.0,
        delay_time: 4.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.04, thrust: 8.0 },
            ThrustPoint { time: 0.10, thrust: 11.5 },
            ThrustPoint { time: 0.25, thrust: 9.0 },
            ThrustPoint { time: 0.45, thrust: 5.0 },
            ThrustPoint { time: 0.70, thrust: 3.5 },
            ThrustPoint { time: 0.95, thrust: 2.0 },
            ThrustPoint { time: 1.10, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_estes_b6_4() -> Motor {
    Motor {
        id: None,
        manufacturer: "Estes".to_string(),
        manufacturer_abbrev: "EST".to_string(),
        designation: "B6-4".to_string(),
        motor_type: MotorType::Solid,
        diameter: 18.0,
        length: 70.0,
        total_impulse: 4.35,
        burn_time: 0.75,
        avg_thrust: 5.8,
        max_thrust: 12.5,
        propellant_mass: 5.5,
        dry_mass: 6.0,
        delay_time: 4.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.03, thrust: 9.0 },
            ThrustPoint { time: 0.08, thrust: 12.5 },
            ThrustPoint { time: 0.15, thrust: 11.0 },
            ThrustPoint { time: 0.30, thrust: 8.0 },
            ThrustPoint { time: 0.50, thrust: 4.0 },
            ThrustPoint { time: 0.65, thrust: 2.0 },
            ThrustPoint { time: 0.75, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_estes_b6_6() -> Motor {
    Motor {
        id: None,
        manufacturer: "Estes".to_string(),
        manufacturer_abbrev: "EST".to_string(),
        designation: "B6-6".to_string(),
        motor_type: MotorType::Solid,
        diameter: 18.0,
        length: 70.0,
        total_impulse: 4.35,
        burn_time: 0.75,
        avg_thrust: 5.8,
        max_thrust: 12.5,
        propellant_mass: 5.5,
        dry_mass: 6.0,
        delay_time: 6.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.03, thrust: 9.0 },
            ThrustPoint { time: 0.08, thrust: 12.5 },
            ThrustPoint { time: 0.15, thrust: 11.0 },
            ThrustPoint { time: 0.30, thrust: 8.0 },
            ThrustPoint { time: 0.50, thrust: 4.0 },
            ThrustPoint { time: 0.65, thrust: 2.0 },
            ThrustPoint { time: 0.75, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_estes_c6_5() -> Motor {
    Motor {
        id: None,
        manufacturer: "Estes".to_string(),
        manufacturer_abbrev: "EST".to_string(),
        designation: "C6-5".to_string(),
        motor_type: MotorType::Solid,
        diameter: 18.0,
        length: 70.0,
        total_impulse: 8.82,
        burn_time: 1.60,
        avg_thrust: 5.5,
        max_thrust: 12.0,
        propellant_mass: 10.5,
        dry_mass: 11.0,
        delay_time: 5.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.05, thrust: 8.0 },
            ThrustPoint { time: 0.10, thrust: 12.0 },
            ThrustPoint { time: 0.20, thrust: 10.0 },
            ThrustPoint { time: 0.40, thrust: 8.0 },
            ThrustPoint { time: 0.60, thrust: 7.0 },
            ThrustPoint { time: 0.80, thrust: 6.0 },
            ThrustPoint { time: 1.00, thrust: 5.0 },
            ThrustPoint { time: 1.20, thrust: 4.0 },
            ThrustPoint { time: 1.40, thrust: 3.0 },
            ThrustPoint { time: 1.50, thrust: 2.0 },
            ThrustPoint { time: 1.55, thrust: 1.0 },
            ThrustPoint { time: 1.60, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_estes_c6_7() -> Motor {
    Motor {
        id: None,
        manufacturer: "Estes".to_string(),
        manufacturer_abbrev: "EST".to_string(),
        designation: "C6-7".to_string(),
        motor_type: MotorType::Solid,
        diameter: 18.0,
        length: 70.0,
        total_impulse: 8.82,
        burn_time: 1.60,
        avg_thrust: 5.5,
        max_thrust: 12.0,
        propellant_mass: 10.5,
        dry_mass: 11.0,
        delay_time: 7.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.05, thrust: 8.0 },
            ThrustPoint { time: 0.10, thrust: 12.0 },
            ThrustPoint { time: 0.20, thrust: 10.0 },
            ThrustPoint { time: 0.40, thrust: 8.0 },
            ThrustPoint { time: 0.60, thrust: 7.0 },
            ThrustPoint { time: 0.80, thrust: 6.0 },
            ThrustPoint { time: 1.00, thrust: 5.0 },
            ThrustPoint { time: 1.20, thrust: 4.0 },
            ThrustPoint { time: 1.40, thrust: 3.0 },
            ThrustPoint { time: 1.50, thrust: 2.0 },
            ThrustPoint { time: 1.55, thrust: 1.0 },
            ThrustPoint { time: 1.60, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_estes_c11_3() -> Motor {
    Motor {
        id: None,
        manufacturer: "Estes".to_string(),
        manufacturer_abbrev: "EST".to_string(),
        designation: "C11-3".to_string(),
        motor_type: MotorType::Solid,
        diameter: 24.0,
        length: 70.0,
        total_impulse: 9.60,
        burn_time: 1.20,
        avg_thrust: 8.0,
        max_thrust: 16.0,
        propellant_mass: 12.0,
        dry_mass: 12.5,
        delay_time: 3.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.04, thrust: 10.0 },
            ThrustPoint { time: 0.10, thrust: 16.0 },
            ThrustPoint { time: 0.20, thrust: 14.0 },
            ThrustPoint { time: 0.40, thrust: 11.0 },
            ThrustPoint { time: 0.60, thrust: 9.0 },
            ThrustPoint { time: 0.80, thrust: 7.0 },
            ThrustPoint { time: 1.00, thrust: 5.0 },
            ThrustPoint { time: 1.15, thrust: 2.0 },
            ThrustPoint { time: 1.20, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_estes_d12_5() -> Motor {
    Motor {
        id: None,
        manufacturer: "Estes".to_string(),
        manufacturer_abbrev: "EST".to_string(),
        designation: "D12-5".to_string(),
        motor_type: MotorType::Solid,
        diameter: 24.0,
        length: 70.0,
        total_impulse: 16.85,
        burn_time: 1.60,
        avg_thrust: 10.5,
        max_thrust: 28.0,
        propellant_mass: 20.6,
        dry_mass: 18.0,
        delay_time: 5.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.04, thrust: 15.0 },
            ThrustPoint { time: 0.10, thrust: 28.0 },
            ThrustPoint { time: 0.20, thrust: 24.0 },
            ThrustPoint { time: 0.40, thrust: 18.0 },
            ThrustPoint { time: 0.60, thrust: 14.0 },
            ThrustPoint { time: 0.80, thrust: 11.0 },
            ThrustPoint { time: 1.00, thrust: 9.0 },
            ThrustPoint { time: 1.20, thrust: 7.0 },
            ThrustPoint { time: 1.40, thrust: 4.0 },
            ThrustPoint { time: 1.55, thrust: 2.0 },
            ThrustPoint { time: 1.60, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_estes_d12_3() -> Motor {
    Motor {
        id: None,
        manufacturer: "Estes".to_string(),
        manufacturer_abbrev: "EST".to_string(),
        designation: "D12-3".to_string(),
        motor_type: MotorType::Solid,
        diameter: 24.0,
        length: 70.0,
        total_impulse: 16.85,
        burn_time: 1.60,
        avg_thrust: 10.5,
        max_thrust: 28.0,
        propellant_mass: 20.6,
        dry_mass: 18.0,
        delay_time: 3.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.04, thrust: 15.0 },
            ThrustPoint { time: 0.10, thrust: 28.0 },
            ThrustPoint { time: 0.20, thrust: 24.0 },
            ThrustPoint { time: 0.40, thrust: 18.0 },
            ThrustPoint { time: 0.60, thrust: 14.0 },
            ThrustPoint { time: 0.80, thrust: 11.0 },
            ThrustPoint { time: 1.00, thrust: 9.0 },
            ThrustPoint { time: 1.20, thrust: 7.0 },
            ThrustPoint { time: 1.40, thrust: 4.0 },
            ThrustPoint { time: 1.55, thrust: 2.0 },
            ThrustPoint { time: 1.60, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_estes_e12_4() -> Motor {
    Motor {
        id: None,
        manufacturer: "Estes".to_string(),
        manufacturer_abbrev: "EST".to_string(),
        designation: "E12-4".to_string(),
        motor_type: MotorType::Solid,
        diameter: 24.0,
        length: 90.0,
        total_impulse: 28.06,
        burn_time: 2.10,
        avg_thrust: 13.4,
        max_thrust: 25.0,
        propellant_mass: 32.5,
        dry_mass: 28.0,
        delay_time: 4.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.05, thrust: 18.0 },
            ThrustPoint { time: 0.15, thrust: 25.0 },
            ThrustPoint { time: 0.30, thrust: 22.0 },
            ThrustPoint { time: 0.50, thrust: 18.0 },
            ThrustPoint { time: 0.80, thrust: 15.0 },
            ThrustPoint { time: 1.10, thrust: 12.0 },
            ThrustPoint { time: 1.40, thrust: 9.0 },
            ThrustPoint { time: 1.70, thrust: 6.0 },
            ThrustPoint { time: 1.95, thrust: 3.0 },
            ThrustPoint { time: 2.10, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_estes_f15_6() -> Motor {
    Motor {
        id: None,
        manufacturer: "Estes".to_string(),
        manufacturer_abbrev: "EST".to_string(),
        designation: "F15-6".to_string(),
        motor_type: MotorType::Solid,
        diameter: 29.0,
        length: 90.0,
        total_impulse: 53.00,
        burn_time: 3.50,
        avg_thrust: 15.1,
        max_thrust: 30.0,
        propellant_mass: 55.0,
        dry_mass: 45.0,
        delay_time: 6.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.04, thrust: 22.0 },
            ThrustPoint { time: 0.10, thrust: 30.0 },
            ThrustPoint { time: 0.30, thrust: 26.0 },
            ThrustPoint { time: 0.60, thrust: 22.0 },
            ThrustPoint { time: 1.00, thrust: 18.0 },
            ThrustPoint { time: 1.50, thrust: 14.0 },
            ThrustPoint { time: 2.00, thrust: 12.0 },
            ThrustPoint { time: 2.50, thrust: 9.0 },
            ThrustPoint { time: 3.00, thrust: 6.0 },
            ThrustPoint { time: 3.30, thrust: 3.0 },
            ThrustPoint { time: 3.50, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

// ─── Aerotech Motors ────────────────────────────────────────────────────────

fn motor_aerotech_e23_5() -> Motor {
    Motor {
        id: None,
        manufacturer: "Aerotech".to_string(),
        manufacturer_abbrev: "AT".to_string(),
        designation: "E23-5".to_string(),
        motor_type: MotorType::Solid,
        diameter: 24.0,
        length: 70.0,
        total_impulse: 37.00,
        burn_time: 1.60,
        avg_thrust: 23.0,
        max_thrust: 42.0,
        propellant_mass: 30.0,
        dry_mass: 28.0,
        delay_time: 5.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.02, thrust: 25.0 },
            ThrustPoint { time: 0.05, thrust: 42.0 },
            ThrustPoint { time: 0.10, thrust: 40.0 },
            ThrustPoint { time: 0.25, thrust: 35.0 },
            ThrustPoint { time: 0.50, thrust: 30.0 },
            ThrustPoint { time: 0.80, thrust: 25.0 },
            ThrustPoint { time: 1.10, thrust: 20.0 },
            ThrustPoint { time: 1.35, thrust: 15.0 },
            ThrustPoint { time: 1.55, thrust: 8.0 },
            ThrustPoint { time: 1.60, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_aerotech_f24_7() -> Motor {
    Motor {
        id: None,
        manufacturer: "Aerotech".to_string(),
        manufacturer_abbrev: "AT".to_string(),
        designation: "F24-7".to_string(),
        motor_type: MotorType::Solid,
        diameter: 29.0,
        length: 90.0,
        total_impulse: 55.00,
        burn_time: 2.30,
        avg_thrust: 24.0,
        max_thrust: 45.0,
        propellant_mass: 50.0,
        dry_mass: 42.0,
        delay_time: 7.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.03, thrust: 30.0 },
            ThrustPoint { time: 0.08, thrust: 45.0 },
            ThrustPoint { time: 0.20, thrust: 40.0 },
            ThrustPoint { time: 0.50, thrust: 35.0 },
            ThrustPoint { time: 0.90, thrust: 28.0 },
            ThrustPoint { time: 1.30, thrust: 22.0 },
            ThrustPoint { time: 1.70, thrust: 18.0 },
            ThrustPoint { time: 2.00, thrust: 12.0 },
            ThrustPoint { time: 2.20, thrust: 5.0 },
            ThrustPoint { time: 2.30, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_aerotech_f35_6() -> Motor {
    Motor {
        id: None,
        manufacturer: "Aerotech".to_string(),
        manufacturer_abbrev: "AT".to_string(),
        designation: "F35-6".to_string(),
        motor_type: MotorType::Solid,
        diameter: 29.0,
        length: 90.0,
        total_impulse: 70.00,
        burn_time: 2.00,
        avg_thrust: 35.0,
        max_thrust: 55.0,
        propellant_mass: 60.0,
        dry_mass: 50.0,
        delay_time: 6.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.02, thrust: 40.0 },
            ThrustPoint { time: 0.06, thrust: 55.0 },
            ThrustPoint { time: 0.15, thrust: 50.0 },
            ThrustPoint { time: 0.40, thrust: 45.0 },
            ThrustPoint { time: 0.80, thrust: 38.0 },
            ThrustPoint { time: 1.20, thrust: 32.0 },
            ThrustPoint { time: 1.60, thrust: 22.0 },
            ThrustPoint { time: 1.85, thrust: 12.0 },
            ThrustPoint { time: 2.00, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_aerotech_g25_10() -> Motor {
    Motor {
        id: None,
        manufacturer: "Aerotech".to_string(),
        manufacturer_abbrev: "AT".to_string(),
        designation: "G25-10".to_string(),
        motor_type: MotorType::Solid,
        diameter: 29.0,
        length: 120.0,
        total_impulse: 120.00,
        burn_time: 4.80,
        avg_thrust: 25.0,
        max_thrust: 40.0,
        propellant_mass: 100.0,
        dry_mass: 85.0,
        delay_time: 10.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.05, thrust: 28.0 },
            ThrustPoint { time: 0.15, thrust: 40.0 },
            ThrustPoint { time: 0.40, thrust: 36.0 },
            ThrustPoint { time: 0.80, thrust: 32.0 },
            ThrustPoint { time: 1.50, thrust: 28.0 },
            ThrustPoint { time: 2.20, thrust: 25.0 },
            ThrustPoint { time: 3.00, thrust: 22.0 },
            ThrustPoint { time: 3.50, thrust: 20.0 },
            ThrustPoint { time: 4.00, thrust: 16.0 },
            ThrustPoint { time: 4.40, thrust: 10.0 },
            ThrustPoint { time: 4.70, thrust: 4.0 },
            ThrustPoint { time: 4.80, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_aerotech_h13_15() -> Motor {
    Motor {
        id: None,
        manufacturer: "Aerotech".to_string(),
        manufacturer_abbrev: "AT".to_string(),
        designation: "H13-15".to_string(),
        motor_type: MotorType::Solid,
        diameter: 29.0,
        length: 230.0,
        total_impulse: 180.00,
        burn_time: 13.80,
        avg_thrust: 13.0,
        max_thrust: 25.0,
        propellant_mass: 180.0,
        dry_mass: 140.0,
        delay_time: 15.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.10, thrust: 18.0 },
            ThrustPoint { time: 0.30, thrust: 25.0 },
            ThrustPoint { time: 0.80, thrust: 22.0 },
            ThrustPoint { time: 2.00, thrust: 18.0 },
            ThrustPoint { time: 4.00, thrust: 15.0 },
            ThrustPoint { time: 6.00, thrust: 13.0 },
            ThrustPoint { time: 8.00, thrust: 11.0 },
            ThrustPoint { time: 10.00, thrust: 9.0 },
            ThrustPoint { time: 12.00, thrust: 6.0 },
            ThrustPoint { time: 13.50, thrust: 3.0 },
            ThrustPoint { time: 13.80, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_aerotech_i161_14() -> Motor {
    Motor {
        id: None,
        manufacturer: "Aerotech".to_string(),
        manufacturer_abbrev: "AT".to_string(),
        designation: "I161-14".to_string(),
        motor_type: MotorType::Solid,
        diameter: 38.0,
        length: 300.0,
        total_impulse: 460.00,
        burn_time: 2.85,
        avg_thrust: 161.0,
        max_thrust: 280.0,
        propellant_mass: 350.0,
        dry_mass: 250.0,
        delay_time: 14.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.02, thrust: 150.0 },
            ThrustPoint { time: 0.05, thrust: 280.0 },
            ThrustPoint { time: 0.10, thrust: 260.0 },
            ThrustPoint { time: 0.30, thrust: 230.0 },
            ThrustPoint { time: 0.60, thrust: 200.0 },
            ThrustPoint { time: 1.00, thrust: 180.0 },
            ThrustPoint { time: 1.40, thrust: 160.0 },
            ThrustPoint { time: 1.80, thrust: 140.0 },
            ThrustPoint { time: 2.20, thrust: 110.0 },
            ThrustPoint { time: 2.60, thrust: 70.0 },
            ThrustPoint { time: 2.80, thrust: 30.0 },
            ThrustPoint { time: 2.85, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_aerotech_j250_15() -> Motor {
    Motor {
        id: None,
        manufacturer: "Aerotech".to_string(),
        manufacturer_abbrev: "AT".to_string(),
        designation: "J250-15".to_string(),
        motor_type: MotorType::Solid,
        diameter: 38.0,
        length: 400.0,
        total_impulse: 720.00,
        burn_time: 2.88,
        avg_thrust: 250.0,
        max_thrust: 420.0,
        propellant_mass: 500.0,
        dry_mass: 350.0,
        delay_time: 15.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.02, thrust: 250.0 },
            ThrustPoint { time: 0.05, thrust: 420.0 },
            ThrustPoint { time: 0.10, thrust: 400.0 },
            ThrustPoint { time: 0.25, thrust: 360.0 },
            ThrustPoint { time: 0.50, thrust: 320.0 },
            ThrustPoint { time: 0.80, thrust: 290.0 },
            ThrustPoint { time: 1.20, thrust: 260.0 },
            ThrustPoint { time: 1.60, thrust: 230.0 },
            ThrustPoint { time: 2.00, thrust: 190.0 },
            ThrustPoint { time: 2.40, thrust: 140.0 },
            ThrustPoint { time: 2.70, thrust: 60.0 },
            ThrustPoint { time: 2.88, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

// ─── Quest Motors ───────────────────────────────────────────────────────────

fn motor_quest_b4_4() -> Motor {
    Motor {
        id: None,
        manufacturer: "Quest".to_string(),
        manufacturer_abbrev: "QST".to_string(),
        designation: "B4-4".to_string(),
        motor_type: MotorType::Solid,
        diameter: 18.0,
        length: 70.0,
        total_impulse: 4.35,
        burn_time: 1.20,
        avg_thrust: 3.6,
        max_thrust: 10.0,
        propellant_mass: 6.0,
        dry_mass: 6.5,
        delay_time: 4.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.04, thrust: 7.0 },
            ThrustPoint { time: 0.10, thrust: 10.0 },
            ThrustPoint { time: 0.25, thrust: 8.0 },
            ThrustPoint { time: 0.50, thrust: 5.0 },
            ThrustPoint { time: 0.75, thrust: 3.5 },
            ThrustPoint { time: 1.00, thrust: 2.0 },
            ThrustPoint { time: 1.20, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_quest_c6_5() -> Motor {
    Motor {
        id: None,
        manufacturer: "Quest".to_string(),
        manufacturer_abbrev: "QST".to_string(),
        designation: "C6-5".to_string(),
        motor_type: MotorType::Solid,
        diameter: 18.0,
        length: 70.0,
        total_impulse: 8.50,
        burn_time: 1.70,
        avg_thrust: 5.0,
        max_thrust: 11.0,
        propellant_mass: 10.0,
        dry_mass: 10.5,
        delay_time: 5.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.05, thrust: 7.0 },
            ThrustPoint { time: 0.12, thrust: 11.0 },
            ThrustPoint { time: 0.30, thrust: 9.0 },
            ThrustPoint { time: 0.55, thrust: 7.0 },
            ThrustPoint { time: 0.85, thrust: 5.5 },
            ThrustPoint { time: 1.10, thrust: 4.0 },
            ThrustPoint { time: 1.35, thrust: 3.0 },
            ThrustPoint { time: 1.55, thrust: 2.0 },
            ThrustPoint { time: 1.70, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_quest_c12_7() -> Motor {
    Motor {
        id: None,
        manufacturer: "Quest".to_string(),
        manufacturer_abbrev: "QST".to_string(),
        designation: "C12-7".to_string(),
        motor_type: MotorType::Solid,
        diameter: 24.0,
        length: 70.0,
        total_impulse: 9.50,
        burn_time: 0.80,
        avg_thrust: 11.8,
        max_thrust: 22.0,
        propellant_mass: 11.0,
        dry_mass: 11.5,
        delay_time: 7.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.03, thrust: 14.0 },
            ThrustPoint { time: 0.08, thrust: 22.0 },
            ThrustPoint { time: 0.18, thrust: 19.0 },
            ThrustPoint { time: 0.35, thrust: 15.0 },
            ThrustPoint { time: 0.55, thrust: 11.0 },
            ThrustPoint { time: 0.70, thrust: 6.0 },
            ThrustPoint { time: 0.80, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

fn motor_quest_d16_6() -> Motor {
    Motor {
        id: None,
        manufacturer: "Quest".to_string(),
        manufacturer_abbrev: "QST".to_string(),
        designation: "D16-6".to_string(),
        motor_type: MotorType::Solid,
        diameter: 24.0,
        length: 70.0,
        total_impulse: 16.70,
        burn_time: 1.30,
        avg_thrust: 12.8,
        max_thrust: 26.0,
        propellant_mass: 19.0,
        dry_mass: 17.0,
        delay_time: 6.0,
        thrust_curve: vec![
            ThrustPoint { time: 0.00, thrust: 0.0 },
            ThrustPoint { time: 0.03, thrust: 18.0 },
            ThrustPoint { time: 0.08, thrust: 26.0 },
            ThrustPoint { time: 0.20, thrust: 23.0 },
            ThrustPoint { time: 0.40, thrust: 18.0 },
            ThrustPoint { time: 0.60, thrust: 14.0 },
            ThrustPoint { time: 0.85, thrust: 10.0 },
            ThrustPoint { time: 1.05, thrust: 7.0 },
            ThrustPoint { time: 1.20, thrust: 4.0 },
            ThrustPoint { time: 1.30, thrust: 0.0 },
        ],
        data_source: "embedded".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ImpulseClass;

    #[test]
    fn test_embedded_motors_count() {
        let motors = embedded_motors();
        assert_eq!(motors.len(), 22);
    }

    #[test]
    fn test_all_motors_have_valid_data() {
        for motor in embedded_motors() {
            // Basic fields
            assert!(
                !motor.manufacturer.is_empty(),
                "Motor {} has empty manufacturer",
                motor.designation
            );
            assert!(
                !motor.manufacturer_abbrev.is_empty(),
                "Motor {} has empty manufacturer_abbrev",
                motor.designation
            );
            assert!(
                !motor.designation.is_empty(),
                "Motor has empty designation"
            );

            // Physical dimensions should be positive
            assert!(
                motor.diameter > 0.0,
                "Motor {} has non-positive diameter {}",
                motor.full_designation(),
                motor.diameter
            );
            assert!(
                motor.length > 0.0,
                "Motor {} has non-positive length {}",
                motor.full_designation(),
                motor.length
            );

            // Motor performance should be positive
            assert!(
                motor.total_impulse > 0.0,
                "Motor {} has non-positive total_impulse {}",
                motor.full_designation(),
                motor.total_impulse
            );
            assert!(
                motor.burn_time > 0.0,
                "Motor {} has non-positive burn_time {}",
                motor.full_designation(),
                motor.burn_time
            );
            assert!(
                motor.avg_thrust > 0.0,
                "Motor {} has non-positive avg_thrust {}",
                motor.full_designation(),
                motor.avg_thrust
            );
            assert!(
                motor.max_thrust > 0.0,
                "Motor {} has non-positive max_thrust {}",
                motor.full_designation(),
                motor.max_thrust
            );

            // Mass should be positive
            assert!(
                motor.propellant_mass > 0.0,
                "Motor {} has non-positive propellant_mass {}",
                motor.full_designation(),
                motor.propellant_mass
            );
            assert!(
                motor.dry_mass > 0.0,
                "Motor {} has non-positive dry_mass {}",
                motor.full_designation(),
                motor.dry_mass
            );

            // Delay time should be >= 0
            assert!(
                motor.delay_time >= 0.0,
                "Motor {} has negative delay_time {}",
                motor.full_designation(),
                motor.delay_time
            );

            // Data source should be "embedded"
            assert_eq!(
                motor.data_source, "embedded",
                "Motor {} has wrong data_source {}",
                motor.full_designation(),
                motor.data_source
            );

            // Motor type should be solid for these common motors
            assert_eq!(
                motor.motor_type,
                MotorType::Solid,
                "Motor {} has non-solid motor type",
                motor.full_designation()
            );

            // Impulse class should match
            let ic = ImpulseClass::from_total_impulse(motor.total_impulse);
            let (min_i, max_i) = ic.impulse_range();
            assert!(
                motor.total_impulse >= min_i && motor.total_impulse <= max_i,
                "Motor {} total_impulse {} outside range [{}, {}] for class {:?}",
                motor.full_designation(),
                motor.total_impulse,
                min_i,
                max_i,
                ic
            );

            // Thrust curve should have points
            assert!(
                !motor.thrust_curve.is_empty(),
                "Motor {} has empty thrust curve",
                motor.full_designation()
            );

            // Thrust curve should be valid
            for (i, pt) in motor.thrust_curve.iter().enumerate() {
                assert!(pt.time >= 0.0, "Negative time at index {} for motor {}", i, motor.full_designation());
                assert!(pt.thrust >= 0.0, "Negative thrust at index {} for motor {}", i, motor.full_designation());
                if i > 0 {
                    assert!(
                        pt.time > motor.thrust_curve[i - 1].time,
                        "Thrust curve not strictly increasing at index {} for motor {}",
                        i,
                        motor.full_designation()
                    );
                }
            }

            // Thrust curve should start and end at zero
            assert!(
                (motor.thrust_curve[0].thrust - 0.0).abs() < 1e-10,
                "Motor {} thrust curve doesn't start at 0",
                motor.full_designation()
            );
            assert!(
                (motor.thrust_curve.last().unwrap().thrust - 0.0).abs() < 1e-10,
                "Motor {} thrust curve doesn't end at 0",
                motor.full_designation()
            );
        }
    }

    #[test]
    fn test_embedded_motors_unique_designations() {
        let motors = embedded_motors();
        let mut seen = std::collections::HashSet::new();
        for motor in &motors {
            let key = format!("{}/{}", motor.manufacturer, motor.designation);
            assert!(
                seen.insert(key.clone()),
                "Duplicate motor: {}",
                key
            );
        }
    }

    #[test]
    fn test_motor_interpolator_valid() {
        for motor in embedded_motors() {
            let model = motor.thrust_curve_interpolator().unwrap();
            // Verify thrust at time 0
            assert!(
                (model.thrust_at(0.0) - motor.thrust_curve[0].thrust).abs() < 1e-10,
                "Motor {} interpolator fails at time 0",
                motor.full_designation()
            );
        }
    }
}