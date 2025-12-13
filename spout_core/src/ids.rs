use sea_orm::{
    sea_query::{ArrayType, Nullable, ValueType, ValueTypeErr},
    DbErr, QueryResult, TryFromU64, TryGetError, TryGetable, Value,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            pub fn into_uuid(self) -> Uuid {
                self.0
            }

            pub fn to_string(&self) -> String {
                self.0.to_string()
            }

            pub fn parse_str(s: &str) -> Result<Self, uuid::Error> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl AsRef<Uuid> for $name {
            fn as_ref(&self) -> &Uuid {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }

        impl TryFrom<String> for $name {
            type Error = uuid::Error;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                Ok(Self(Uuid::parse_str(&s)?))
            }
        }

        impl<'a> TryFrom<&'a str> for $name {
            type Error = uuid::Error;

            fn try_from(s: &'a str) -> Result<Self, Self::Error> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }

        // SeaORM trait implementations
        impl From<$name> for Value {
            fn from(id: $name) -> Self {
                Value::Uuid(Some(Box::new(id.0)))
            }
        }

        impl TryGetable for $name {
            fn try_get_by<I: sea_orm::ColIdx>(
                res: &QueryResult,
                idx: I,
            ) -> Result<Self, TryGetError> {
                let uuid: Uuid = res.try_get_by(idx).map_err(TryGetError::DbErr)?;
                Ok(Self(uuid))
            }
        }

        impl ValueType for $name {
            fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
                match v {
                    Value::Uuid(Some(uuid)) => Ok(Self(*uuid)),
                    _ => Err(ValueTypeErr),
                }
            }

            fn type_name() -> String {
                stringify!($name).to_owned()
            }

            fn array_type() -> ArrayType {
                ArrayType::Uuid
            }

            fn column_type() -> sea_orm::ColumnType {
                sea_orm::ColumnType::Uuid
            }
        }

        impl Nullable for $name {
            fn null() -> Value {
                Value::Uuid(None)
            }
        }

        impl TryFromU64 for $name {
            fn try_from_u64(_: u64) -> Result<Self, DbErr> {
                Err(DbErr::ConvertFromU64(stringify!($name)))
            }
        }
    };
}

// Define all our ID types
define_id!(ProfileId);
define_id!(GroupId);
define_id!(UserId);
define_id!(TopicId);
define_id!(PostId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_id_creation() {
        let id1 = ProfileId::new();
        let id2 = ProfileId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_id_conversion() {
        let uuid = Uuid::now_v7();
        let profile_id = ProfileId::from_uuid(uuid);
        assert_eq!(profile_id.as_uuid(), &uuid);
        assert_eq!(profile_id.into_uuid(), uuid);
    }

    #[test]
    fn test_id_string_conversion() {
        let id = GroupId::new();
        let s = id.to_string();
        let parsed = GroupId::parse_str(&s).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_id_serialization() {
        let id = TopicId::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: TopicId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }
}
