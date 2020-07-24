use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct ConfigAuth {

}

impl Default for ConfigAuth {
    fn default() -> Self {
        todo!()
    }
}