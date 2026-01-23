use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Abstract {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub affiliation: Option<String>,
    pub center: Option<String>,
    pub contact_email: Option<String>,
    pub abstract_text: String,
    pub keywords: Vec<String>,
    pub take_home: Option<String>,
    pub reference: Option<String>,
    pub literature: Option<String>,
    pub locale: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemRef {
    pub id: String,
    pub order: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub order: u32,
    pub items: Vec<ItemRef>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub event: String,
    pub sessions: Vec<Session>,
}
