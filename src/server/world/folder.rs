use crate::imports::*;
use crate::server::symbols::*;
use crate::init_flags::*;

pub struct WorldFolder {}
impl WorldFolder {
    // TODO actually implement
    pub fn load_or_default(vf: &ValidatedInitFlags) -> Result<HashMap<Uuid, World>, Vec<String>> {
        Ok(HashMap::new())
    }
}