pub use super::_entities::device_code::{ActiveModel, Column, Entity, Model};
use crate::common::utils::code::CodeUtil;
use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    QueryFilter, Set,
};
use uuid::Uuid;

pub type DeviceCode = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = Set(Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

// ① 单实例读写
impl Model {
    /// 检查该设备授权请求是否已超时过期
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }

    /// 检查是否仍在等待用户网页端授权中
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.status == "pending" && !self.is_expired()
    }

    /// 检查用户是否已同意授权
    #[must_use]
    pub fn is_approved(&self) -> bool {
        self.status == "approved" && !self.is_expired()
    }

    /// 检查用户是否已明确拒绝授权
    #[must_use]
    pub fn is_denied(&self) -> bool {
        self.status == "denied"
    }
}

// ② ActiveModel 钩子
impl ActiveModel {}

// ③ 集合查询与流转管理
impl Entity {
    /// 格式化生成 8 位易读大写字符用户码 (如 "WDJB-MJHT")
    #[must_use]
    pub fn generate_user_code() -> String {
        let chars = CodeUtil::generate_alphanumeric(8).to_uppercase();
        format!("{}-{}", &chars[..4], &chars[4..])
    }

    /// 签发新的设备授权码与用户配对码
    pub async fn create_code<C>(
        db: &C,
        client_id: Option<String>,
        scope: Option<String>,
        expires_in_seconds: i64,
        polling_interval: i32,
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        let now = Utc::now();
        let expires_at = now + Duration::seconds(expires_in_seconds);
        let device_code = format!("dev_{}", Uuid::new_v4().simple());
        let user_code = Self::generate_user_code();

        let active = ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            device_code: Set(device_code),
            user_code: Set(user_code),
            user_id: Set(None),
            expires_at: Set(expires_at.into()),
            status: Set("pending".to_string()),
            last_polled_at: Set(None),
            polling_interval: Set(polling_interval),
            client_id: Set(client_id),
            scope: Set(scope),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };

        active.insert(db).await
    }

    /// 根据用户输入的 user_code 查找记录
    pub async fn find_by_user_code<C>(db: &C, user_code: &str) -> Result<Option<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::find()
            .filter(Column::UserCode.eq(user_code.trim().to_uppercase()))
            .one(db)
            .await
    }

    /// 根据设备轮询持有的 device_code 查找记录
    pub async fn find_by_device_code<C>(db: &C, device_code: &str) -> Result<Option<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::find()
            .filter(Column::DeviceCode.eq(device_code.trim()))
            .one(db)
            .await
    }

    /// 用户登录态下批准设备授权，绑定 user_id 并置状态为 approved
    pub async fn approve_for_user<C>(
        db: &C,
        user_code: &str,
        user_id: &str,
    ) -> Result<Option<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        let opt_model = Self::find_by_user_code(db, user_code).await?;
        let Some(m) = opt_model else {
            return Ok(None);
        };

        if m.is_expired() || m.status != "pending" {
            return Ok(None);
        }

        let mut active: ActiveModel = m.into();
        active.status = Set("approved".to_string());
        active.user_id = Set(Some(user_id.to_string()));
        active.updated_at = Set(Utc::now().into());
        let updated = active.update(db).await?;
        Ok(Some(updated))
    }

    /// 用户明确拒绝设备授权
    pub async fn deny_for_user<C>(db: &C, user_code: &str) -> Result<Option<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        let opt_model = Self::find_by_user_code(db, user_code).await?;
        let Some(m) = opt_model else {
            return Ok(None);
        };

        if m.is_expired() || m.status != "pending" {
            return Ok(None);
        }

        let mut active: ActiveModel = m.into();
        active.status = Set("denied".to_string());
        active.updated_at = Set(Utc::now().into());
        let updated = active.update(db).await?;
        Ok(Some(updated))
    }

    /// 设备完成换票后原子置为 consumed (已核销)
    pub async fn consume_code<C>(db: &C, device_code: &str) -> Result<Option<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        let opt_model = Self::find_by_device_code(db, device_code).await?;
        let Some(m) = opt_model else {
            return Ok(None);
        };

        let mut active: ActiveModel = m.into();
        active.status = Set("consumed".to_string());
        active.updated_at = Set(Utc::now().into());
        let updated = active.update(db).await?;
        Ok(Some(updated))
    }

    /// 记录设备轮询时间戳（供频控与防暴力轮询使用）
    pub async fn update_last_polled<C>(db: &C, id: &str) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        if let Some(m) = Self::find_by_id(id.to_string()).one(db).await? {
            let mut active: ActiveModel = m.into();
            active.last_polled_at = Set(Some(Utc::now().into()));
            active.update(db).await?;
        }
        Ok(())
    }
}
