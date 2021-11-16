# DiscordBot
DiscordBotをRustでつくる

---
### メモとか
``Arc<Mutex<T>>``はスレットで可変共有参照するためのやつ

何故か`unknown_command`がnot foundになる

`#[required_permissions(ADMINISTRATOR)]`でadmin限定できるけどadminじゃないときの分岐が出来ない