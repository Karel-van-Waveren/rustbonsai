use crate::{dice, domain::branch_type::BranchType, rand};

pub fn set_deltas(branch_type: BranchType, life: i32, age: i32, multiplier: i32) -> (i32, i32) {
    let mut dx = 0;
    let mut dy = 0;
    match branch_type {
        BranchType::Trunk => {
            set_delta_trunk(age, life, &mut dy, &mut dx, multiplier);
        }
        BranchType::ShootLeft | BranchType::ShootRight => {
            set_delta_shoot(&mut dy, &mut dx, branch_type);
        }
        BranchType::Dying => {
            set_deltas_dying(&mut dy, &mut dx);
        }
        BranchType::Dead => {
            set_deltas_dead(&mut dy, &mut dx);
        }
    }
    (dx, dy)
}

fn set_deltas_dead(dy: &mut i32, dx: &mut i32) {
    // dead: fill in surrounding area
    let roll = dice(10);
    if (0..=2).contains(&roll) {
        *dy = -1;
    } else if (3..=6).contains(&roll) {
        *dy = 0;
    } else if (7..=9).contains(&roll) {
        *dy = 1;
    }
    *dx = (dice(3)) - 1;
}

fn set_deltas_dying(dy: &mut i32, dx: &mut i32) {
    // dying: discourage vertical growth(?); trend left/right (-3,3)
    let roll = dice(10);
    if (0..=1).contains(&roll) {
        *dy = -1;
    } else if (2..=8).contains(&roll) {
        *dy = 0;
    } else if roll == 9 {
        *dy = 1;
    }

    let roll = dice(15);
    if roll == 0 {
        *dx = -3;
    } else if (1..=2).contains(&roll) {
        *dx = -2;
    } else if (3..=5).contains(&roll) {
        *dx = -1;
    } else if (6..=8).contains(&roll) {
        *dx = 0;
    } else if (9..=11).contains(&roll) {
        *dx = 1;
    } else if (12..=13).contains(&roll) {
        *dx = 2;
    } else if roll == 14 {
        *dx = 3;
    }
}

fn set_delta_shoot(dy: &mut i32, dx: &mut i32, branch_type: BranchType) {
    // left shoot: trend left and little vertical movement
    let roll = dice(10);
    if (0..=1).contains(&roll) {
        *dy = -1;
    } else if (2..=7).contains(&roll) {
        *dy = 0;
    } else if (8..=9).contains(&roll) {
        *dy = 1;
    }

    let roll = dice(10);
    if (0..=1).contains(&roll) {
        *dx = -2;
    } else if (2..=5).contains(&roll) {
        *dx = -1;
    } else if (6..=8).contains(&roll) {
        *dx = 0;
    } else if roll == 9 {
        *dx = 1;
    }

    if BranchType::ShootRight == branch_type {
        *dx = -*dx;
    }
}

fn set_delta_trunk(age: i32, life: i32, dy: &mut i32, dx: &mut i32, multiplier: i32) {
    // new or dead trunk
    if age <= 2 || life < 4 {
        *dy = 0;
        *dx = (rand() % 3) - 1;
    }
    // young trunk should grow wide
    else if age < multiplier * 3 {
        // every (multiplier * 0.8) steps, raise tree to next level
        if age % (multiplier as f32 * 0.5) as i32 == 0 {
            *dy = -1;
        } else {
            *dy = 0;
        }

        let roll = dice(10);
        if roll == 0 {
            *dx = -2;
        } else if (1..=3).contains(&roll) {
            *dx = -1;
        } else if (4..=5).contains(&roll) {
            *dx = 0;
        } else if (6..=8).contains(&roll) {
            *dx = 1;
        } else if roll == 9 {
            *dx = 2;
        }
    }
    // middle-aged trunk
    else {
        let roll = dice(10);
        if roll > 2 {
            *dy = -1;
        } else {
            *dy = 0;
        }
        *dx = (dice(3)) - 1;
    }
}
