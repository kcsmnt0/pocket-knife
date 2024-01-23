use alloc::string::String;

#[derive(Debug, Clone)]
pub struct Error(pub String);

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}
