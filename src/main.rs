#[macro_use]
extern crate clap;

#[macro_use]
extern crate slog;

use slog::Drain;

fn main() {
    let args = app_from_crate!()
        .about("Automatically absorb staged changes into your current branch")
        .arg(
            clap::Arg::with_name("base")
                .help("Use this commit as the base of the absorb stack")
                .short("b")
                .long("base")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("dry-run")
                .help("Don't make any actual changes")
                .short("n")
                .long("dry-run")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name("force")
                .help("Skip safety checks")
                .short("f")
                .long("force")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name("verbose")
                .help("Display more output")
                .short("v")
                .long("verbose")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name("and-rebase")
                .help("Run rebase if successful")
                .short("r")
                .long("and-rebase")
                .takes_value(false),
        )
        .get_matches();

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = slog::LevelFilter::new(
        drain,
        if args.is_present("verbose") {
            slog::Level::Debug
        } else {
            slog::Level::Info
        },
    )
    .fuse();
    let mut logger = slog::Logger::root(drain, o!());
    if args.is_present("verbose") {
        logger = logger.new(o!(
            "module" => slog::FnValue(|record| record.module()),
            "line" => slog::FnValue(|record| record.line()),
        ));
    }

    if let Err(e) = git_absorb::run(&git_absorb::Config {
        dry_run: args.is_present("dry-run"),
        force: args.is_present("force"),
        base: args.value_of("base"),
        and_rebase: args.is_present("and-rebase"),
        logger: &logger,
    }) {
        crit!(logger, "absorb failed"; "err" => e.to_string());
        // wait for async logger to finish writing messages
        drop(logger);
        ::std::process::exit(1);
    }
}
