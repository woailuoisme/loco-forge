[{{ app_name | default(value='Loco') }}] 您的登录验证码

尊敬的 {{ name }}：

您正在登录 {{ app_name | default(value='Loco') }}。

您的 6 位数字验证码为：

    {{ code | default(value=token) }}

安全提示：
- 验证码有效期为 10 分钟。
- 严禁将此验证码透露、截图或告知任何人，官方客服绝不会向您索要此验证码。
- 如果您未曾发起登录申请，请立即忽略本邮件。

----------------------------------------
本邮件由系统自动发出，请勿直接回复。
&copy; {{ app_name | default(value='Loco') }}