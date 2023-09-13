use std::time::Instant;
use crate::error::{PidError, ErrorType};

pub struct PidControlSync {
    runmode: PidRunMode,
    kp: f64,
    ki: f64,
    kd: f64,
    input: f64,

    output: Option<f64>, // Output variable
    max_output: Option<f64>,
    min_output: Option<f64>,

    setpoint: f64, // Desired variable

    execution_frequency_ms: u128, // Amount of milliseconds between calculations
    last_execution: Instant,
    
    last_error: f64, // error value of previous iteration
    total_error: Option<f64>,
    delta_error: Option<f64>,
}

impl PidControlSync {
    /// Calculates the next output based on `Self`
    pub fn calculate_next(&mut self) -> Result<f64, PidError> {
        // Die if it is not exeuction time 
        if (self.last_execution.duration_since(Instant::now())).as_millis() < self.execution_frequency_ms { 
            return Err(PidError {
                error_type: ErrorType::CalledTooSoon,
                msg: String::from("Method calculate was called before minimum time has elapsed"),
            })  
        }

        let error = Self::calculate_error_clamped(self, self.setpoint, self.input);
        let mut output = self.kp * error; // P

        // Calculate based off of runmode
        match self.runmode {
            PidRunMode::P => {},
            PidRunMode::PI => {
                self.total_error = Some(error + self.total_error.unwrap_or(0.0));
                output += (self.ki * self.execution_frequency_ms as f64) * self.total_error.unwrap_or(0.0)  // I
            },
            PidRunMode::PD => {
                self.delta_error = Some(error - self.last_error);
                output += (self.kd / self.execution_frequency_ms as f64) * self.delta_error.unwrap_or(0.0);  // D
            },
            PidRunMode::PID => {
                self.total_error = Some(error + self.total_error.unwrap_or(0.0));   // I
                self.delta_error = Some(error - self.last_error);                           // D

                output += (self.ki * self.execution_frequency_ms as f64) * self.total_error.unwrap_or(0.0);  // I
                output += (self.kd / self.execution_frequency_ms as f64) * self.delta_error.unwrap_or(0.0);  // D

            },
            
        }

        self.last_error = error;
        self.last_execution = Instant::now();
        self.output = Some(output);
        
        Ok(output)
    }

    /// Sets `Self.total_error` if applicable and returns the error value based off of `setpoint` & `control_variable`
    fn calculate_error_clamped(&mut self, setpoint: f64, control_variable: f64) -> f64 {
        // SP: 20, CV: 21 = -1
        let error = setpoint - control_variable;

        match self.max_output {
            Some(max) => {
                if self.total_error < Some(max) {
                    self.total_error = Some(max)
                }
            },
            None => {},
        }
        match self.min_output {
            Some(min) => {
                if self.total_error > Some(min) {
                    self.total_error = Some(min)
                }
            },
            None => {},
        }
        error
    }
    
    /// Sleep the current thread until the next execution cycle
    pub async fn await_next_blocking(&self) {
        let difference = self.execution_frequency_ms - self.last_execution.duration_since(Instant::now()).as_millis();
        std::thread::sleep(std::time::Duration::from_millis(difference as u64))
    }
    pub fn get_next_execution_time(&self) -> u128 {
        self.execution_frequency_ms - self.last_execution.duration_since(Instant::now()).as_millis()
    }

    /////////////////////// SETTERS
    /// Set the input value to be used for the next calculation cycle
    pub fn set_input_value(&mut self, input: f64) {
        self.input = input;
    }
    pub fn set_kp(&mut self, kp: f64) {
        self.kp = kp;
    }
    pub fn set_ki(&mut self, ki: f64) {
        self.ki = ki;
    }
    pub fn set_kd(&mut self, kd: f64) {
        self.kd = kd;
    }
    pub fn set_runmode(&mut self, runmode: PidRunMode) {
        self.runmode = runmode;
    }

    /////////////////////// GETTERS
    /// Retrieve the last stored output value, returns None if no calculation had been made
    pub fn get_output_value(&self) -> Option<f64> {
        self.output
    }
}

pub enum PidRunMode {
    P,
    PI,
    PD,
    PID,
}

