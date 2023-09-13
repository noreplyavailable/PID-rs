pub enum ErrorType {
    CalledTooSoon,
    MutexAlreadyLocked

}

pub struct PidError {
    pub error_type: ErrorType,
    pub msg: String
}