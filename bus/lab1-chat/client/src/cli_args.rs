use clap::{App, Arg, ArgMatches};

pub fn get_args() -> ArgMatches<'static> {
    App::new("Client tcp chat")
        .version("1.0")
        .author("Szymon Baginski <baginski.szymon@gmail.com>")
        .arg(
            Arg::with_name("nickname")
                .short("n")
                .long("nickname")
                .value_name("NICKNAME")
                .help("Sets your nickname. \"anonymous\" if not given")
                .takes_value(true),
        ).arg(
            Arg::with_name("encryption")
                .short("e")
                .long("encryption")
                .value_name("NONE|XOR|CEZAR")
                .help("Sets encryption method.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("address")
                .short("a")
                .long("address")
                .value_name("ADDRESS")
                .help("Sets ipv4 address of server. Default is 127.0.0.1")
                .takes_value(true),
        ).arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("Sets port of server. Default is 12345")
                .takes_value(true),
        )
        .get_matches()
}
