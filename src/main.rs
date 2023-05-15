#![allow(dead_code)]

extern crate ncurses;
use std::{thread::sleep, time::Duration};

use domain::{branch_type::BranchType, config::BaseType};
use ncurses::*;
use rand::Rng;

use crate::domain::{config::Config, counters::Counters, ncurses_objects::NcursesObjects};

mod domain;

fn main() {
    let mut tree = Tree::default();
    tree.init();
    tree.grow_tree();
    loop {
        sleep(Duration::from_millis(1));
    }
}

#[derive(Default)]
struct Tree {
    config: Config,
    objects: NcursesObjects,
    counters: Counters,
}

impl Tree {
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

        if self.config.verbosity > 0 {
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

            (dx, dy) = self.set_deltas(branch_type, life, age, self.config.multiplier);

            let max_y = getmaxy(self.objects.tree_win);
            if dy > 0 && y > (max_y - 2) {
                dy -= 1; // reduce dy if too close to the ground
            }

            // near-dead branch should branch into a lot of leaves
            if life < 3 {
                self.branch(y, x, BranchType::Dead, life);
            }
            // dying trunk should branch into a lot of leaves
            else if BranchType::Trunk == branch_type && life < (self.config.multiplier + 2) {
                self.branch(y, x, BranchType::Dying, life);
            }
            // dying shoot should branch into a lot of leaves
            else if (BranchType::ShootLeft == branch_type
                || BranchType::ShootRight == branch_type)
                && life < self.config.multiplier + 2
            {
                self.branch(y, x, BranchType::Dying, life);
            }
            // trunks should re-branch if not close to ground AND either randomly, or upon every <multiplier> steps
            else if BranchType::Trunk == branch_type && (dice(3)) == 0
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
                    if self.config.verbosity > 0 {
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

            if self.config.verbosity > 0 {
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
                    format!("shootCooldown: % {shoot_cooldown}").as_str(),
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
            update_screen(self.config.time_step);
        }
    }

    fn set_deltas(
        &self,
        branch_type: BranchType,
        life: i32,
        age: i32,
        multiplier: i32,
    ) -> (i32, i32) {
        let mut dx = 0;
        let mut dy = 0;
        match branch_type {
            BranchType::Trunk => {
                // new or dead trunk
                if age <= 2 || life < 4 {
                    dy = 0;
                    dx = (rand() % 3) - 1;
                }
                // young trunk should grow wide
                else if (age < (multiplier * 3)) {
                    // every (multiplier * 0.8) steps, raise tree to next level
                    if (age % ((multiplier as f32 * 0.5) as i32) == 0) {
                        dy = -1;
                    } else {
                        dy = 0;
                    }

                    let roll = dice(10);
                    if (roll >= 0 && roll <= 0) {
                        dx = -2;
                    } else if (roll >= 1 && roll <= 3) {
                        dx = -1;
                    } else if (roll >= 4 && roll <= 5) {
                        dx = 0;
                    } else if (roll >= 6 && roll <= 8) {
                        dx = 1;
                    } else if (roll >= 9 && roll <= 9) {
                        dx = 2;
                    }
                }
                // middle-aged trunk
                else {
                    let roll = dice(10);
                    if roll > 2 {
                        dy = -1;
                    } else {
                        dy = 0;
                    }
                    dx = (dice(3)) - 1;
                }
            }
            BranchType::ShootLeft => {
                // left shoot: trend left and little vertical movement
                let roll = dice(10);
                if roll >= 0 && roll <= 1 {
                    dy = -1;
                } else if roll >= 2 && roll <= 7 {
                    dy = 0;
                } else if (roll >= 8 && roll <= 9) {
                    dy = 1;
                }

                let roll = dice(10);
                if roll >= 0 && roll <= 1 {
                    dx = -2;
                } else if roll >= 2 && roll <= 5 {
                    dx = -1;
                } else if roll >= 6 && roll <= 8 {
                    dx = 0;
                } else if roll >= 9 && roll <= 9 {
                    dx = 1;
                }
            }
            BranchType::ShootRight => {
                // right shoot: trend right and little vertical movement
                let roll = dice(10);
                if (roll >= 0 && roll <= 1) {
                    dy = -1;
                } else if (roll >= 2 && roll <= 7) {
                    dy = 0;
                } else if (roll >= 8 && roll <= 9) {
                    dy = 1;
                }

                let roll = dice(10);
                if (roll >= 0 && roll <= 1) {
                    dx = 2;
                } else if (roll >= 2 && roll <= 5) {
                    dx = 1;
                } else if (roll >= 6 && roll <= 8) {
                    dx = 0;
                } else if (roll >= 9 && roll <= 9) {
                    dx = -1;
                }
            }
            BranchType::Dying => {
                // dying: discourage vertical growth(?); trend left/right (-3,3)
                let roll = dice(10);
                if (roll >= 0 && roll <= 1) {
                    dy = -1;
                } else if (roll >= 2 && roll <= 8) {
                    dy = 0;
                } else if (roll >= 9 && roll <= 9) {
                    dy = 1;
                }

                let roll = dice(15);
                if (roll >= 0 && roll <= 0) {
                    dx = -3;
                } else if (roll >= 1 && roll <= 2) {
                    dx = -2;
                } else if (roll >= 3 && roll <= 5) {
                    dx = -1;
                } else if (roll >= 6 && roll <= 8) {
                    dx = 0;
                } else if (roll >= 9 && roll <= 11) {
                    dx = 1;
                } else if (roll >= 12 && roll <= 13) {
                    dx = 2;
                } else if (roll >= 14 && roll <= 14) {
                    dx = 3;
                }
            }
            BranchType::Dead => {
                // dead: fill in surrounding area
                let roll = dice(10);
                if (roll >= 0 && roll <= 2) {
                    dy = -1;
                } else if (roll >= 3 && roll <= 6) {
                    dy = 0;
                } else if (roll >= 7 && roll <= 9) {
                    dy = 1;
                }
                dx = (dice(3)) - 1;
            }
        }
        (dx, dy)
    }

    // based on type of tree, determine what color a branch should be
    fn choose_color(&self, branch_type: BranchType) {
        match branch_type {
            BranchType::Trunk | BranchType::ShootLeft | BranchType::ShootRight => {
                if rand() % 2 == 0 {
                    wattron(self.objects.tree_win, A_BOLD() | COLOR_PAIR(11));
                } else {
                    wattron(self.objects.tree_win, COLOR_PAIR(3));
                }
            }
            BranchType::Dying => {
                if rand() % 10 == 0 {
                    wattron(self.objects.tree_win, A_BOLD() | COLOR_PAIR(2));
                } else {
                    wattron(self.objects.tree_win, COLOR_PAIR(2));
                }
            }
            BranchType::Dead => {
                if rand() % 3 == 0 {
                    wattron(self.objects.tree_win, A_BOLD() | COLOR_PAIR(10));
                } else {
                    wattron(self.objects.tree_win, COLOR_PAIR(10));
                }
            }
        }
    }

    const fn choose_string(
        &self,
        mut branch_type: BranchType,
        life: i32,
        dx: i32,
        dy: i32,
    ) -> &str {
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
                // strncpy(branchStr, conf->leaves[rand() % conf->leavesSize], maxStrLen - 1);
                // branchStr[maxStrLen - 1] = '\0';
                "&"
            }
        }
    }

    fn draw_wins(&mut self) {
        let mut rows = 0;
        let mut cols = 0;

        let (base_width, base_height) = match self.config.base_type {
            BaseType::Big => (31, 4),
            BaseType::Small => (15, 3),
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
    rand::thread_rng().gen::<i32>()
}
