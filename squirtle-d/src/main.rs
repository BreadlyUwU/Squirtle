// Trying to make SquirtleD its own binary
fn main() -> std::io::Result<()> {
    squirtle_core::utils::enable_debug_logging_if_needed();
    squirtle_d::start();
    Ok(())
}