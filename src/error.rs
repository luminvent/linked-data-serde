use std::fmt;

#[derive(Debug)]
pub enum Error {
  /// Plain JSON (de)serialization failed.
  Json(serde_json::Error),
  /// The RDF document is not valid.
  RdfSyntax(oxrdfio::RdfSyntaxError),
  /// Writing the RDF document failed.
  Io(std::io::Error),
  /// No node of the RDF document can be deserialized as the expected object.
  NoMatchingObject,
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Json(error) => write!(f, "{error}"),
      Self::RdfSyntax(error) => write!(f, "{error}"),
      Self::Io(error) => write!(f, "{error}"),
      Self::NoMatchingObject => write!(f, "Unable to find a matching object in the document"),
    }
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::Json(error) => Some(error),
      Self::RdfSyntax(error) => Some(error),
      Self::Io(error) => Some(error),
      Self::NoMatchingObject => None,
    }
  }
}

impl From<serde_json::Error> for Error {
  fn from(error: serde_json::Error) -> Self {
    Self::Json(error)
  }
}

impl From<oxrdfio::RdfSyntaxError> for Error {
  fn from(error: oxrdfio::RdfSyntaxError) -> Self {
    Self::RdfSyntax(error)
  }
}

impl From<std::io::Error> for Error {
  fn from(error: std::io::Error) -> Self {
    Self::Io(error)
  }
}
