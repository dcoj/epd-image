#[derive(Debug)]
pub enum Error {
    Image(image::ImageError),
    WrongDimensions(usize, usize),
    LodepngError(lodepng::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Image(e) => write!(f, "Image error: {}", e),
            Error::WrongDimensions(w, h) => write!(f, "Wrong dimensions: {}x{}", w, h),
            Error::LodepngError(e) => write!(f, "PNG error: {}", e),
            Error::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Image(e) => Some(e),
            Error::LodepngError(e) => Some(e),
            Error::Io(e) => Some(e),
            Error::WrongDimensions(_, _) => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<lodepng::Error> for Error {
    fn from(e: lodepng::Error) -> Self {
        Error::LodepngError(e)
    }
}

impl From<image::error::ImageError> for Error {
    fn from(e: image::error::ImageError) -> Self {
        Error::Image(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
