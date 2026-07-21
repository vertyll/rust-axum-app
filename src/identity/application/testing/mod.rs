//! Test doubles for the identity ports: in-memory adapters good enough to
//! exercise every use case without a database, SMTP or real crypto.

mod fake_hasher;
mod fake_mailer;
mod fake_tokens;
mod harness;
mod in_memory_sessions;
mod in_memory_users;
mod test_ports;

pub(crate) use harness::{confirmed_user, harness, register_cmd};
