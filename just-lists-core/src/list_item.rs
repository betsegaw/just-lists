use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListItem {
    pub id: String,
    pub value: String,
    pub(crate) children: Vec<String>,
    #[serde(default = "State::default")]
    pub state: State,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum State {
    Pending,
    Completed,
    Blocked,
}

impl State {
    fn default() -> Self {
        State::Pending
    }
}

impl ListItem {
    pub fn new(value: String) -> ListItem {
        ListItem {
            id: Self::random_string_from_chars(32),
            value: value,
            children: Vec::<String>::new(),
            state: State::Pending,
        }
    }

    fn random_string_from_chars(length: usize) -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

        let mut rng = rand::rng();
        (0..length)
            .map(|_| CHARSET[rng.random_range(0..CHARSET.len())] as char)
            .collect()
    }
}
