#[derive(Clone, Debug)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub is_host: bool,
}

impl Participant {
    pub fn new(id: String, name: String, is_host: bool) -> Self {
        Self { id, name, is_host }
    }
}
