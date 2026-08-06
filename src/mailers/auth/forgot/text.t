[{{ app_name | default(value='Loco') }}] 重置您的账户密码

尊敬的 {{ name }}：

我们收到了您重置 {{ app_name | default(value='Loco') }} 账户密码的申请。请复制或在浏览器中打开以下链接完成密码重置：

{{ host }}/reset#{{ resetToken }}

安全提示：
- 此重置链接将在 30 分钟内失效。
- 如果您未曾申请重置密码，请直接忽略本邮件，您的账户依然安全。

----------------------------------------
本邮件由系统自动发出，请勿直接回复。
&copy; {{ app_name | default(value='Loco') }}
