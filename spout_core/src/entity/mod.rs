// SeaORM entities
// This module contains SeaORM-based entity definitions
// that mirror the sqlx models in the `models` module

pub mod group;
pub mod group_admin;
pub mod group_banned;
pub mod group_post;
pub mod group_topic;
pub mod group_user;
pub mod identity;
pub mod profile;

#[cfg(test)]
mod tests;

pub mod prelude {
    // Re-export all entities for convenience
    pub use super::group::{
        ActiveModel as GroupActiveModel, Column as GroupColumn, Entity as Group,
        Model as GroupModel,
    };
    pub use super::group_admin::{
        ActiveModel as GroupAdminActiveModel, Column as GroupAdminColumn, Entity as GroupAdmin,
        Model as GroupAdminModel,
    };
    pub use super::group_banned::{
        ActiveModel as GroupBannedActiveModel, Column as GroupBannedColumn, Entity as GroupBanned,
        Model as GroupBannedModel,
    };
    pub use super::group_post::{
        ActiveModel as GroupPostActiveModel, Column as GroupPostColumn, Entity as GroupPost,
        Model as GroupPostModel,
    };
    pub use super::group_topic::{
        ActiveModel as GroupTopicActiveModel, Column as GroupTopicColumn, Entity as GroupTopic,
        Model as GroupTopicModel,
    };
    pub use super::group_user::{
        ActiveModel as GroupUserActiveModel, Column as GroupUserColumn, Entity as GroupUser,
        Model as GroupUserModel,
    };
    pub use super::identity::{
        ActiveModel as IdentityActiveModel, Column as IdentityColumn, Entity as Identity,
        Model as IdentityModel,
    };
    pub use super::profile::{
        ActiveModel as ProfileActiveModel, Column as ProfileColumn, Entity as Profile,
        Model as ProfileModel,
    };

    // Re-export commonly used SeaORM types and traits
    pub use sea_orm::{
        ActiveModelTrait,
        ActiveValue,

        ColumnTrait,
        ConnectionTrait,

        // Database and connection types
        Database,
        DatabaseConnection,
        DbConn,
        // Common result types
        DbErr,
        Delete,

        // Core traits
        EntityTrait,
        Insert,
        ItemsAndPagesNumber,
        Linked,

        ModelTrait,
        NotSet,
        // Pagination
        Paginator,
        PaginatorTrait,
        QueryFilter,
        QueryOrder,
        QuerySelect,
        Related,
        RelationTrait,
        // Query builders
        Select,
        // Active model helpers
        Set,
        TryInsertResult,

        Unchanged,
        Update,
    };
}
