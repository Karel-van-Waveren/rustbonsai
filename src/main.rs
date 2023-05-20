#![allow(dead_code)]

extern crate ncurses;
use std::{sync::Mutex, thread::sleep, time::Duration};

use domain::{branch_type::BranchType, config::BaseType};
use ncurses::{
    cbreak, clear, curs_set, del_panel, delwin, doupdate, endwin, getmaxy, getmaxyx, has_colors,
    init_pair, mvwinch, mvwprintw, new_panel, newwin, nodelay, noecho, overlay, overwrite,
    pair_content, refresh, savetty, start_color, stdscr, timeout, update_panels,
    use_default_colors, wattroff, wattron, wgetch, wprintw, A_BOLD, COLORS, COLOR_BLACK,
    COLOR_PAIR, ERR,
};
use once_cell::sync::OnceCell;
use rand::{rngs::StdRng, Rng, SeedableRng};
use set_deltas::set_deltas;

use crate::domain::{config::Config, counters::Counters, ncurses_objects::NcursesObjects};

mod domain;
mod set_deltas;

static RNG: OnceCell<Mutex<StdRng>> = OnceCell::new();

fn main() {
    let mut tree = Tree::from_args();

    RNG.set(Mutex::new(StdRng::seed_from_u64(tree.config.seed)))
        .unwrap();

    loop {
        tree.init();
        tree.grow_tree();
        if tree.config.load {
            tree.config.target_branch_count = 0;
        }
        if tree.config.infinite {
            timeout(tree.config.time_wait * 1000);
            if tree.check_key_press() {
                break;
            }
        }
        if !tree.config.infinite {
            break;
        }
    }

    if tree.config.print_tree {
        tree.finish();

        // overlay all windows onto stdscr
        overlay(tree.objects.base_win, stdscr());
        overlay(tree.objects.tree_win, stdscr());
        overwrite(tree.objects.message_border_win, stdscr());
        overwrite(tree.objects.message_win, stdscr());

        printstdscr();
    } else {
        wgetch(tree.objects.tree_win);
        tree.finish();
    }
    //
    // loop {
    //     sleep(Duration::from_secs(1));
    // }
}

// print stdscr to terminal window
fn printstdscr() {
    todo!();
}

#[derive(Default)]
struct Tree {
    config: Config,
    objects: NcursesObjects,
    counters: Counters,
}

impl Tree {
    fn from_args() -> Self {
        Self {
            config: Config::from_args(),
            objects: NcursesObjects::default(),
            counters: Counters::default(),
        }
    }

    fn init(&mut self) {
        savetty();
        noecho();
        curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        cbreak();
        nodelay(stdscr(), true);

        // if terminal has color capabilities, use them
        if has_colors() {
            start_color();

            // use native background color when possible
            let bg = if use_default_colors() == ERR {
                COLOR_BLACK
            } else {
                -1
            };

            // define color pairs
            for i in 0..16 {
                init_pair(i, i, bg);
            }

            // restrict color pallete in non-256color terminals (e.g. screen or linux)
            if COLORS() < 256 {
                init_pair(8, 7, bg); // gray will look white
                init_pair(9, 1, bg);
                init_pair(10, 2, bg);
                init_pair(11, 3, bg);
                init_pair(12, 4, bg);
                init_pair(13, 5, bg);
                init_pair(14, 6, bg);
                init_pair(15, 7, bg);
            }
        } else {
            println!("Warning: terminal does not have color support.");
        }

        // define and draw windows, then create panels
        self.draw_wins();
        self.draw_message();
    }

    fn grow_tree(&mut self) {
        let mut max_y = 0;
        let mut max_x = 0;
        getmaxyx(self.objects.tree_win, &mut max_y, &mut max_x);

        // reset counters
        self.counters.shoots = 0;
        self.counters.branches = 0;
        self.counters.shoot_counter = rand();

        if self.config.verbose {
            mvwprintw(
                self.objects.tree_win,
                2,
                5,
                format!("maxX: {max_x}, maxY: {max_y}").as_str(),
            );
        }

        self.branch(
            max_y - 1,
            max_x / 2,
            BranchType::Trunk,
            self.config.life_start,
        );

        update_panels();
        doupdate();
    }

    fn branch(&mut self, mut y: i32, mut x: i32, branch_type: BranchType, mut life: i32) {
        self.counters.branches += 1;
        let mut dx;
        let mut dy;
        let mut shoot_cooldown = self.config.multiplier;

        while life > 0 {
            life -= 1;
            let age = self.config.life_start - life;

            (dx, dy) = set_deltas(branch_type, life, age, self.config.multiplier);

            let max_y = getmaxy(self.objects.tree_win);
            if dy > 0 && y > (max_y - 2) {
                dy -= 1; // reduce dy if too close to the ground
            }

            // near-dead branch should branch into a lot of leaves
            if life < 3 {
                self.branch(y, x, BranchType::Dead, life);
            }
            // dying trunk/branch should branch into a lot of leaves
            else if (BranchType::Trunk == branch_type
                || BranchType::ShootLeft == branch_type
                || BranchType::ShootRight == branch_type)
                && life < (self.config.multiplier + 2)
            {
                self.branch(y, x, BranchType::Dying, life);
            }
            // trunks should re-branch if not close to ground AND either randomly, or upon every <multiplier> steps
            else if (BranchType::Trunk == branch_type && (dice(3)) == 0)
                || (life % self.config.multiplier == 0)
            {
                // if trunk is branching and not about to die, create another trunk with random life
                if dice(8) == 0 && life > 7 {
                    // reset shoot cooldown
                    shoot_cooldown = self.config.multiplier * 2;
                    self.branch(y, x, BranchType::Trunk, life + (dice(5) - 2));
                }
                // otherwise create a shoot
                else if shoot_cooldown <= 0 {
                    // reset shoot cooldown
                    shoot_cooldown = self.config.multiplier * 2;

                    let shoot_life = life + self.config.multiplier;

                    // first shoot is randomly directed
                    self.counters.shoots += 1;
                    self.counters.shoot_counter += 1;
                    if self.config.verbose {
                        mvwprintw(
                            self.objects.tree_win,
                            4,
                            5,
                            format!("shoots: {}", self.counters.shoots).as_str(),
                        );
                    }
                    // create shoot
                    let direction = match self.counters.shoot_counter % 2 {
                        0 => BranchType::ShootLeft,
                        1 => BranchType::ShootRight,
                        _ => BranchType::Dead,
                    };
                    self.branch(y, x, direction, shoot_life);
                }
            }
            shoot_cooldown -= 1;

            if self.config.verbose {
                mvwprintw(self.objects.tree_win, 5, 5, format!("dx: {dx}").as_str());
                mvwprintw(self.objects.tree_win, 6, 5, format!("dy: {dy}").as_str());
                mvwprintw(
                    self.objects.tree_win,
                    7,
                    5,
                    format!("branchtype: {branch_type:?}").as_str(),
                );
                mvwprintw(
                    self.objects.tree_win,
                    8,
                    5,
                    format!("shootCooldown: {shoot_cooldown:?}").as_str(),
                );
            }

            // move in x and y directions
            x += dx;
            y += dy;

            self.choose_color(branch_type);

            // choose string to use for this branch
            let branchstr = self.choose_string(branch_type, life, dx, dy);

            mvwprintw(self.objects.tree_win, y, x, branchstr);
            wattroff(self.objects.tree_win, A_BOLD());
            if self.config.live {
                update_screen(self.config.time_step);
            }
        }
    }

    // based on type of tree, determine what color a branch should be
    fn choose_color(&self, branch_type: BranchType) {
        match branch_type {
            BranchType::Trunk | BranchType::ShootLeft | BranchType::ShootRight => {
                if dice(2) == 0 {
                    wattron(self.objects.tree_win, A_BOLD() | COLOR_PAIR(11));
                } else {
                    wattron(self.objects.tree_win, COLOR_PAIR(3));
                }
            }
            BranchType::Dying => {
                if dice(10) == 0 {
                    wattron(self.objects.tree_win, A_BOLD() | COLOR_PAIR(2));
                } else {
                    wattron(self.objects.tree_win, COLOR_PAIR(2));
                }
            }
            BranchType::Dead => {
                if dice(3) == 0 {
                    wattron(self.objects.tree_win, A_BOLD() | COLOR_PAIR(10));
                } else {
                    wattron(self.objects.tree_win, COLOR_PAIR(10));
                }
            }
        }
    }

    fn choose_string(&self, mut branch_type: BranchType, life: i32, dx: i32, dy: i32) -> &str {
        let fallback_char = "?";

        if life < 4 {
            branch_type = BranchType::Dying;
        }

        match branch_type {
            BranchType::Trunk => {
                if dy == 0 {
                    "/~"
                } else if dx < 0 {
                    "\\|"
                } else if dx == 0 {
                    "/|\\"
                } else if dx > 0 {
                    "|/"
                } else {
                    fallback_char
                }
            }
            BranchType::ShootLeft => {
                if dy > 0 {
                    "\\"
                } else if dy == 0 {
                    "\\_"
                } else if dx < 0 {
                    "\\|"
                } else if dx == 0 {
                    "/|"
                } else if dx > 0 {
                    "/"
                } else {
                    fallback_char
                }
            }
            BranchType::ShootRight => {
                if dy > 0 {
                    "/"
                } else if dy == 0 {
                    "_/"
                } else if dx < 0 {
                    "\\|"
                } else if dx == 0 {
                    "/|"
                } else if dx > 0 {
                    "/"
                } else {
                    fallback_char
                }
            }
            BranchType::Dying | BranchType::Dead => {
                if self.config.leaves_size > 0 {
                    &self.config.leaves[..=(dice(self.config.leaves_size) as usize)]
                } else {
                    ""
                }
            }
        }
    }

    fn draw_wins(&mut self) {
        let mut rows = 0;
        let mut cols = 0;

        let (base_width, base_height) = match self.config.base_type {
            BaseType::None => (0, 0),
            BaseType::Small => (15, 3),
            BaseType::Big => (31, 4),
        };

        // calculate where base should go
        getmaxyx(stdscr(), &mut rows, &mut cols);
        let base_origin_y = rows - base_height;
        let base_origin_x = (cols / 2) - (base_width / 2);

        // clean up old objects
        // del_objects(objects);

        // create windows
        self.objects.base_win = newwin(base_height, base_width, base_origin_y, base_origin_x);
        self.objects.tree_win = newwin(rows - base_height, cols, 0, 0);

        // // create tree and base panels
        self.objects.base_panel = new_panel(self.objects.base_win);
        self.objects.tree_panel = new_panel(self.objects.tree_win);

        self.draw_base();
    }

    fn draw_message(&self) {
        if self.config.message.is_empty() {
            return;
        }
        todo!();
    }

    fn del_objects(objects: &NcursesObjects) {
        // this seg faults
        del_panel(objects.base_panel);
        del_panel(objects.tree_panel);
        del_panel(objects.message_border_panel);
        del_panel(objects.message_panel);

        delwin(objects.base_win);
        delwin(objects.tree_win);
        delwin(objects.message_border_win);
        delwin(objects.message_win);
    }

    fn draw_base(&self) {
        match self.config.base_type {
            BaseType::None => {}
            BaseType::Small => {
                wattron(self.objects.base_win, COLOR_PAIR(8));
                wprintw(self.objects.base_win, "(");
                wattron(self.objects.base_win, COLOR_PAIR(2));
                wprintw(self.objects.base_win, "---");
                wattron(self.objects.base_win, COLOR_PAIR(11));
                wprintw(self.objects.base_win, "./~~~\\.");
                wattron(self.objects.base_win, COLOR_PAIR(2));
                wprintw(self.objects.base_win, "---");
                wattron(self.objects.base_win, COLOR_PAIR(8));
                wprintw(self.objects.base_win, ")");

                mvwprintw(self.objects.base_win, 1, 0, " (           ) ");
                mvwprintw(self.objects.base_win, 2, 0, "  (_________)  ");
            }
            BaseType::Big => {
                wattron(self.objects.base_win, A_BOLD() | COLOR_PAIR(8));
                wprintw(self.objects.base_win, ":");
                wattron(self.objects.base_win, COLOR_PAIR(2));
                wprintw(self.objects.base_win, "___________");
                wattron(self.objects.base_win, COLOR_PAIR(11));
                wprintw(self.objects.base_win, "./~~~\\.");
                wattron(self.objects.base_win, COLOR_PAIR(2));
                wprintw(self.objects.base_win, "___________");
                wattron(self.objects.base_win, COLOR_PAIR(8));
                wprintw(self.objects.base_win, ":");

                mvwprintw(
                    self.objects.base_win,
                    1,
                    0,
                    " \\                           / ",
                );
                mvwprintw(
                    self.objects.base_win,
                    2,
                    0,
                    "  \\_________________________/ ",
                );
                mvwprintw(self.objects.base_win, 3, 0, "  (_)                     (_)");

                wattroff(self.objects.base_win, A_BOLD());
            }
        }
    }

    // check for key press
    fn check_key_press(&self) -> bool {
        if (self.config.screensaver && wgetch(stdscr()) != ERR) || (wgetch(stdscr()) == 'q' as i32)
        {
            self.finish();
            true
        } else {
            false
        }
    }

    fn finish(&self) {
        clear();
        refresh();
        endwin();
        if self.config.save {
            // saveToFile
        }
    }
}

fn update_screen(time_step: u64) {
    update_panels();
    doupdate();
    if time_step > 0 {
        sleep(Duration::from_millis(time_step));
    }
}
fn dice(sides: i32) -> i32 {
    rand() % sides
}
fn rand() -> i32 {
    RNG.get().unwrap().lock().unwrap().gen::<i32>().abs()
}
