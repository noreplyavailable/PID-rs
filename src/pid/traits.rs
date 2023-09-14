use std::time::Instant;

use crate::error::{PidError, ErrorType};
pub trait PidControl {
    fn time_remaining(now: Instant, last_execution: Instant) -> u128 {
        now.duration_since(last_execution).as_millis()
    }

    fn calculate_error_clamped(
        max_output: Option<f64>, 
        min_output: Option<f64>,
        setpoint: f64, 
        input_var: f64, 
        mut total_error: Option<f64>
    ) -> (f64, Option<f64>) 
    {
        // SP: 20, CV: 21 = -1
        let error = setpoint - input_var;

        match max_output {
            Some(max) => {
                if total_error < Some(max) {
                    total_error = Some(max)
                }
            },
            None => {},
        }
        match min_output {
            Some(min) => {
                if total_error > Some(min) {
                    total_error = Some(min)
                }
            },
            None => {},
        }
        (error, total_error)
    }

    fn calculate_proportional(kp: f64, error: f64) -> f64 {
        kp * error
    }

    fn calculate_integral(ki: f64, execution_frequency_ms: u128, total_error: Option<f64>) -> f64 {
        (ki * execution_frequency_ms as f64) * total_error.unwrap_or(0.0) 
    }

    fn calculate_derivative(kd: f64, execution_frequency_ms: u128, delta_error: Option<f64>) -> f64 {
        (kd / execution_frequency_ms as f64) * delta_error.unwrap_or(0.0)
    }
}


#[derive(Clone, Copy)]
pub enum PidRunMode {
    P,
    PI,
    PD,
    PID,
}

