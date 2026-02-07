/// Trait for infallible conversions from entity models to domain models.
///
/// Use this trait when the conversion cannot fail and doesn't require error handling.
///
/// # Type Parameters
/// * `Domain` - The target domain type to convert to
/// * `Context` - Optional context needed for the conversion (defaults to `()`)
pub trait EntityToDomain<Domain, Context = ()> {
    /// Converts an entity model to a domain model
    ///
    /// # Arguments
    /// * `self` - The entity to convert
    /// * `context` - Additional context needed for the conversion
    ///
    /// # Returns
    /// The converted domain model
    fn entity_to_domain(self, context: Context) -> Domain;
}

/// Trait for fallible conversions from entity models to domain models.
///
/// Use this trait when the conversion can fail and requires error handling.
///
/// # Type Parameters
/// * `Domain` - The target domain type to convert to
/// * `Context` - Optional context needed for the conversion (defaults to `()`)
pub trait TryEntityToDomain<Domain, Context = ()> {
    /// The error type that can occur during conversion
    type Error;

    /// Attempts to convert an entity model to a domain model
    ///
    /// # Arguments
    /// * `self` - The entity to convert
    /// * `context` - Additional context needed for the conversion
    ///
    /// # Returns
    /// The converted domain model or an error
    fn try_entity_to_domain(self, context: Context) -> Result<Domain, Self::Error>;
}
