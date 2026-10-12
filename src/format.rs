use oxrdfio::{JsonLdProfileSet, RdfFormat};

/// Representation of an object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkedDataFormat {
  /// Plain JSON, using the `serde` implementation of the object.
  Json,
  /// JSON-LD (expanded), using the RDF description of the object.
  JsonLd,
  /// N-Triples, using the RDF description of the object.
  NTriples,
  /// Turtle, using the RDF description of the object.
  Turtle,
}

impl LinkedDataFormat {
  pub const ALL: [Self; 4] = [Self::Json, Self::JsonLd, Self::NTriples, Self::Turtle];

  /// Format of a media type, ignoring its parameters (e.g. `; charset=utf-8`).
  ///
  /// Both `application/json+ld` and the registered `application/ld+json` are JSON-LD.
  pub fn from_media_type(media_type: &str) -> Option<Self> {
    match media_type.split(';').next()?.trim() {
      "application/json" => Some(Self::Json),
      "application/json+ld" | "application/ld+json" => Some(Self::JsonLd),
      "application/n-triples" => Some(Self::NTriples),
      "text/turtle" => Some(Self::Turtle),
      _ => None,
    }
  }

  pub fn media_type(self) -> &'static str {
    match self {
      Self::Json => "application/json",
      Self::JsonLd => "application/json+ld",
      Self::NTriples => "application/n-triples",
      Self::Turtle => "text/turtle",
    }
  }

  /// RDF format of the representation, `None` for plain JSON.
  pub(crate) fn rdf_format(self) -> Option<RdfFormat> {
    match self {
      Self::Json => None,
      Self::JsonLd => Some(RdfFormat::JsonLd {
        profile: JsonLdProfileSet::empty(),
      }),
      Self::NTriples => Some(RdfFormat::NTriples),
      Self::Turtle => Some(RdfFormat::Turtle),
    }
  }
}
