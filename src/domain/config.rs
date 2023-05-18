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

impl Default for Config {
    fn default() -> Self {
        Self {
            live: true,
            infinite: true,
            screensaver: Default::default(),
            print_tree: Default::default(),
            verbose: true,
            life_start: 64,
            multiplier: 10,
            base_type: BaseType::Big,
            seed: Default::default(),
            leaves_size: 1,
            save: Default::default(),
            load: Default::default(),
            target_branch_count: Default::default(),
            time_wait: 4,
            time_step: 1000,
            message: String::default(),
            leaves: ['&'; 64].iter().collect(),
            save_file: String::default(),
            load_file: String::default(),
        }
    }
}

pub enum BaseType {
    Big,
    Small,
}
