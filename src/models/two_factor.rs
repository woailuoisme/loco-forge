pub use super::_entities::two_factor::{ActiveModel, Column, Entity, Model};
use crate::common::utils::code::CodeUtil;
use chrono::{Duration, Utc};
use sea_orm::{QueryFilter, Set, entity::prelude::*};
use totp_rs::{Algorithm, Builder as TotpBuilder, Secret};
use uuid::Uuid;

pub type TwoFactor = Entity;

/// 备份码哈希（SHA-256 hex，只存哈希不存明文，防库泄露重放）
fn hash_backup_code(code: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(code.trim().to_uppercase().as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// 备份码列表序列化为哈希 JSON
fn hash_backup_codes(codes: &[String]) -> Result<String, DbErr> {
    let hashed: Vec<String> = codes.iter().map(|c| hash_backup_code(c)).collect();
    serde_json::to_string(&hashed)
        .map_err(|e| DbErr::Custom(format!("Failed to serialize backup codes: {e}")))
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

// ① 单实例读写
impl Model {
    /// 检查该两步验证账号是否处于多次失败锁定状态
    #[must_use]
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            locked_until > Utc::now()
        } else {
            false
        }
    }

    /// 反序列化获取当前所有未被使用的备用恢复码列表
    #[must_use]
    pub fn get_backup_codes(&self) -> Vec<String> {
        serde_json::from_str::<Vec<String>>(&self.backup_codes).unwrap_or_default()
    }

    /// 校验用户提交的 6 位 TOTP 动态码 (使用 RFC 6238 标准窗口)
    #[must_use]
    pub fn verify_totp_code(&self, code: &str) -> bool {
        let Ok(secret) = Secret::try_from_base32(&self.secret) else {
            return false;
        };

        let Ok(totp) = TotpBuilder::new()
            .with_algorithm(Algorithm::SHA1)
            .with_digits(6)
            .with_skew(1)
            .with_step_duration(30)
            .with_secret(secret)
            .build()
        else {
            return false;
        };

        totp.check_current(code).is_some()
    }

    /// 校验并原子消耗单次备用恢复码（按哈希比对）
    pub async fn verify_and_consume_backup_code<C>(&self, db: &C, code: &str) -> Result<bool, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut codes = self.get_backup_codes();
        let target = hash_backup_code(code);

        if let Some(pos) = codes.iter().position(|c| *c == target) {
            codes.remove(pos);
            let updated_json = serde_json::to_string(&codes)
                .map_err(|e| DbErr::Custom(format!("Failed to serialize backup codes: {e}")))?;

            let mut active: ActiveModel = self.clone().into();
            active.backup_codes = Set(updated_json);
            active.failed_verification_count = Set(0);
            active.locked_until = Set(None);
            active.update(db).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 记录失败校验尝试次数（达到 5 次则锁定 15 分钟）
    pub async fn record_failed_attempt<C>(&self, db: &C) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        let mut active: ActiveModel = self.clone().into();
        let new_count = self.failed_verification_count + 1;
        active.failed_verification_count = Set(new_count);

        if new_count >= 5 {
            let lock_time = Utc::now() + Duration::minutes(15);
            active.locked_until = Set(Some(lock_time.into()));
        }

        active.update(db).await?;
        Ok(())
    }

    /// 重置失败计数与锁定状态
    pub async fn reset_failed_attempts<C>(&self, db: &C) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        if self.failed_verification_count > 0 || self.locked_until.is_some() {
            let mut active: ActiveModel = self.clone().into();
            active.failed_verification_count = Set(0);
            active.locked_until = Set(None);
            active.update(db).await?;
        }
        Ok(())
    }
}

// ② 钩子行为与生命周期
impl ActiveModel {}

// ③ 集合查询与聚合管理
impl Entity {
    /// 查找用户的 2FA 绑定记录
    pub async fn find_by_user_id<C>(db: &C, user_id: &str) -> Result<Option<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::find()
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await
    }

    /// 批量生成高熵随机备用恢复码 (默认 10 组，每组 10 位大写字母数字)
    #[must_use]
    pub fn generate_backup_codes(count: usize) -> Vec<String> {
        (0..count)
            .map(|_| CodeUtil::generate_alphanumeric(10).to_uppercase())
            .collect()
    }

    /// 创建或重置待验证的 2FA 凭据
    pub async fn upsert_pending<C>(
        db: &C,
        user_id: &str,
        secret: &str,
        backup_codes: &[String],
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        let codes_json = hash_backup_codes(backup_codes)?;

        let existing = Self::find_by_user_id(db, user_id).await?;
        let now = Utc::now();

        if let Some(m) = existing {
            let mut active: ActiveModel = m.into();
            active.secret = Set(secret.to_string());
            active.backup_codes = Set(codes_json);
            active.verified = Set(false);
            active.failed_verification_count = Set(0);
            active.locked_until = Set(None);
            active.updated_at = Set(now.into());
            active.update(db).await
        } else {
            let active = ActiveModel {
                id: Set(Uuid::new_v4().to_string()),
                user_id: Set(user_id.to_string()),
                secret: Set(secret.to_string()),
                backup_codes: Set(codes_json),
                verified: Set(false),
                failed_verification_count: Set(0),
                locked_until: Set(None),
                created_at: Set(now.into()),
                updated_at: Set(now.into()),
            };
            active.insert(db).await
        }
    }

    /// 确认并激活 2FA 绑定
    pub async fn confirm_enabled<C>(db: &C, user_id: &str) -> Result<Option<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        let existing = Self::find_by_user_id(db, user_id).await?;
        if let Some(m) = existing {
            let mut active: ActiveModel = m.into();
            active.verified = Set(true);
            active.failed_verification_count = Set(0);
            active.locked_until = Set(None);
            let updated = active.update(db).await?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    /// 解绑并彻底清除指定用户的 2FA 记录
    pub async fn disable_for_user<C>(db: &C, user_id: &str) -> Result<u64, DbErr>
    where
        C: ConnectionTrait,
    {
        let res = Self::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(res.rows_affected)
    }

    /// 重新生成并覆盖用户的备用码列表
    pub async fn update_backup_codes<C>(
        db: &C,
        user_id: &str,
        new_codes: &[String],
    ) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        let existing = Self::find_by_user_id(db, user_id).await?;
        if let Some(m) = existing {
            let codes_json = hash_backup_codes(new_codes)?;
            let mut active: ActiveModel = m.into();
            active.backup_codes = Set(codes_json);
            active.update(db).await?;
        }
        Ok(())
    }
}
