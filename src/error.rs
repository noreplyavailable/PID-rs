#[derive(Debug)]
pub enum ErrorType {
    CalledTooSoon,
    MutexAlreadyLocked,
    MissingValue

}

#[derive(Debug)]
pub struct PidError {
    pub error_type: ErrorType,
    pub msg: String
}