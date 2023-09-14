use std::{time::Instant, sync::Arc, process::Output};
use tokio::sync::Mutex;

use crate::error::{PidError, ErrorType};

use super::traits::{PidRunMode, PidControl};

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
    last_execution: Instant,
    
    last_error: Option<f64>, // error value of previous iteration
    total_error: Option<f64>,

    output_channel: OutputChannel,
    input_channel: InputChannel,
}

impl PidControl for PidControlStreaming {}

impl PidControlStreaming {
    fn start (&'static mut self) {
        let (input_tx, mut input_rx) = tokio::sync::mpsc::channel::<f64>(2);
        let (output_tx, output_rx) = tokio::sync::watch::channel::<Option<f64>>(None);

        self.input_channel = input_tx;
        self.output_channel = output_rx;
        
        // Sending/receiving through channels
        tokio::task::spawn(async move {
            // Receiving incoming input values
            let mut input_var = self.input_var;
            tokio::task::spawn(async move {
                while let Some(msg) = input_rx.recv().await {
                    input_var = msg
                }
            });

            
            // Transmitting output values
            loop {
                // Sleep until the next cycle
                std::thread::sleep(std::time::Duration::from_millis(Self::time_remaining(Instant::now(), self.last_execution).try_into().unwrap()));
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
                    input_var
                ) 
                {
                    Ok(a) => a,
                    Err(e) => {eprintln!("{:?}", e);break},
                };

                self.last_execution = Instant::now();
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
