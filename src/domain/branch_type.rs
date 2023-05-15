#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BranchType {
    Trunk,
    ShootLeft,
    ShootRight,
    Dying,
    Dead,
}
