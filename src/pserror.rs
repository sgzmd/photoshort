pub mod error {
    use std::fmt::{Debug, Formatter, Result};
    use zip::result::ZipError;

    #[derive(Debug, Eq, PartialEq)]
    pub enum PsErrorKind {
        Unknown,
        NoExif,
        FileNotSupported,
        IoError,
        FormatError,
        NoDateField,
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct PsError {
        kind: PsErrorKind,
        msg: String,
    }

    impl std::fmt::Display for PsError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "PsError {:?}: {}", self.kind, self.msg)
        }
    }

    impl PsError {
        pub fn new(kind: PsErrorKind, msg: String) -> PsError {
            return PsError { kind, msg };
        }
    }

    impl From<std::io::Error> for PsError {
        fn from(err: std::io::Error) -> Self {
            PsError {
                kind: PsErrorKind::IoError,
                msg: err.to_string(),
            }
        }
    }

    impl From<ffmpeg::Error> for PsError {
        fn from(e: ffmpeg::Error) -> Self {
            return PsError::new(PsErrorKind::FormatError, e.to_string());
        }
    }

    impl From<ZipError> for PsError {
        fn from(e: ZipError) -> Self {
            return PsError::new(PsErrorKind::FormatError, e.to_string());
        }
    }

    #[cfg(test)]
    mod tests {

        use crate::pserror::error::{PsError, PsErrorKind};

        #[test]
        fn test_create_error() {
            let error = PsError::new(
                PsErrorKind::FileNotSupported,
                "File not supported".to_string(),
            );
            let expected = PsError {
                kind: PsErrorKind::FileNotSupported,
                msg: "File not supported".to_string(),
            };
            assert_eq!(error, expected);
        }
    }
}
