#[derive(Debug)]
pub enum Error {
    Image(image::ImageError),
    WrongDimensions(usize, usize),
    Lodepng(lodepng::Error),
    Io(std::io::Error),
    Http(reqwest::Error),
    Json(serde_json::Error),
    NoThumbnailFound,
    InvalidApiResponse,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Image(e) => write!(f, "Image error: {e}"),
            Error::WrongDimensions(w, h) => write!(f, "Wrong dimensions: {w}x{h}"),
            Error::Lodepng(e) => write!(f, "PNG error: {e}"),
            Error::Io(e) => write!(f, "IO error: {e}"),
            Error::Http(e) => write!(f, "HTTP error: {e}"),
            Error::Json(e) => write!(f, "JSON error: {e}"),
            Error::NoThumbnailFound => write!(f, "No thumbnail found in API response"),
            Error::InvalidApiResponse => write!(f, "Invalid API response format"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Image(e) => Some(e),
            Error::Lodepng(e) => Some(e),
            Error::Io(e) => Some(e),
            Error::Http(e) => Some(e),
            Error::Json(e) => Some(e),
            Error::WrongDimensions(_, _) => None,
            Error::NoThumbnailFound => None,
            Error::InvalidApiResponse => None,
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
        Error::Lodepng(e)
    }
}

impl From<image::error::ImageError> for Error {
    fn from(e: image::error::ImageError) -> Self {
        Error::Image(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
