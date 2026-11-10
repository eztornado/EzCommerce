//! Data access lives in `app/Support/db.rs` (dynamic `RowMap` helpers over
//! `sqlx::Any`) and in the controllers' SQL, mirroring Bagisto's repositories.
//! Row metadata that needs to be shared across controllers is declared here.
