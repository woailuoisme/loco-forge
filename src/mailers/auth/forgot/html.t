<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>重置您的账户密码</title>
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
                重置您的账户密码
              </h1>
              
              <p style="margin: 0 0 16px 0; font-size: 15px; line-height: 1.6; color: #475569;">
                尊敬的 <strong>{{ name }}</strong>：
              </p>
              
              <p style="margin: 0 0 28px 0; font-size: 15px; line-height: 1.6; color: #475569;">
                我们收到了为您的 <strong>{{ app_name | default(value='Loco') }}</strong> 账户重置密码的申请。请点击下方按钮设置新密码：
              </p>

              <!-- CTA Button -->
              <table role="presentation" cellspacing="0" cellpadding="0" border="0" style="margin: 0 0 28px 0;">
                <tr>
                  <td align="center" style="border-radius: 8px; background-color: #dc2626;">
                    <a href="{{host}}/reset#{{resetToken}}" target="_blank" style="display: inline-block; padding: 14px 28px; font-size: 15px; font-weight: 600; color: #ffffff; text-decoration: none; border-radius: 8px; background-color: #dc2626;">
                      重置我的密码
                    </a>
                  </td>
                </tr>
              </table>

              <!-- Safety Warning Box -->
              <div style="background-color: #fffbebfb; border-left: 4px solid #f59e0b; padding: 14px 16px; border-radius: 4px; margin-bottom: 24px;">
                <p style="margin: 0; font-size: 13px; line-height: 1.5; color: #b45309;">
                  ⚠️ <strong>安全警示：</strong> 此重置链接将在 30 分钟内失效。如果您未曾发起此重置申请，请忽略此邮件，您的账户密码不会被改变。
                </p>
              </div>

              <!-- Alternative Link Display -->
              <p style="margin: 0 0 8px 0; font-size: 12px; color: #94a3b8;">
                如果按钮无法点击，请复制下方链接到浏览器中打开：
              </p>
              <p style="margin: 0; font-size: 12px; line-height: 1.4; word-break: break-all;">
                <a href="{{host}}/reset#{{resetToken}}" style="color: #dc2626; text-decoration: underline;">
                  {{host}}/reset#{{resetToken}}
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
