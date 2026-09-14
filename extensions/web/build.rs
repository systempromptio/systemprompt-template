//! Discovers `schema/migrations/NNN_<name>.sql` for `extension_migrations!()`.

fn main() {
    systemprompt_extension::build::emit_migrations();
}
