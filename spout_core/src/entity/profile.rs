use crate::ids::ProfileId;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "profile")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: ProfileId,
    pub name: String,
    #[sea_orm(column_name = "desc")]
    pub desc: String,
    pub picture: Option<Vec<u8>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
