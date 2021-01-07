use crate::imports::*;
use crate::server::symbols::*;
use hashbrown::HashSet;

pub struct PrefixIOAsync {
    dir: PathBuf,
    clients: HashMap<String, (Receiver<PrefixIORequest>, Sender<PrefixIOResult>)>
}

pub struct PrefixIOAdmin {
    clients: HashSet<String>
}

impl PrefixIOAdmin {
    pub fn new_c_user(&self, name: &str) -> Result<PrefixIOUser, Box<dyn Error>> {}
}

pub enum PrefixIOAdminRequest {
    NewUserHandle {

    }
}

pub enum PrefixIOAdminResponse {}

pub struct PrefixIOUsers {
    send: Sender<PrefixIORequest>,
    recv: Receiver<PrefixIOResult>
}

impl PrefixIOUsers {
    pub fn load_user(&self, username: &str) -> Result<InternalUserRecord, Box<dyn Error>> {}

    pub fn load_user_uuid(&self, uuid: &UserOnlineUuid) -> Result<InternalUserRecord, Box<dyn Error>> {}

    pub fn save_user_ev(&self, data: &InternalUserRecord)
}

pub trait PrefixIOHandle {

}