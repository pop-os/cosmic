fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    pop_cosmic::run();
}
