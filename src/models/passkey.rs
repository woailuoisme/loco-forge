pub use super::_entities::passkey::{ActiveModel, Column, Entity, Model};
use chrono::Utc;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

pub type Passkey = Entity;

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
    /// 递增防重放签名计数器
    pub async fn increment_counter<C>(&self, db: &C, new_counter: i64) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let counter_to_set = if new_counter > self.counter {
            new_counter
        } else {
            self.counter + 1
        };

        let mut active: ActiveModel = self.clone().into();
        active.counter = Set(counter_to_set);
        active.update(db).await
    }

    /// 认证成功后更新签名计数器与凭据状态（备份位等，由 WebAuthn 验证结果驱动）
    pub async fn apply_authentication_result<C>(
        &self,
        db: &C,
        counter: i64,
        refreshed_state: Option<String>,
    ) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut active: ActiveModel = self.clone().into();
        active.counter = Set(counter.max(self.counter));
        if let Some(json) = refreshed_state {
            active.public_key = Set(json);
        }
        active.update(db).await
    }
}

// ② ActiveModel 钩子与生命周期
impl ActiveModel {}

// ③ 集合查询与管理
impl Entity {
    /// 注册写入新的 Passkey 凭据记录
    #[expect(
        clippy::too_many_arguments,
        reason = "WebAuthn 凭据字段较多，仓储层逐字段传参以保持调用方零 DTO 转换"
    )]
    pub async fn create_passkey<C>(
        db: &C,
        user_id: &str,
        credential_id: &str,
        public_key: &str,
        counter: i64,
        name: Option<String>,
        device_type: Option<String>,
        backed_up: bool,
        transports: Option<String>,
        aaguid: Option<String>,
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        let now = Utc::now();
        let active = ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            name: Set(name),
            public_key: Set(public_key.to_string()),
            user_id: Set(user_id.to_string()),
            credential_id: Set(credential_id.to_string()),
            counter: Set(counter),
            device_type: Set(device_type.unwrap_or_else(|| "singleDevice".to_string())),
            backed_up: Set(backed_up),
            transports: Set(transports),
            aaguid: Set(aaguid),
            created_at: Set(now.into()),
        };

        active.insert(db).await
    }

    /// 根据 Credential ID 查找公钥凭据
    pub async fn find_by_credential_id<C>(
        db: &C,
        credential_id: &str,
    ) -> Result<Option<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::find()
            .filter(Column::CredentialId.eq(credential_id))
            .one(db)
            .await
    }

    /// 查询指定用户拥有的所有 Passkey 列表（按注册时间倒序）
    pub async fn find_by_user_id<C>(db: &C, user_id: &str) -> Result<Vec<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::CreatedAt)
            .all(db)
            .await
    }

    /// 用户安全删除指定 Passkey（验证拥有者 user_id）
    pub async fn delete_by_id_for_user<C>(db: &C, id: &str, user_id: &str) -> Result<bool, DbErr>
    where
        C: ConnectionTrait,
    {
        let res = Self::delete_many()
            .filter(Column::Id.eq(id))
            .filter(Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(res.rows_affected > 0)
    }

    /// 更新指定 Passkey 的名称/备注
    pub async fn update_name_for_user<C>(
        db: &C,
        id: &str,
        user_id: &str,
        name: &str,
    ) -> Result<bool, DbErr>
    where
        C: ConnectionTrait,
    {
        let opt_model = Self::find()
            .filter(Column::Id.eq(id))
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await?;

        let Some(m) = opt_model else {
            return Ok(false);
        };

        let mut active: ActiveModel = m.into();
        active.name = Set(Some(name.to_string()));
        active.update(db).await?;
        Ok(true)
    }
}
