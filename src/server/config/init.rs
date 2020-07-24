use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct ConfigInit {

}

impl Default for ConfigInit {
    fn default() -> Self {
        todo!()
    }
}