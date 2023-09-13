use tokio::sync::{Mutex, watch::Sender, mpsc::Receiver, RwLock};

use crate::error::{PidError, ErrorType};
use super::sync::{PidRunMode, PidControlSync};

type InputStream = tokio::sync::mpsc::Sender<f64>;
type OutputStream = tokio::sync::watch::Receiver<Option<f64>>;

/// A wrapper around `PidControlSync` using tokio's `watch` & `mpsc` channels, allowing for real-time & concurrent value updates
pub struct PidControlStreaming {
    inner: PidControlSync,
    rwlock: RwLock<PidControlSync>,
    
    input_value_stream: InputStream,
    output_value_stream: OutputStream
}

impl PidControlStreaming {
    /// Creata a `tokio::task` that periodically runs the inner `calculate_next`
    pub async fn start(&self) -> Result<(), PidError> {
        // let mut inner = self.mutex.lock().await;

        // // Sending the output
        // tokio::task::spawn(async move {
        //     loop {
        //         inner.await_next_blocking().await;
        //         match PidControlSync::calculate_next(&mut inner) {
        //             Ok(output) => {

        //             },
        //             Err(_) => continue,
        //         };

        //     }
        // });




        Ok(())
    }

    async fn init_streams(&mut self) {
        let (input_tx, mut input_rx) = tokio::sync::mpsc::channel::<f64>(2);
        let (output_tx, output_rx) = tokio::sync::watch::channel::<Option<f64>>(None);

        self.input_value_stream = input_tx;
        self.output_value_stream = output_rx;


        // // Sending the output
        // tokio::task::spawn(async move {
        //     loop {
        //         inner.await_next_blocking().await;
        //         match PidControlSync::calculate_next(&mut inner) {
        //             Ok(output) => {
        //                 match output_tx.send(Some(output)) {
        //                     Ok(_) => (),
        //                     Err(_) => break
        //                 }
        //             },
        //             Err(_) => continue, // The only fail case is when the function is called before it's execution frequency
        //         };

        //     }
        // });


        // // Receiving input values
        // tokio::task::spawn(async move {
        //     while let Some(value) = input_rx.recv().await {
        //         inner.set_input_value(value);
        //     }
    
        // });

    }

    /////////////////////// GETTERS
    /// Clone a new `OutputStream` to send new input value's through
    fn output_stream(&self) -> OutputStream {
        self.output_value_stream.clone()
    }
    /// Clone a new `InputStream` to read the latest output value from
    fn input_stream(&self) -> InputStream {
        self.input_value_stream.clone()
    }


    // Re-exports from PidControlSync


}
