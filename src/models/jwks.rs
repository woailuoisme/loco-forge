pub use super::_entities::jwks::{ActiveModel, Entity, Model};
use chrono::Utc;
use loco_rs::model::ModelResult;
use sea_orm::{
    ActiveModelBehavior, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr,
    EntityTrait, QueryFilter, QueryOrder,
};
use uuid::Uuid;

pub type Jwks = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, _insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        Ok(self)
    }
}

// ① 单实例读写
impl Model {
    /// 转换为公开 JWK 结构体字典
    #[must_use]
    pub fn to_jwk_value(&self) -> serde_json::Value {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&self.public_key) {
            val
        } else {
            serde_json::json!({
                "kty": "oct",
                "use": "sig",
                "alg": self.alg.as_deref().unwrap_or("HS256"),
                "kid": self.id,
                "key": self.public_key,
            })
        }
    }
}

// ② 字段级钩子与 ActiveModel 行为
impl ActiveModel {}

// ③ 集合级查询与密钥管理
impl Entity {
    /// 获取所有未过期的公开密钥
    pub async fn find_active_keys(db: &DatabaseConnection) -> ModelResult<Vec<Model>> {
        use super::_entities::jwks::Column;
        use sea_orm::Condition;
        let now = Utc::now();
        let keys = Self::find()
            .filter(
                Condition::any()
                    .add(Column::ExpiresAt.is_null())
                    .add(Column::ExpiresAt.gt(now)),
            )
            .order_by_desc(Column::CreatedAt)
            .all(db)
            .await?;
        Ok(keys)
    }

    /// 获取主签名密钥，如果不存在则基于 fallback_secret 初始化默认 JWK
    pub async fn get_or_create_primary_key(
        db: &DatabaseConnection,
        fallback_secret: &str,
    ) -> ModelResult<Model> {
        use super::_entities::jwks::Column;
        let existing = Self::find()
            .order_by_desc(Column::CreatedAt)
            .one(db)
            .await?;

        if let Some(key) = existing {
            return Ok(key);
        }

        let kid = format!("key-{}", Uuid::now_v7());
        let public_jwk = serde_json::json!({
            "kty": "oct",
            "use": "sig",
            "alg": "HS256",
            "kid": kid,
        });

        let active_model = ActiveModel {
            id: ActiveValue::Set(kid),
            public_key: ActiveValue::Set(public_jwk.to_string()),
            private_key: ActiveValue::Set(fallback_secret.to_string()),
            alg: ActiveValue::Set(Some("HS256".to_string())),
            crv: ActiveValue::Set(None),
            expires_at: ActiveValue::Set(None),
            created_at: ActiveValue::Set(Utc::now().into()),
        };

        let inserted = Self::insert(active_model).exec_with_returning(db).await?;
        Ok(inserted)
    }
}
