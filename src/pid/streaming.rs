use std::time::Instant;
use crate::error::PidError;

use super::traits::{PidRunMode, PidControl};
use crate::error::ErrorType;

type OutputChannel = tokio::sync::watch::Receiver<Option<f64>>;
type InputChannel = tokio::sync::mpsc::Sender<f64>;

pub struct PidControlStreaming {
    runmode: PidRunMode,
    kp: f64,
    ki: Option<f64>,
    kd: Option<f64>,
    input_var: f64,

    max_output: Option<f64>,
    min_output: Option<f64>,
    setpoint: f64, // Desired variable
    execution_frequency_ms: u128, // Amount of milliseconds between calculations

    last_execution: Option<Instant>,
    
    last_error: Option<f64>, // error value of previous iteration
    total_error: Option<f64>,

    pub output_channel: Option<OutputChannel>,
    pub input_channel: Option<InputChannel>,
}

impl PidControl for PidControlStreaming {}

impl PidControlStreaming {
    pub fn new(
        runmode:                    PidRunMode,
        kp:                         f64,
        ki:                         Option<f64>,
        kd:                         Option<f64>,
        input_var:                  f64,    
        max_output:                 Option<f64>,
        min_output:                 Option<f64>,
        setpoint:                   f64,
        execution_frequency_ms:     u128, 
    ) -> Result<Self, PidError> {

        // Validation
        match runmode {
            PidRunMode::P => {},
            PidRunMode::PI => {
                if ki.is_none() {return Err(PidError { error_type: ErrorType::MissingValue, msg: "Missing ki during construction".to_string() })}
            },
            PidRunMode::PD => {
                if kd.is_none() {return Err(PidError { error_type: ErrorType::MissingValue, msg: "Missing kd during construction".to_string() })}
            },
            PidRunMode::PID => {
                if ki.is_none() {return Err(PidError { error_type: ErrorType::MissingValue, msg: "Missing ki during construction".to_string() })}
                if kd.is_none() {return Err(PidError { error_type: ErrorType::MissingValue, msg: "Missing kd during construction".to_string() })}
            },
        };

        Ok(Self {
            runmode,
            kp,
            ki,
            kd,
            input_var,
            max_output,
            min_output,
            setpoint,
            execution_frequency_ms,
            last_execution:             None,
            last_error:                 None,
            total_error:                None,
            output_channel:             None,
            input_channel:              None,
        })

    }

    pub fn start (&'static mut self) {
        let (input_tx, mut input_rx) = tokio::sync::mpsc::channel::<f64>(2);
        let (output_tx, output_rx) = tokio::sync::watch::channel::<Option<f64>>(None);

        self.input_channel = Some(input_tx);
        self.output_channel = Some(output_rx);
        
        // Sending/receiving through channels
        tokio::task::spawn(async move {

            // Receiving incoming input values
            let mut _input_var = self.input_var;
            tokio::task::spawn(async move {
                while let Some(msg) = input_rx.recv().await {
                    _input_var = msg
                }
            });

            
            // Transmitting output values
            loop {
                // Sleep until the next cycle, if this is the first, compare now to now
                std::thread::sleep(std::time::Duration::from_millis(Self::time_remaining(
                    Instant::now(),
                    self.last_execution.unwrap_or(Instant::now())
                ).try_into().unwrap()));
                
                let (output, error, total_error) = match Self::calculate_next(
                    self.runmode, 
                    self.kp, 
                    self.ki, 
                    self.kd, 
                    self.execution_frequency_ms, 
                    self.total_error, 
                    self.last_error, 
                    self.max_output, 
                    self.min_output, 
                    self.setpoint, 
                    _input_var
                ) 
                {
                    Ok(a) => a,
                    Err(e) => {eprintln!("{:?}", e);break},
                };

                self.last_execution = Some(Instant::now());
                self.last_error = Some(error);
                self.total_error = total_error;

                match output_tx.send(Some(output)) {
                    Ok(_) => {
                        #[cfg(debug_assertions)]
                        println!("{}", output.clone())
                    }
                    Err(e) => {
                        eprintln!("{:?}", e);
                        break
                    }
                }
            };
        });

  

    }


}
