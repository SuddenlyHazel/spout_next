use crate::ids::{PostId, TopicId, UserId};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "group_post")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: PostId,
    pub user_id: UserId,
    pub topic_id: TopicId,
    pub parent_post_id: Option<PostId>,  // NEW: NULL for top-level posts
    pub title: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::group_topic::Entity",
        from = "Column::TopicId",
        to = "super::group_topic::Column::Id"
    )]
    GroupTopic,
    #[sea_orm(
        belongs_to = "super::group_user::Entity",
        from = "Column::UserId",
        to = "super::group_user::Column::Id"
    )]
    GroupUser,
}

impl Related<super::group_topic::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GroupTopic.def()
    }
}

impl Related<super::group_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GroupUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
