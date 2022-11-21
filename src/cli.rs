use clap::{Arg, ArgAction, ArgMatches, Command};

pub fn cmd() -> Command {
    Command::new("Crypto Alerter")
        .author("Blatko(Leon)")
        .version("1.0")
        .about(
            "Small program which takes price levels as \
            input and outputs alerts on crossovers.",
        )
        .arg(Arg::new("symbol").required(true))
        .arg(
            Arg::new("sfx")
                .long("sound")
                .short('s')
                .alias("sfx")
                .help("Provide a sound file to play at each alert"),
        )
        .arg(
            Arg::new("price_trigger")
                .action(ArgAction::Append)
                .value_parser(clap::value_parser!(f64))
                .required(true),
        )
}
