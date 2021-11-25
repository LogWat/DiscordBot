# DiscordBot
DiscordBotをRustでつくる

---
### メモとか
``Arc<Mutex<T>>``はスレットで可変共有参照するためのやつ

何故か`unknown_command`がnot foundになる

`#[required_permissions(ADMINISTRATOR)]`でadmin限定できるけどadminじゃないときの分岐が出来ない

### CurrentUserについて
- `ctx.http.get_current_user()`で取得できる
- `UserId`はこっから取れる

### 非同期処理について
- `tokio`を利用している
- `tokio`のランタイム上でタスクをspawnするとき，そのタスク内では外部の変数に通常アクセスしてはならない
    - アクセスしたい場合は`async move`を利用する