#[derive(Debug, Clone)]
pub enum Subject {
    Mount { id: String, path: String },
}

impl Subject {
    pub fn id(&self) -> String {
        match self {
            Subject::Mount { id, .. } => id.clone(),
        }
    }
}
