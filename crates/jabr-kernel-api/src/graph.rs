//! Generic dependency graph; it does not interpret formulas or schedule execution.
use core::fmt;
use std::collections::{BTreeMap,BTreeSet};
/// Directed graph in which each node records its prerequisites.
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct DependencyGraph<N:Ord>{dependencies:BTreeMap<N,BTreeSet<N>>}
impl<N:Ord>Default for DependencyGraph<N>{fn default()->Self{Self{dependencies:BTreeMap::new()}}}
impl<N:Ord+Clone>DependencyGraph<N>{
 /// Inserts a node if absent.
 pub fn add_node(&mut self,node:N){self.dependencies.entry(node).or_default();}
 /// Adds dependency as a prerequisite of node.
 pub fn add_dependency(&mut self,node:N,dependency:N)->Result<(),GraphError<N>>{
  if !self.dependencies.contains_key(&node){return Err(GraphError::UnknownNode(node))}
  if !self.dependencies.contains_key(&dependency){return Err(GraphError::UnknownNode(dependency))}
  self.dependencies.get_mut(&node).expect("validated").insert(dependency);Ok(())
 }
 /// Removes a node and all references to it.
 pub fn remove_node(&mut self,node:&N)->bool{let removed=self.dependencies.remove(node).is_some();if removed{for deps in self.dependencies.values_mut(){deps.remove(node);}}removed}
 /// Deterministic prerequisite-first order. Cycle errors include every unresolved node.
 pub fn topological_order(&self)->Result<Vec<N>,GraphError<N>>{
  let mut remaining:BTreeMap<N,usize>=self.dependencies.iter().map(|(n,d)|(n.clone(),d.len())).collect();
  let mut dependents:BTreeMap<N,BTreeSet<N>>=BTreeMap::new();
  for(node,deps)in &self.dependencies{for dep in deps{dependents.entry(dep.clone()).or_default().insert(node.clone());}}
  let mut ready:BTreeSet<N>=remaining.iter().filter_map(|(n,c)|(*c==0).then_some(n.clone())).collect();let mut order=Vec::with_capacity(remaining.len());
  while let Some(node)=ready.pop_first(){order.push(node.clone());if let Some(children)=dependents.get(&node){for child in children{let count=remaining.get_mut(child).expect("indexed graph node");*count-=1;if *count==0{ready.insert(child.clone());}}}}
  if order.len()==remaining.len(){Ok(order)}else{Err(GraphError::Cycle(remaining.into_iter().filter_map(|(n,c)|(c>0).then_some(n)).collect()))}
 }
 /// Number of nodes.
 pub fn len(&self)->usize{self.dependencies.len()}
 /// Whether the graph is empty.
 pub fn is_empty(&self)->bool{self.dependencies.is_empty()}
}
/// Dependency graph error.
#[derive(Clone,Debug,Eq,PartialEq)]
pub enum GraphError<N>{/// Referenced node does not exist. UnknownNode(N),/// Nodes left unresolved by topological ordering. Cycle(Vec<N>)}
impl<N:fmt::Debug>fmt::Display for GraphError<N>{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{match self{Self::UnknownNode(n)=>write!(f,"unknown graph node: {n:?}"),Self::Cycle(n)=>write!(f,"topological order blocked by a dependency cycle; unresolved nodes: {n:?}")}}}
impl<N:fmt::Debug>std::error::Error for GraphError<N>{}
#[cfg(test)]mod tests{use super::*;#[test]fn deterministic_order(){let mut g=DependencyGraph::default();for n in 1..=3{g.add_node(n)}g.add_dependency(3,1).unwrap();g.add_dependency(3,2).unwrap();assert_eq!(g.topological_order().unwrap(),vec![1,2,3]);}}
