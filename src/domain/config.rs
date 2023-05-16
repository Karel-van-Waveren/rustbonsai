pub struct Config {
    pub live: i32,
    pub infinite: i32,
    pub screensaver: i32,
    pub print_tree: i32,
    pub verbosity: i32,
    pub life_start: i32,
    pub multiplier: i32,
    pub base_type: BaseType,
    pub seed: i32,
    pub leaves_size: i32,
    pub save: i32,
    pub load: i32,
    pub target_branch_count: i32,

    pub time_wait: u64,
    pub time_step: u64,

    pub message: String,
    pub leaves: [&'static str; 64],
    pub save_file: String,
    pub load_file: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            live: Default::default(),
            infinite: Default::default(),
            screensaver: Default::default(),
            print_tree: Default::default(),
            verbosity: 100,
            life_start: 64,
            multiplier: 10,
            base_type: BaseType::Big,
            seed: Default::default(),
            leaves_size: Default::default(),
            save: Default::default(),
            load: Default::default(),
            target_branch_count: Default::default(),
            time_wait: 4,
            time_step: 2,
            message: String::default(),
            leaves: ["&"; 64],
            save_file: String::default(),
            load_file: String::default(),
        }
    }
}

pub enum BaseType {
    Big,
    Small,
}
