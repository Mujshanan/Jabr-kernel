//! Domain-neutral, revisioned variable state. Values are opaque to the Kernel.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(pub String);
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VariableId(pub String);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValueEnvelope { pub type_id: String, pub schema_version: u32, pub payload: Vec<u8> }
impl ValueEnvelope {
    pub fn new(type_id: impl Into<String>, schema_version: u32, payload: Vec<u8>) -> Result<Self, KernelError> {
        let type_id = type_id.into();
        if type_id.trim().is_empty() || schema_version == 0 { return Err(KernelError::InvalidEnvelope); }
        Ok(Self { type_id, schema_version, payload })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariableRecord { pub id: VariableId, pub scope: ScopeId, pub name: Option<String>, pub value: ValueEnvelope, pub changed_revision: u64 }
#[derive(Clone, Debug)]
pub struct Snapshot { revision: u64, scope: ScopeId, records: Arc<BTreeMap<VariableId, VariableRecord>> }
impl Snapshot {
    pub fn revision(&self) -> u64 { self.revision }
    pub fn scope(&self) -> &ScopeId { &self.scope }
    pub fn get(&self, id: &VariableId) -> Option<&VariableRecord> { self.records.get(id) }
    pub fn iter(&self) -> impl Iterator<Item=&VariableRecord> { self.records.values() }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeSet { pub revision: u64, pub changed: Vec<VariableId>, pub deleted: Vec<VariableId> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KernelError { RevisionConflict { expected: u64, actual: u64 }, DuplicateId(VariableId), MissingId(VariableId), ScopeMismatch, InvalidEnvelope, Poisoned }
#[derive(Default)]
struct State { revision: u64, records: BTreeMap<VariableId, VariableRecord> }
#[derive(Clone, Default)]
pub struct Kernel { state: Arc<RwLock<State>> }
impl Kernel {
    pub fn new() -> Self { Self::default() }
    pub fn revision(&self) -> Result<u64, KernelError> { Ok(self.state.read().map_err(|_| KernelError::Poisoned)?.revision) }
    pub fn snapshot(&self, scope: ScopeId) -> Result<Snapshot, KernelError> {
        let state = self.state.read().map_err(|_| KernelError::Poisoned)?;
        let records = state.records.iter().filter(|(_, r)| r.scope == scope).map(|(k,v)|(k.clone(),v.clone())).collect();
        Ok(Snapshot { revision: state.revision, scope, records: Arc::new(records) })
    }
    pub fn begin_transaction(&self, expected_revision: u64, scope: ScopeId) -> Transaction {
        Transaction { kernel: self.clone(), expected_revision, scope, operations: Vec::new() }
    }
}
enum Operation { Insert(VariableId, Option<String>, ValueEnvelope), Update(VariableId, ValueEnvelope), Rename(VariableId, Option<String>), Delete(VariableId) }
pub struct Transaction { kernel: Kernel, expected_revision: u64, scope: ScopeId, operations: Vec<Operation> }
impl Transaction {
    pub fn insert(&mut self, id: VariableId, name: Option<String>, value: ValueEnvelope) { self.operations.push(Operation::Insert(id,name,value)); }
    pub fn update(&mut self, id: VariableId, value: ValueEnvelope) { self.operations.push(Operation::Update(id,value)); }
    pub fn rename(&mut self, id: VariableId, name: Option<String>) { self.operations.push(Operation::Rename(id,name)); }
    pub fn delete(&mut self, id: VariableId) { self.operations.push(Operation::Delete(id)); }
    pub fn commit(self) -> Result<ChangeSet, KernelError> {
        let mut state = self.kernel.state.write().map_err(|_| KernelError::Poisoned)?;
        if state.revision != self.expected_revision { return Err(KernelError::RevisionConflict { expected:self.expected_revision, actual:state.revision }); }
        // Validate and apply to a private working copy. No partial mutation is observable.
        let mut next = state.records.clone();
        let mut changed = BTreeSet::new(); let mut deleted = BTreeSet::new();
        for op in self.operations {
            match op {
                Operation::Insert(id,name,value) => {
                    if value.type_id.trim().is_empty() || value.schema_version == 0 { return Err(KernelError::InvalidEnvelope); }
                    if next.contains_key(&id) { return Err(KernelError::DuplicateId(id)); }
                    next.insert(id.clone(), VariableRecord { id:id.clone(), scope:self.scope.clone(), name, value, changed_revision:state.revision+1 }); changed.insert(id);
                }
                Operation::Update(id,value) => {
                    if value.type_id.trim().is_empty() || value.schema_version == 0 { return Err(KernelError::InvalidEnvelope); }
                    let record=next.get_mut(&id).ok_or_else(||KernelError::MissingId(id.clone()))?;
                    if record.scope != self.scope { return Err(KernelError::ScopeMismatch); }
                    record.value=value; record.changed_revision=state.revision+1; changed.insert(id);
                }
                Operation::Rename(id,name) => { let record=next.get_mut(&id).ok_or_else(||KernelError::MissingId(id.clone()))?; if record.scope!=self.scope{return Err(KernelError::ScopeMismatch)} record.name=name; record.changed_revision=state.revision+1; changed.insert(id); }
                Operation::Delete(id) => { let record=next.get(&id).ok_or_else(||KernelError::MissingId(id.clone()))?; if record.scope!=self.scope{return Err(KernelError::ScopeMismatch)} next.remove(&id); changed.remove(&id); deleted.insert(id); }
            }
        }
        if changed.is_empty() && deleted.is_empty() { return Ok(ChangeSet { revision:state.revision, changed:vec![], deleted:vec![] }); }
        state.revision += 1; state.records=next;
        let committed_revision = state.revision;
        for id in &changed { if let Some(record)=state.records.get_mut(id) { record.changed_revision=committed_revision; } }
        Ok(ChangeSet { revision:state.revision, changed:changed.into_iter().collect(), deleted:deleted.into_iter().collect() })
    }
}

#[cfg(test)] mod tests {
 use super::*;
 fn val(n:&str)->ValueEnvelope{ValueEnvelope::new("test.scalar",1,n.as_bytes().to_vec()).unwrap()}
 fn scope()->ScopeId{ScopeId("worksheet".into())}
 #[test] fn snapshots_are_isolated(){let k=Kernel::new();let mut t=k.begin_transaction(0,scope());t.insert(VariableId("x".into()),Some("x".into()),val("1"));t.commit().unwrap();let old=k.snapshot(scope()).unwrap();let mut t=k.begin_transaction(1,scope());t.update(VariableId("x".into()),val("2"));t.commit().unwrap();assert_eq!(old.get(&VariableId("x".into())).unwrap().value.payload,b"1");assert_eq!(k.snapshot(scope()).unwrap().get(&VariableId("x".into())).unwrap().value.payload,b"2");}
 #[test] fn stale_writer_conflicts(){let k=Kernel::new();let mut a=k.begin_transaction(0,scope());let mut b=k.begin_transaction(0,scope());a.insert(VariableId("a".into()),None,val("a"));b.insert(VariableId("b".into()),None,val("b"));a.commit().unwrap();assert_eq!(b.commit().unwrap_err(),KernelError::RevisionConflict{expected:0,actual:1});assert!(k.snapshot(scope()).unwrap().get(&VariableId("b".into())).is_none());}
 #[test] fn failed_multi_op_is_atomic(){let k=Kernel::new();let mut t=k.begin_transaction(0,scope());t.insert(VariableId("x".into()),None,val("1"));t.commit().unwrap();let mut t=k.begin_transaction(1,scope());t.update(VariableId("x".into()),val("2"));t.update(VariableId("missing".into()),val("3"));assert_eq!(t.commit().unwrap_err(),KernelError::MissingId(VariableId("missing".into())));assert_eq!(k.revision().unwrap(),1);assert_eq!(k.snapshot(scope()).unwrap().get(&VariableId("x".into())).unwrap().value.payload,b"1");}
 #[test] fn no_op_does_not_increment_revision(){let k=Kernel::new();let c=k.begin_transaction(0,scope()).commit().unwrap();assert_eq!(c.revision,0);}
 #[test] fn scopes_are_isolated(){let k=Kernel::new();let mut t=k.begin_transaction(0,scope());t.insert(VariableId("x".into()),None,val("1"));t.commit().unwrap();assert!(k.snapshot(ScopeId("other".into())).unwrap().get(&VariableId("x".into())).is_none());}
}
