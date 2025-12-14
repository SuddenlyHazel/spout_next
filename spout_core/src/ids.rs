use sea_orm::{ColIdx, DbErr, QueryResult, TryFromU64, TryGetError, TryGetable, Value};
use sea_orm::sea_query::{ArrayType, ColumnType, Nullable, ValueType, ValueTypeErr};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

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

        // === Required SeaORM Trait Implementations ===

        /// Implement TryGetable to read from database
        impl TryGetable for $name {
            fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
                let uuid: Uuid = Uuid::try_get_by(res, idx)?;
                Ok($name(uuid))
            }
        }

        /// Implement conversion to Value for writing to database
        impl From<$name> for Value {
            fn from(id: $name) -> Self {
                Value::Uuid(Some(Box::new(id.0)))
            }
        }

        /// Implement Nullable for optional columns
        impl Nullable for $name {
            fn null() -> Value {
                Value::Uuid(None)
            }
        }

        /// Implement ValueType for sea-query type system
        impl ValueType for $name {
            fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
                match v {
                    Value::Uuid(Some(uuid)) => Ok($name(*uuid)),
                    _ => Err(ValueTypeErr),
                }
            }

            fn type_name() -> String {
                stringify!($name).to_owned()
            }

            fn array_type() -> ArrayType {
                ArrayType::Uuid
            }

            fn column_type() -> ColumnType {
                ColumnType::Uuid
            }
        }

        /// Implement TryFromU64 (UUID cannot be converted from u64)
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

    #[test]
    fn test_type_safety() {
        let profile_id = ProfileId::new();
        let group_id = GroupId::new();
        
        // This should compile - same type
        let _same: ProfileId = profile_id;
        
        // This would NOT compile - different types (uncomment to verify):
        // let _different: GroupId = profile_id;
        
        // Can only convert through Uuid
        let uuid: Uuid = profile_id.into();
        let _as_group: GroupId = GroupId::from_uuid(uuid);
    }

    #[test]
    fn test_value_type_conversion() {
        use sea_orm::sea_query::ValueType;
        
        let id = PostId::new();
        let value: Value = id.into();
        
        match value {
            Value::Uuid(Some(uuid)) => {
                let recovered = <PostId as ValueType>::try_from(Value::Uuid(Some(uuid))).unwrap();
                assert_eq!(recovered, id);
            }
            _ => panic!("Expected UUID value"),
        }
    }

    #[test]
    fn test_nullable() {
        use sea_orm::sea_query::Nullable;
        
        let null_value = ProfileId::null();
        assert!(matches!(null_value, Value::Uuid(None)));
    }

    #[test]
    fn test_try_from_u64_fails() {
        let result = UserId::try_from_u64(12345);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbErr::ConvertFromU64(_)));
    }
}
