use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Logger, Root},
    encode::pattern::PatternEncoder,
    Config,
};

// fn asserted_log_path(filename: &str) -> PathBuf {
//     PathBuf::from(std::env::var("DATA_FOLDER").expect("must have DATA_FOLDER env var set"))
//         .join(format!("logs/{filename}"))
// }

pub fn init_logging() {
    let pattern = {
        use owo_colors::OwoColorize;
        format!(
            "{} {} {} {} {} {}{{n}}",
            "{h({l:>5})}".bright_black(),
            "{d(%Y-%m-%d %H:%M:%S)}".blue(),
            "\u{2E2C}".magenta(),
            "{f}:{L}".bright_black(),
            "\u{21C0}".magenta(),
            "{m}".white()
        )
    };

    let encoder = Box::new(PatternEncoder::new(&pattern));

    let stdout = ConsoleAppender::builder()
        .tty_only(true)
        .encoder(encoder)
        .build();

    // let serenity_file = FileAppender::builder()
    //     .encoder(encoder)
    //     .build(asserted_log_path("serenity.log"))
    //     .expect("failed to build serenity file appender");

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        // .appender(Appender::builder().build("serenity_file", Box::new(serenity_file)))
        .logger(
            Logger::builder()
                .appender("stdout")
                .build("botofgreed_rs", LevelFilter::Trace),
        )
        // .logger(
        //     Logger::builder()
        //         .appender("stdout")
        //         .build("serenity", LevelFilter::Debug),
        // )
        .build(Root::builder().build(LevelFilter::Off))
        .expect("failed to build config");

    let _ = log4rs::init_config(config).expect("failed to initialize logger");
}
