use crate::ids::{GroupId, ProfileId, TopicId};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "group_topic")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: TopicId,
    pub group_id: GroupId,
    pub profile_id: ProfileId,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::group::Entity",
        from = "Column::GroupId",
        to = "super::group::Column::Id"
    )]
    Group,
    #[sea_orm(has_many = "super::group_post::Entity")]
    GroupPost,
}

impl Related<super::group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Group.def()
    }
}

impl Related<super::group_post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GroupPost.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
