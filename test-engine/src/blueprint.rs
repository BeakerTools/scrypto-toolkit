//! Defines traits to be implemented to declare a new blueprint

/// Trait to implement for a new blueprint
pub trait Blueprint {
    /// Returns the name of the function to instantiate the blueprint
    fn instantiation_name(&self) -> &str;

    /// Returns the name of the blueprint
    fn name(&self) -> &str;

    /// Returns the type of admin badge used by the blueprint
    fn admin_badge_type(&self) -> AdminBadge;
}

/// Defines the type of admin badge used by a blueprint
pub enum AdminBadge {
    Internal(usize),
    External(String),
    None,
}

impl AdminBadge {
    pub fn return_position(&self) -> usize {
        match self {
            AdminBadge::Internal(position) => position.clone(),
            _ => 0,
        }
    }
}
