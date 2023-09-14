pub enum ErrorType {
    CalledTooSoon,
    MutexAlreadyLocked,
    MissingValue

}

pub struct PidError {
    pub error_type: ErrorType,
    pub msg: String
}