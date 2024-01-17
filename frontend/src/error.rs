#[derive(Debug, PartialEq, Eq)]
pub enum SlotReadError {
    InvalidSlot,
    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ImageReadError {
    SlotReadError(SlotReadError),
    ParseError(tinybmp::ParseError),
}
