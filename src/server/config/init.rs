use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ConfigInit {

}

impl Default for ConfigInit {
    fn default() -> Self {
        todo!()
    }
}