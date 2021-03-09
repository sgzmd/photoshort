pub mod error {
    use std::fmt::{Debug, Formatter, Result};

    #[derive(Debug, Eq, PartialEq)]
    pub enum PsErrorKind {
        UNKNOWN,
        NO_EXIF,
        FILE_NOT_SUPPORTED,
        IO_ERROR,
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
        pub fn new() -> PsError {
            return PsError {
                kind: PsErrorKind::UNKNOWN,
                msg: "".to_string(),
            };
        }

        pub fn with_error_kind(&mut self, kind: PsErrorKind) -> &mut PsError {
            self.kind = kind;
            return self;
        }

        pub fn with_message(&mut self, msg: String) -> &mut PsError {
            self.msg = msg;
            return self;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::pserror::error::{PsError, PsErrorKind};
        use std::ops::DerefMut;

        #[test]
        fn test_create_error() {
            let mut error = PsError::new();
            error
                .with_error_kind(PsErrorKind::FILE_NOT_SUPPORTED)
                .with_message("File not supported".to_string());
            let expected = PsError {
                kind: PsErrorKind::FILE_NOT_SUPPORTED,
                msg: "File not supported".to_string(),
            };
            assert_eq!(error, expected);
        }
    }
}
