use crate::ids::{GroupId, ProfileId};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "group_admin")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub group_id: GroupId,
    #[sea_orm(primary_key, auto_increment = false)]
    pub identity_id: ProfileId,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::group::Entity",
        from = "Column::GroupId",
        to = "super::group::Column::Id"
    )]
    Group,
}

impl Related<super::group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Group.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
