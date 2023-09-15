use std::time::Instant;

use crate::error::{PidError, ErrorType};
pub trait PidControl {
    fn calculate_next(
        runmode: PidRunMode, 
        kp: f64, 
        ki: Option<f64>, 
        kd: Option<f64>, 
        execution_frequency_ms: u128,
        total_error: Option<f64>,
        last_error: Option<f64>,
        max_output: Option<f64>, 
        min_output: Option<f64>, 
        setpoint: f64, 
        input_var: f64
    ) -> Result<(f64, f64, Option<f64>), PidError> 
    {
        let (error, total_error, delta_error) = Self::calculate_error_clamped(
            runmode,
            max_output, 
            min_output, 
            setpoint, 
            input_var, 
            total_error,
            last_error
        )?;

        let (ki,kd, total_error, delta_error) = Self::check_values(
            runmode, 
            ki, 
            kd, 
            total_error, 
            delta_error
        )?;
        
        // I can unwrap here because of Self::check_values
        let mut output_val: f64 = Self::calculate_proportional(kp, error);
        match runmode {
            PidRunMode::P => {},
            PidRunMode::PI => {
                output_val += Self::calculate_integral(ki.unwrap(), execution_frequency_ms, total_error);
            },
            PidRunMode::PD => {
                output_val += Self::calculate_derivative(kd.unwrap(), execution_frequency_ms, delta_error);
            },
            PidRunMode::PID => {
                output_val += Self::calculate_integral(ki.unwrap(), execution_frequency_ms, total_error);
                output_val += Self::calculate_derivative(kd.unwrap(), execution_frequency_ms, delta_error);
            },
        };


        Ok((output_val, error, total_error))
    }

    /// Returns `Err<PidError>` upon missing required values based on `PidRunMode`
    fn check_values(
        runmode: PidRunMode, 
        ki: Option<f64>, 
        kd: Option<f64>, 
        total_error: Option<f64>,
        delta_error: Option<f64>
    ) -> Result<(Option<f64>,Option<f64>,Option<f64>,Option<f64>), PidError>  
    {
        match runmode {
            PidRunMode::P => {return Ok((None,None,None,None))},
            PidRunMode::PI => {
                let ki = ki.ok_or_else(|| {
                    PidError {
                        error_type: ErrorType::MissingValue,
                        msg: "Missing ki Value".to_string(),
                    }
                })?;
                let total_error= total_error.ok_or_else(|| {
                    PidError {
                        error_type: ErrorType::MissingValue,
                        msg: "Missing total_error Value".to_string(),
                    }
                })?;

                return Ok((Some(ki),Some(total_error),None,None)) 
                
            },
            PidRunMode::PD => {
                let kd = kd.ok_or_else(|| {
                    PidError {
                        error_type: ErrorType::MissingValue,
                        msg: "Missing kd Value".to_string(),
                    }
                })?;
                let delta_error = delta_error.ok_or_else(|| {
                    PidError {
                        error_type: ErrorType::MissingValue,
                        msg: "Missing delta_error Value".to_string(),
                    }
                })?;

                return Ok((None,Some(kd),None,Some(delta_error)))
            },
            PidRunMode::PID => {
                let ki = ki.ok_or_else(|| {
                    PidError {
                        error_type: ErrorType::MissingValue,
                        msg: "Missing ki Value".to_string(),
                    }
                })?;
                let total_error= total_error.ok_or_else(|| {
                    PidError {
                        error_type: ErrorType::MissingValue,
                        msg: "Missing total_error Value".to_string(),
                    }
                })?;

                let kd = kd.ok_or_else(|| {
                    PidError {
                        error_type: ErrorType::MissingValue,
                        msg: "Missing kd Value".to_string(),
                    }
                })?;
                let delta_error = delta_error.ok_or_else(|| {
                    PidError {
                        error_type: ErrorType::MissingValue,
                        msg: "Missing delta_error Value".to_string(),
                    }
                })?;

                return Ok((Some(ki),Some(kd),Some(total_error),Some(delta_error)))

            },
        }
    }
    
    fn time_remaining(now: Instant, last_execution: Instant) -> u128 {
        now.duration_since(last_execution).as_millis()
    }

    fn calculate_error_clamped(
        runmode: PidRunMode,
        max_output: Option<f64>, 
        min_output: Option<f64>,
        setpoint: f64, 
        input_var: f64, 
        mut total_error: Option<f64>,
        last_error: Option<f64>,
    ) -> Result<(f64, Option<f64>, Option<f64>), PidError> 
    {
        // SP: 20, CV: 21 = -1
        let error = setpoint - input_var;
        total_error = match total_error {
            Some(total_e) => {
                Some(total_e + error)
            }
            None => None
        };

        let delta_error = match runmode {
            PidRunMode::P => None,
            PidRunMode::PI => None,
            PidRunMode::PD => {
                let last_error = last_error.ok_or_else(|| {
                    PidError {
                        error_type: ErrorType::MissingValue,
                        msg: "Missing last_error Value".to_string(),
                    }
                })?;
                Some(last_error - error)
            },
            PidRunMode::PID => {
                let last_error = last_error.ok_or_else(|| {
                    PidError {
                        error_type: ErrorType::MissingValue,
                        msg: "Missing last_error Value".to_string(),
                    }
                })?;
                Some(last_error - error)
            },
        };

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

        Ok((error, total_error, delta_error))
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

