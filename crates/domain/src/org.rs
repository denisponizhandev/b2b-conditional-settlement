use uuid::{Uuid};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletRole {
    Treasury,
    Payee,
    Approver
}

#[derive(Debug, Clone)]
pub struct Organization {
    id: Uuid,
    name: String
}

impl Organization {
    pub fn new(id: Uuid, name: String) -> Self {
        Organization {
            id,
            name
        }
    }

    pub fn id(&self) -> Uuid { self.id }
    pub fn name(&self) -> &String { &self.name }
}