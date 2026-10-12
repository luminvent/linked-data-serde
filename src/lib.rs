//! Serialize and deserialize linked data objects in their semantic representations: plain JSON
//! (with `serde`), or RDF as JSON-LD, N-Triples or Turtle (with `linked-data-sparql`).

mod error;
mod format;

pub use error::Error;
pub use format::LinkedDataFormat;

use linked_data_sparql::reexport::oxrdf::vocab::rdf;
use linked_data_sparql::reexport::oxrdf::{Dataset, GraphNameRef, NamedOrBlankNode, QuadRef, Term};
use linked_data_sparql::{FromRdfSubject, ToRdfTerm};
use oxrdfio::{RdfParser, RdfSerializer};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashSet;

/// Serialize an object in the given format.
pub fn to_vec<T: Serialize + ToRdfTerm>(
  object: &T,
  format: LinkedDataFormat,
) -> Result<Vec<u8>, Error> {
  match format.rdf_format() {
    None => Ok(serde_json::to_vec(object)?),
    Some(rdf_format) => to_rdf(std::slice::from_ref(object), rdf_format),
  }
}

/// Serialize a list of objects in the given format: a JSON array, or the RDF description of all
/// the objects.
pub fn to_vec_list<T: Serialize + ToRdfTerm>(
  objects: &[T],
  format: LinkedDataFormat,
) -> Result<Vec<u8>, Error> {
  match format.rdf_format() {
    None => Ok(serde_json::to_vec(objects)?),
    Some(rdf_format) => to_rdf(objects, rdf_format),
  }
}

/// Deserialize an object from the given format.
///
/// For RDF formats, the object is the first node of the document which can be deserialized as `T`
/// and has the `rdf:type`(s) of `T`, nodes which are not referenced by other nodes being tried
/// first.
pub fn from_slice<T: DeserializeOwned + FromRdfSubject + ToRdfTerm>(
  bytes: &[u8],
  format: LinkedDataFormat,
) -> Result<T, Error> {
  match format.rdf_format() {
    None => Ok(serde_json::from_slice(bytes)?),
    Some(rdf_format) => from_rdf(bytes, rdf_format),
  }
}

fn to_rdf<T: ToRdfTerm>(objects: &[T], rdf_format: oxrdfio::RdfFormat) -> Result<Vec<u8>, Error> {
  let mut quads = vec![];

  for object in objects {
    object.to_term(&mut quads);
  }

  let mut serializer = RdfSerializer::from_format(rdf_format).for_writer(Vec::new());

  for quad in &quads {
    serializer.serialize_quad(quad)?;
  }

  Ok(serializer.finish()?)
}

fn from_rdf<T: FromRdfSubject + ToRdfTerm>(
  bytes: &[u8],
  rdf_format: oxrdfio::RdfFormat,
) -> Result<T, Error> {
  let mut dataset = Dataset::new();

  for quad in RdfParser::from_format(rdf_format).for_slice(bytes) {
    dataset.insert(&quad?);
  }

  let subjects: HashSet<NamedOrBlankNode> = dataset
    .iter()
    .map(|quad| quad.subject.into_owned())
    .collect();

  let referenced: HashSet<Term> = dataset
    .iter()
    .map(|quad| quad.object.into_owned())
    .collect();

  // Try the root nodes first (not referenced by any other node), so a nested object is not
  // picked instead of the main one when both match the expected shape.
  let (roots, nested): (Vec<_>, Vec<_>) = subjects
    .into_iter()
    .partition(|subject| !referenced.contains(&Term::from(subject.clone())));

  roots
    .iter()
    .chain(nested.iter())
    .find_map(|subject| {
      T::deserialize_subject(&dataset, subject)
        .ok()
        .filter(|object| has_rdf_types_of(object, &dataset, subject))
    })
    .ok_or(Error::NoMatchingObject)
}

/// Deserialization does not check `rdf:type`, so compare the types the object is serialized
/// with against the types declared on the source subject.
fn has_rdf_types_of<T: ToRdfTerm>(
  object: &T,
  dataset: &Dataset,
  subject: &NamedOrBlankNode,
) -> bool {
  let mut quads = vec![];
  let object_subject = object.to_term(&mut quads);

  quads
    .iter()
    .filter(|quad| {
      Term::from(quad.subject.clone()) == object_subject && quad.predicate == rdf::TYPE
    })
    .all(|quad| {
      dataset.contains(QuadRef::new(
        subject,
        rdf::TYPE,
        &quad.object,
        GraphNameRef::DefaultGraph,
      ))
    })
}
