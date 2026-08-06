<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>验证您的电子邮箱</title>
</head>
<body style="margin: 0; padding: 0; background-color: #f4f6f9; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; color: #334155; -webkit-font-smoothing: antialiased;">
  <table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0" style="background-color: #f4f6f9; padding: 40px 10px;">
    <tr>
      <td align="center">
        <!-- Main Card -->
        <table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0" style="max-width: 560px; background-color: #ffffff; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05), 0 2px 4px -1px rgba(0, 0, 0, 0.03); border: 1px solid #e2e8f0;">
          
          <!-- Header Bar -->
          <tr>
            <td style="background-color: #0f172a; padding: 28px 32px; text-align: left;">
              <span style="font-size: 20px; font-weight: 700; color: #ffffff; letter-spacing: -0.5px;">
                {{ app_name | default(value='Loco') }}
              </span>
            </td>
          </tr>

          <!-- Content Body -->
          <tr>
            <td style="padding: 36px 32px 32px 32px;">
              <h1 style="margin: 0 0 16px 0; font-size: 22px; font-weight: 700; color: #0f172a; line-height: 1.3;">
                验证您的电子邮箱
              </h1>
              
              <p style="margin: 0 0 16px 0; font-size: 15px; line-height: 1.6; color: #475569;">
                尊敬的 <strong>{{ name }}</strong>，欢迎加入 <strong>{{ app_name | default(value='Loco') }}</strong>！
              </p>
              
              <p style="margin: 0 0 28px 0; font-size: 15px; line-height: 1.6; color: #475569;">
                请点击下方按钮验证您的电子邮箱地址，以激活您的账号并开始使用我们的服务。
              </p>

              <!-- CTA Button -->
              <table role="presentation" cellspacing="0" cellpadding="0" border="0" style="margin: 0 0 28px 0;">
                <tr>
                  <td align="center" style="border-radius: 8px; background-color: #2563eb;">
                    <a href="{{host}}/api/auth/verify/{{verifyToken}}" target="_blank" style="display: inline-block; padding: 14px 28px; font-size: 15px; font-weight: 600; color: #ffffff; text-decoration: none; border-radius: 8px; background-color: #2563eb;">
                      验证我的电子邮箱
                    </a>
                  </td>
                </tr>
              </table>

              <!-- Safety Warning Box -->
              <div style="background-color: #f8fafc; border-left: 4px solid #3b82f6; padding: 14px 16px; border-radius: 4px; margin-bottom: 24px;">
                <p style="margin: 0; font-size: 13px; line-height: 1.5; color: #64748b;">
                  🔒 <strong>安全提示：</strong> 此验证链接在 24 小时内有效。如果您从未注册过账号，请直接忽略本邮件。
                </p>
              </div>

              <!-- Alternative Link Display -->
              <p style="margin: 0 0 8px 0; font-size: 12px; color: #94a3b8;">
                如果按钮无法点击，请复制下方链接到浏览器中打开：
              </p>
              <p style="margin: 0; font-size: 12px; line-height: 1.4; word-break: break-all;">
                <a href="{{host}}/api/auth/verify/{{verifyToken}}" style="color: #2563eb; text-decoration: underline;">
                  {{host}}/api/auth/verify/{{verifyToken}}
                </a>
              </p>
            </td>
          </tr>

          <!-- Footer -->
          <tr>
            <td style="background-color: #f8fafc; padding: 20px 32px; border-top: 1px solid #f1f5f9; text-align: center;">
              <p style="margin: 0; font-size: 12px; color: #94a3b8; line-height: 1.5;">
                本邮件由 {{ app_name | default(value='Loco') }} 系统自动发送，请勿直接回复。<br>
                &copy; {{ app_name | default(value='Loco') }}. All rights reserved.
              </p>
            </td>
          </tr>

        </table>
      </td>
    </tr>
  </table>
</body>
</html>
