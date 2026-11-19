//! Source-independent UTF-8 byte spans.
/// Half-open UTF-8 byte range. Character-boundary validation belongs to the source-owning layer.
#[derive(Clone,Copy,Debug,Eq,Hash,PartialEq)]
#[cfg_attr(feature="serde",derive(serde::Serialize))]
pub struct Span{start:usize,end:usize}
impl Span{
 /// Constructs a span, rejecting reversed ranges.
 pub fn new(start:usize,end:usize)->Result<Self,SpanError>{if start>end{Err(SpanError::Reversed{start,end})}else{Ok(Self{start,end})}}
 /// Inclusive starting byte offset.
 pub const fn start(self)->usize{self.start}
 /// Exclusive ending byte offset.
 pub const fn end(self)->usize{self.end}
 /// Byte length.
 pub const fn len(self)->usize{self.end-self.start}
 /// Whether the range is empty.
 pub const fn is_empty(self)->bool{self.start==self.end}
}
#[cfg(feature="serde")] impl<'de> serde::Deserialize<'de> for Span{
 fn deserialize<D:serde::Deserializer<'de>>(d:D)->Result<Self,D::Error>{
  #[derive(serde::Deserialize)]struct Repr{start:usize,end:usize}
  let r=Repr::deserialize(d)?;Self::new(r.start,r.end).map_err(serde::de::Error::custom)
 }
}
/// Invalid span data.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum SpanError{/// End precedes start. Reversed{/// Start byte. start:usize,/// End byte. end:usize}}
impl core::fmt::Display for SpanError{fn fmt(&self,f:&mut core::fmt::Formatter<'_>)->core::fmt::Result{match self{Self::Reversed{start,end}=>write!(f,"span end {end} precedes start {start}")}}}
impl std::error::Error for SpanError{}
#[cfg(test)]mod tests{use super::*;#[test]fn half_open_and_empty(){let s=Span::new(2,5).unwrap();assert_eq!(s.len(),3);assert!(Span::new(2,2).unwrap().is_empty());assert!(Span::new(5,2).is_err());}}
