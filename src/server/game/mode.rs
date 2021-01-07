use crate::imports::*;
use crate::server::symbols::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Gamemode {
    Survival,
    Creative,
    Adventure,
    Spectator
}

impl Into<u8> for Gamemode {
    fn into(self) -> u8 {
        match self {
            Gamemode::Survival => 0,
            Gamemode::Creative => 1,
            Gamemode::Adventure => 2,
            Gamemode::Spectator => 3
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Dimension {
    Nether,
    Overworld,
    End
}

impl From<Dimension> for i8 {
    fn from(dim: Dimension) -> i8 {
        match dim {
            Dimension::Nether => -1,
            Dimension::Overworld => 0,
            Dimension::End => 1
        }
    }
}