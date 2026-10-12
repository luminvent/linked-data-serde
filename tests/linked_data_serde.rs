use linked_data_serde::{Error, LinkedDataFormat, from_slice, to_vec, to_vec_list};

#[derive(
  Debug,
  PartialEq,
  serde::Serialize,
  serde::Deserialize,
  linked_data_sparql::Serialize,
  linked_data_sparql::Deserialize,
)]
#[ld(prefix("ex" = "http://ex/"))]
#[ld(type = "ex:Book")]
struct Book {
  #[ld("ex:title")]
  title: String,

  #[ld("ex:author")]
  author: Author,
}

#[derive(
  Debug,
  PartialEq,
  serde::Serialize,
  serde::Deserialize,
  linked_data_sparql::Serialize,
  linked_data_sparql::Deserialize,
)]
#[ld(prefix("ex" = "http://ex/"))]
#[ld(type = "ex:Author")]
struct Author {
  #[ld("ex:title")]
  title: String,
}

fn book() -> Book {
  Book {
    title: "Le Petit Prince".to_string(),
    author: Author {
      title: "Antoine de Saint-Exupéry".to_string(),
    },
  }
}

#[test]
fn test_round_trip() {
  for format in LinkedDataFormat::ALL {
    let bytes = to_vec(&book(), format).unwrap();
    let actual: Book = from_slice(&bytes, format).unwrap();

    assert_eq!(actual, book(), "{format:?}");
  }
}

#[test]
fn test_list() {
  let books = [book(), book()];

  let json: serde_json::Value =
    serde_json::from_slice(&to_vec_list(&books, LinkedDataFormat::Json).unwrap()).unwrap();
  assert_eq!(json.as_array().unwrap().len(), 2);

  let n_triples = to_vec_list(&books, LinkedDataFormat::NTriples).unwrap();
  let book_types = String::from_utf8(n_triples)
    .unwrap()
    .lines()
    .filter(|line| line.ends_with("<http://ex/Book> ."))
    .count();
  assert_eq!(book_types, 2);
}

#[test]
fn test_turtle_with_prefixes() {
  let turtle = r#"
    @prefix ex: <http://ex/> .
    <http://ex/book/1> a ex:Book ;
      ex:title "Le Petit Prince" ;
      ex:author [ a ex:Author ; ex:title "Antoine de Saint-Exupéry" ] .
  "#;

  let actual: Book = from_slice(turtle.as_bytes(), LinkedDataFormat::Turtle).unwrap();

  assert_eq!(actual, book());
}

#[test]
fn test_wrong_rdf_type() {
  let json_ld = r#"{
    "@context": { "ex": "http://ex/" },
    "@type": "ex:Magazine",
    "ex:title": "Le Petit Prince",
    "ex:author": { "@type": "ex:Author", "ex:title": "Antoine de Saint-Exupéry" }
  }"#;

  let result = from_slice::<Book>(json_ld.as_bytes(), LinkedDataFormat::JsonLd);

  assert!(matches!(result, Err(Error::NoMatchingObject)));
}

#[test]
fn test_nested_object_is_not_picked() {
  // The author matches `Author`, but the root node (the book) must be the one deserialized.
  let json_ld = r#"{
    "@context": { "ex": "http://ex/" },
    "@type": "ex:Book",
    "ex:title": "Le Petit Prince",
    "ex:author": { "@type": "ex:Author", "ex:title": "Antoine de Saint-Exupéry" }
  }"#;

  let actual: Author = from_slice(json_ld.as_bytes(), LinkedDataFormat::JsonLd).unwrap();

  assert_eq!(actual, book().author);
}

#[test]
fn test_invalid_documents() {
  assert!(matches!(
    from_slice::<Book>(b"{", LinkedDataFormat::Json),
    Err(Error::Json(_))
  ));
  assert!(matches!(
    from_slice::<Book>(b"<http://ex/a> <http://ex/b>", LinkedDataFormat::NTriples),
    Err(Error::RdfSyntax(_))
  ));
}

#[test]
fn test_media_types() {
  for format in LinkedDataFormat::ALL {
    assert_eq!(
      LinkedDataFormat::from_media_type(format.media_type()),
      Some(format)
    );
  }

  assert_eq!(
    LinkedDataFormat::from_media_type("application/ld+json"),
    Some(LinkedDataFormat::JsonLd)
  );
  assert_eq!(
    LinkedDataFormat::from_media_type("text/turtle; charset=utf-8"),
    Some(LinkedDataFormat::Turtle)
  );
  assert_eq!(LinkedDataFormat::from_media_type("text/plain"), None);
}
