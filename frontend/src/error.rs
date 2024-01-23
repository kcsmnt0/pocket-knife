#[derive(Debug)]
pub enum ImageLoadError<E> {
    Filesystem(pocket_knife_file_format::OpenError<E>),
    Parse(tinybmp::ParseError),
}

impl <E> From<pocket_knife_file_format::OpenError<E>> for ImageLoadError<E> {
    fn from(error: pocket_knife_file_format::OpenError<E>) -> Self {
        ImageLoadError::Filesystem(error)
    }
}

impl <E> From<tinybmp::ParseError> for ImageLoadError<E> {
    fn from(error: tinybmp::ParseError) -> Self {
        ImageLoadError::Parse(error)
    }
}
