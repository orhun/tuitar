//! Error implementation.

/// Error type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Indicates that the note letter is missing.
    MissingLetter,
    /// Indicates that the note octave is missing.
    MissingOctave,
    /// Indicates that the note letter is invalid.
    InvalidLetter(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingLetter => write!(f, "note letter is missing"),
            Error::MissingOctave => write!(f, "note octave is missing"),
            Error::InvalidLetter(letter) => {
                write!(f, "invalid note letter: {letter}")
            }
        }
    }
}

impl std::error::Error for Error {}
