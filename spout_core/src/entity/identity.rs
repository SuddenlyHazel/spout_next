use crate::ids::ProfileId;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Note: iroh::PublicKey is represented as Vec<u8> in the database
// The binary column stores the 32-byte public key
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "identity")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub node_id: Vec<u8>,
    #[sea_orm(primary_key, auto_increment = false)]
    pub profile_id: ProfileId,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::profile::Entity",
        from = "Column::ProfileId",
        to = "super::profile::Column::Id"
    )]
    Profile,
}

impl Related<super::profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Profile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
