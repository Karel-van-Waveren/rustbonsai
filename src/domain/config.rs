use clap::{arg, command};
use rand::Rng;

#[allow(clippy::struct_excessive_bools)]
pub struct Config {
    pub live: bool,
    pub infinite: bool,
    pub screensaver: bool,
    pub print_tree: bool,
    pub verbose: bool,
    pub life_start: i32,
    pub multiplier: i32,
    pub base_type: BaseType,
    pub seed: u64,
    pub leaves_size: i32,
    pub save: bool,
    pub load: bool,
    pub target_branch_count: i32,

    pub time_wait: i32,
    pub time_step: u64,

    pub message: String,
    pub leaves: String,
    pub save_file: String,
    pub load_file: String,
}

impl Config {
    pub fn from_args() -> Self {
        let matches = get_arg_matches();
        match parse_arg_matches(&matches) {
            Ok(ok) => ok,
            Err(why) => {
                println!("couldnt read args {why} \n defaulting to defaults");
                Self::default()
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            live: false,
            infinite: false,
            screensaver: false,
            print_tree: false,
            verbose: false,
            life_start: 64,
            multiplier: 10,
            base_type: BaseType::Big,
            seed: rand::thread_rng().gen::<u64>(),
            leaves_size: 1,
            save: false,
            load: false,
            target_branch_count: 0,
            time_wait: 4,
            time_step: 30,
            message: String::default(),
            leaves: ['&'; 64].iter().collect(),
            save_file: String::default(),
            load_file: String::default(),
        }
    }
}

pub enum BaseType {
    None,
    Small,
    Big,
}

#[allow(clippy::cognitive_complexity)]
fn get_arg_matches() -> clap::ArgMatches {
    command!()
        .arg(arg!(-l --live "live mode: show each step of growth"))
        .arg(arg!(-t --time <TIME> "in live mode, wait TIME secs between steps of growth (must be larger than 0) [default: 0.03]"))
        .arg(arg!(-i --infinite "infinite mode: keep growing trees"))
        .arg(arg!(-w --wait <TIME> "in infinite mode, wait TIME between each tree generation [default: 4.00]"))
        .arg(arg!(-S --screensaver "screensaver mode; equivalent to -li and quit on any keypress"))
        .arg(arg!(-m --message <STR> "attach message next to the tree"))
        .arg(arg!(-b --base <INT> "ascii-art plant base to use, 0 is none"))
        .arg(arg!(-c --leaf <LIST> "list of comma-delimited strings randomly chosen for leaves"))
        .arg(arg!(-M --multiplier <INT> "branch multiplier; higher -> more branching (0-20) [default: 5]"))
        .arg(arg!(-L --life <INT> "life; higher -> more growth (0-200) [default: 32]"))
        .arg(arg!(-p --print "print tree to terminal when finished"))
        .arg(arg!(-s --seed <INT> "seed random number generator"))
        .arg(arg!(-W --save <FILE> "save progress to file [default: $XDG_CACHE_HOME/cbonsai or $HOME/.cache/cbonsai]"))
        .arg(arg!(-C --load <FILE> "load progress from file [default: $XDG_CACHE_HOME/cbonsai]"))
        .arg(arg!(-v --verbose "increase output verbosity"))
        .get_matches()
}

fn parse_arg_matches(matches: &clap::ArgMatches) -> anyhow::Result<Config> {
    let mut config = Config::default();
    if let Some(value) = matches.get_one::<bool>("live") {
        config.live = *value;
    }

    if let Some(value) = matches.get_one::<String>("time") {
        config.time_step = value.parse()?;
    }

    if let Some(value) = matches.get_one::<bool>("infinite") {
        config.infinite = *value;
    }

    if let Some(value) = matches.get_one::<String>("wait") {
        config.time_wait = value.parse()?;
    }

    if let Some(value) = matches.get_one::<bool>("screensaver") {
        config.live = *value;
        config.infinite = *value;

        config.save = *value;
        config.load = *value;

        config.screensaver = *value;
    }

    if let Some(value) = matches.get_one::<String>("message") {
        config.message = value.clone();
    }

    if let Some(value) = matches.get_one::<String>("base") {
        config.base_type = match value.parse()? {
            1 => BaseType::Small,
            2 => BaseType::Big,
            _ => BaseType::None,
        }
    }

    if let Some(value) = matches.get_one::<String>("leaf") {
        config.leaves = value.clone();
    }

    if let Some(value) = matches.get_one::<String>("multiplier") {
        config.multiplier = value.parse()?;
    }

    if let Some(value) = matches.get_one::<String>("life") {
        config.life_start = value.parse()?;
    }

    if let Some(value) = matches.get_one::<bool>("print") {
        config.print_tree = *value;
    }

    if let Some(value) = matches.get_one::<String>("seed") {
        config.seed = value.parse()?;
    }

    if let Some(value) = matches.get_one::<String>("save") {
        config.save = !value.is_empty();
        config.save_file = value.clone();
    }

    if let Some(value) = matches.get_one::<String>("load") {
        config.load = !value.is_empty();
        config.load_file = value.clone();
    }

    if let Some(value) = matches.get_one::<bool>("verbose") {
        config.verbose = *value;
    }

    Ok(config)
}
