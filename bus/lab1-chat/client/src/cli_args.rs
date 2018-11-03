use clap::{App, Arg, ArgMatches};

pub fn get_args() -> ArgMatches<'static> {
    App::new("Client-server tcp chat")
        .version("1.0")
        .author("Szymon Baginski <baginski.szymon@gmail.com>")
        .arg(
            Arg::with_name("nickname")
                .short("n")
                .long("nickname")
                .value_name("NICKNAME")
                .help("Sets your nickname.")
                .takes_value(true),
        ).arg(
            Arg::with_name("encryption")
                .short("e")
                .long("encryption")
                .value_name("NONE|XOR|CEZAR")
                .help("Sets encryption method.")
                .takes_value(true),
        ).get_matches()
}
