[{{ app_name | default(value='Loco') }}] 验证您的电子邮箱

尊敬的 {{ name }}：

欢迎加入 {{ app_name | default(value='Loco') }}！

为了保障您的账户安全，请复制或在浏览器中打开以下链接，完成电子邮箱验证与账号激活：

{{ host }}/api/auth/verify/{{ verifyToken }}

注意：
- 此验证链接在 24 小时内有效。
- 如果您从未在 {{ app_name | default(value='Loco') }} 注册过账号，请直接忽略本邮件。

----------------------------------------
本邮件由系统自动发出，请勿直接回复。
&copy; {{ app_name | default(value='Loco') }}
