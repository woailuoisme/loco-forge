<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>登录验证码</title>
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
                登录验证码
              </h1>
              
              <p style="margin: 0 0 16px 0; font-size: 15px; line-height: 1.6; color: #475569;">
                尊敬的 <strong>{{ name }}</strong>：
              </p>
              
              <p style="margin: 0 0 24px 0; font-size: 15px; line-height: 1.6; color: #475569;">
                您正在登录 <strong>{{ app_name | default(value='Loco') }}</strong>，请在登录页面输入以下 6 位数字验证码：
              </p>

              <!-- 6-digit OTP Code Box -->
              <table role="presentation" cellspacing="0" cellpadding="0" border="0" style="margin: 0 auto 28px auto; width: 100%;">
                <tr>
                  <td align="center" style="background-color: #f1f5f9; border-radius: 8px; padding: 20px 24px; border: 2px dashed #cbd5e1;">
                    <span style="font-family: 'SF Mono', Consolas, Monaco, monospace; font-size: 32px; font-weight: 800; color: #4f46e5; letter-spacing: 10px; display: inline-block;">
                      {{ code | default(value=token) }}
                    </span>
                  </td>
                </tr>
              </table>

              <!-- Critical Security Warning Box -->
              <div style="background-color: #fef2f2; border-left: 4px solid #ef4444; padding: 14px 16px; border-radius: 4px; margin-bottom: 24px;">
                <p style="margin: 0; font-size: 13px; line-height: 1.5; color: #991b1b;">
                  ⛔ <strong>安全提示：</strong> 验证码有效期为 10 分钟。<strong>请勿将此验证码透露、截图或告知任何人！</strong>
                </p>
              </div>
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
