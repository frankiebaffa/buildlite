use {
    std::{
        error::Error as StdError,
        fmt::Display as StdDisplay,
        fmt::Formatter as StdFormatter,
        fmt::Result as FmtResult,
    },
    worm::core::sql::Error as RusqliteError,
};
pub trait BuildliteErrorMatch<T, U>: Sized where U: StdError {
    fn quick_match(self) -> Result<T, BuildliteError>;
}
#[derive(Debug)]
pub enum BuildliteError {
    NoRowsError,
    SQLError(RusqliteError),
}
impl StdDisplay for BuildliteError {
    fn fmt(&self, f: &mut StdFormatter) -> FmtResult {
        match self {
            BuildliteError::NoRowsError => {
                write!(f, "No rows found!")
            },
            BuildliteError::SQLError(e) => {
                let msg = &format!("{}", e);
                f.write_str(msg)
            },
        }
    }
}
impl StdError for BuildliteError {}
impl<T> BuildliteErrorMatch<T, RusqliteError> for Result<T, RusqliteError> {
    fn quick_match(self) -> Result<T, BuildliteError> {
        return match self {
            Ok(s) => Ok(s),
            Err(e) => Err(BuildliteError::SQLError(e)),
        };
    }
}
