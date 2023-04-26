 GPL Session
==============

Manage sessions in your Solana Anchor Programs.


# Installation

```bash
cargo add gpl-session --features no-entrypoint
```

# Usage

1. Import the dependencies

```rust
use gpl_session::{SessionError, SessionToken, session_auth_or, Session};
```

2. Derive the `Session` trait on your instruction struct

```rust
#[derive(Accounts, Session)]
pub struct Instruction<'info> {
    .....
    pub user: Account<'info, User>,

    #[session(
        // The ephemeral keypair signing the transaction
        signer = signer,
        // The authority of the user account which must have created the session
        authority = user.authority.key()
    )]
    // Session Tokens are passed as optional accounts
    pub session_token: Option<Account<'info, SessionToken>>,

    #[account(mut)]
    pub signer: Signer<'info>,
    .....
}
```

3. Add the `session_auth_or` macro to your instruction handler with fallback logic on who the instruction should validate the signer when sessions are not present and an appropirate ErrorCode. If you've used `require*!` macros in anchor_lang you already know how this works.

```rust
#[session_auth_or(
    ctx.accounts.user.authority.key() == ctx.accounts.authority.key(),
    ErrorCode
)]
pub fn ix_handler(ctx: Context<Instruction>,) -> Result<()> {
.....
}

```
