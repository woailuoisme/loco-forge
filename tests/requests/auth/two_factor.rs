use axum::http::StatusCode;
use loco_rs::testing::prelude::*;
use locos::{
    app::App,
    dtos::auth::*,
    models::{two_factor, user},
};
use serial_test::serial;
use totp_rs::{Algorithm, Builder as TotpBuilder, Secret};
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_two_factor_enable_and_totp_challenge() {
    request::<App, _, _>(|request, ctx| async move {
        let email = format!("2fa_{}@example.com", Uuid::new_v4().simple());
        let password = "two_factor_master_pass_123";

        // 1. 注册新用户
        let signup_res = request
            .post("/v1/users")
            .add_header("x-forwarded-for", "10.20.1.1")
            .json(&serde_json::json!({
                "email": email,
                "password": password,
                "name": "2FA User"
            }))
            .await;
        assert_eq!(signup_res.status_code(), StatusCode::CREATED);
        let auth: AuthSessionResource = serde_json::from_str(&signup_res.text()).unwrap();
        let user_id = auth.user.id.clone();
        let token = auth.token.clone();

        // 2. 启用两步验证 (错误密码应返回 401)
        let enable_bad = request
            .post("/v1/users/me/two-factor")
            .add_header("x-forwarded-for", "10.20.1.2")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
            .json(&serde_json::json!({
                "password": "wrong_password"
            }))
            .await;
        assert_eq!(enable_bad.status_code(), StatusCode::UNAUTHORIZED);

        // 正确密码启用 2FA
        let enable_res = request
            .post("/v1/users/me/two-factor")
            .add_header("x-forwarded-for", "10.20.1.3")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
            .json(&serde_json::json!({
                "password": password
            }))
            .await;
        assert_eq!(enable_res.status_code(), StatusCode::OK);
        let enable_data: TwoFactorEnableResource =
            serde_json::from_str(&enable_res.text()).unwrap();
        assert!(enable_data.totp_uri.starts_with("otpauth://totp/"));
        assert_eq!(enable_data.backup_codes.len(), 10);

        // 3. 从两步验证表提取 Secret 并生成有效 TOTP
        let tf_record = two_factor::Entity::find_by_user_id(&ctx.db, &user_id)
            .await
            .unwrap()
            .expect("2FA pending record must exist");
        let secret = Secret::try_from_base32(&tf_record.secret).unwrap();
        let totp = TotpBuilder::new()
            .with_algorithm(Algorithm::SHA1)
            .with_digits(6)
            .with_skew(1)
            .with_step_duration(30)
            .with_secret(secret.clone())
            .with_account_name(email.as_str())
            .build()
            .unwrap();
        let valid_code = totp.generate_current().to_string();

        // 4. 首次确认绑定 TOTP
        let confirm_res = request
            .post("/v1/totp-checks")
            .add_header("x-forwarded-for", "10.20.1.4")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
            .json(&serde_json::json!({
                "code": valid_code
            }))
            .await;
        assert_eq!(confirm_res.status_code(), StatusCode::OK);

        // 检查数据库用户状态已开启 2FA
        let u = user::Entity::find_by_id(&ctx.db, &user_id)
            .await
            .unwrap()
            .unwrap();
        assert!(u.two_factor_enabled);

        // 5. 再次通过邮箱密码登录 -> 触发 2FA Challenge 拦截
        let login_2fa = request
            .post("/v1/sessions/email")
            .add_header("x-forwarded-for", "10.20.1.5")
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .await;
        assert_eq!(login_2fa.status_code(), StatusCode::OK);
        let login_json: serde_json::Value = serde_json::from_str(&login_2fa.text()).unwrap();
        assert_eq!(login_json["twoFactorRedirect"], true);

        let set_cookie = login_2fa
            .headers()
            .get("set-cookie")
            .and_then(|v| v.to_str().ok())
            .unwrap();
        assert!(set_cookie.contains("better-auth.challenge="));

        // 提取 challenge token
        let challenge_token = set_cookie
            .split(';')
            .find_map(|p| p.trim().strip_prefix("better-auth.challenge="))
            .unwrap();

        // 6. 无凭据调用 verify-totp -> 401
        let no_auth_res = request
            .post("/v1/totp-checks")
            .add_header("x-forwarded-for", "10.20.1.6")
            .json(&serde_json::json!({
                "code": "123456"
            }))
            .await;
        assert_eq!(no_auth_res.status_code(), StatusCode::UNAUTHORIZED);

        // 7. 使用 Challenge Cookie + 有有效 TOTP 完成登录挑战
        let totp_challenge_code = totp.generate_current().to_string();
        let challenge_verify = request
            .post("/v1/totp-checks")
            .add_header("x-forwarded-for", "10.20.1.7")
            .add_header(
                axum::http::header::COOKIE,
                format!("better-auth.challenge={challenge_token}"),
            )
            .json(&serde_json::json!({
                "code": totp_challenge_code
            }))
            .await;
        assert_eq!(challenge_verify.status_code(), StatusCode::OK);
        let sign_res: SignInResponse = serde_json::from_str(&challenge_verify.text()).unwrap();
        let logged_in_user = sign_res
            .user
            .expect("Expected SignInResponse to carry user");
        let logged_in_token = sign_res
            .token
            .expect("Expected SignInResponse to carry token");
        assert_eq!(logged_in_user.email, email);
        assert!(!logged_in_token.is_empty());
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_two_factor_backup_code_and_disable() {
    request::<App, _, _>(|request, ctx| async move {
        let email = format!("2fa_bk_{}@example.com", Uuid::new_v4().simple());
        let password = "two_factor_backup_pass_123";

        // 1. 注册并启用 2FA
        let signup_res = request
            .post("/v1/users")
            .add_header("x-forwarded-for", "10.20.2.1")
            .json(&serde_json::json!({
                "email": email,
                "password": password,
                "name": "2FA Backup User"
            }))
            .await;
        assert_eq!(signup_res.status_code(), StatusCode::CREATED);
        let auth: AuthSessionResource = serde_json::from_str(&signup_res.text()).unwrap();
        let user_id = auth.user.id.clone();
        let token = auth.token.clone();

        let enable_res = request
            .post("/v1/users/me/two-factor")
            .add_header("x-forwarded-for", "10.20.2.2")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
            .json(&serde_json::json!({
                "password": password
            }))
            .await;
        assert_eq!(enable_res.status_code(), StatusCode::OK);
        let enable_data: TwoFactorEnableResource =
            serde_json::from_str(&enable_res.text()).unwrap();
        let backup_code = enable_data.backup_codes[0].clone();

        // 绑定 TOTP 激活
        let tf_record = two_factor::Entity::find_by_user_id(&ctx.db, &user_id)
            .await
            .unwrap()
            .expect("2FA pending record must exist");
        let secret = Secret::try_from_base32(&tf_record.secret).unwrap();
        let totp = TotpBuilder::new()
            .with_algorithm(Algorithm::SHA1)
            .with_digits(6)
            .with_skew(1)
            .with_step_duration(30)
            .with_secret(secret.clone())
            .with_account_name(email.as_str())
            .build()
            .unwrap();
        let valid_code = totp.generate_current().to_string();

        let confirm_res = request
            .post("/v1/totp-checks")
            .add_header("x-forwarded-for", "10.20.2.3")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
            .json(&serde_json::json!({
                "code": valid_code
            }))
            .await;
        assert_eq!(confirm_res.status_code(), StatusCode::OK);

        // 2. 登录获取 Challenge
        let login_2fa = request
            .post("/v1/sessions/email")
            .add_header("x-forwarded-for", "10.20.2.4")
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .await;
        assert_eq!(login_2fa.status_code(), StatusCode::OK);
        let set_cookie = login_2fa
            .headers()
            .get("set-cookie")
            .and_then(|v| v.to_str().ok())
            .unwrap();
        let challenge_token = set_cookie
            .split(';')
            .find_map(|p| p.trim().strip_prefix("better-auth.challenge="))
            .unwrap();

        // 3. 校验备用码登录
        let backup_verify = request
            .post("/v1/backup-code-checks")
            .add_header("x-forwarded-for", "10.20.2.5")
            .add_header(
                axum::http::header::COOKIE,
                format!("better-auth.challenge={challenge_token}"),
            )
            .json(&serde_json::json!({
                "code": backup_code
            }))
            .await;
        assert_eq!(backup_verify.status_code(), StatusCode::OK);
        let sign_res: SignInResponse = serde_json::from_str(&backup_verify.text()).unwrap();
        let logged_in_token = sign_res
            .token
            .expect("Expected SignInResponse to carry token");

        // 4. 重新生成备用恢复码
        let gen_codes = request
            .post("/v1/backup-codes")
            .add_header("x-forwarded-for", "10.20.2.6")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {logged_in_token}"),
            )
            .json(&serde_json::json!({
                "password": password
            }))
            .await;
        assert_eq!(gen_codes.status_code(), StatusCode::OK);
        let new_codes: TwoFactorBackupCodesResource =
            serde_json::from_str(&gen_codes.text()).unwrap();
        assert_eq!(new_codes.backup_codes.len(), 10);

        // 5. 禁用两步验证
        let disable_res = request
            .delete("/v1/users/me/two-factor")
            .add_header("x-forwarded-for", "10.20.2.7")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {logged_in_token}"),
            )
            .json(&serde_json::json!({
                "password": password
            }))
            .await;
        assert_eq!(disable_res.status_code(), StatusCode::OK);

        // 6. 再次以邮箱密码登录 -> 不再拦截 2FA，直接返回 AuthResource
        let normal_login = request
            .post("/v1/sessions/email")
            .add_header("x-forwarded-for", "10.20.2.8")
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .await;
        assert_eq!(normal_login.status_code(), StatusCode::OK);
        let normal_json: serde_json::Value = serde_json::from_str(&normal_login.text()).unwrap();
        assert!(normal_json.get("token").is_some());
        assert!(normal_json.get("twoFactorRedirect").is_none());
    })
    .await;
}
