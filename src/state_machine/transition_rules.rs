//! Transition rules and shared types for state machines

use cim_domain::{DomainResult};

/// Guard condition for state transitions
pub trait TransitionGuard<T>: Send + Sync {
    /// Check if the transition is allowed
    fn check(&self, context: &T) -> DomainResult<()>;
}

/// Effect to execute after successful transition
pub trait TransitionEffect<T, E>: Send + Sync {
    /// Execute the effect and return events
    fn execute(&self, context: &mut T) -> Vec<E>;
}

/// Common transition rules that can be reused across state machines
pub struct TransitionRules;

impl TransitionRules {
    /// Create a guard that always passes
    pub fn always_allow<T: 'static>() -> Box<dyn Fn(&T) -> DomainResult<()> + Send + Sync> {
        Box::new(|_| Ok(()))
    }

    /// Create a guard that checks a boolean condition
    pub fn when<T: 'static, F>(condition: F) -> Box<dyn Fn(&T) -> DomainResult<()> + Send + Sync>
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        Box::new(move |ctx| {
            if condition(ctx) {
                Ok(())
            } else {
                Err(cim_domain::DomainError::generic("Guard condition not met"))
            }
        })
    }

    /// Create a guard that checks multiple conditions (AND)
    pub fn all_of<T: 'static>(
        guards: Vec<Box<dyn Fn(&T) -> DomainResult<()> + Send + Sync>>,
    ) -> Box<dyn Fn(&T) -> DomainResult<()> + Send + Sync> {
        Box::new(move |ctx| {
            for guard in &guards {
                guard(ctx)?;
            }
            Ok(())
        })
    }

    /// Create a guard that checks any condition (OR)
    pub fn any_of<T: 'static>(
        guards: Vec<Box<dyn Fn(&T) -> DomainResult<()> + Send + Sync>>,
    ) -> Box<dyn Fn(&T) -> DomainResult<()> + Send + Sync> {
        Box::new(move |ctx| {
            let mut errors = Vec::new();
            for guard in &guards {
                match guard(ctx) {
                    Ok(()) => return Ok(()),
                    Err(e) => errors.push(e.to_string()),
                }
            }
            Err(cim_domain::DomainError::generic(format!(
                "None of the conditions were met: {:?}",
                errors
            )))
        })
    }

    /// Create an effect that does nothing
    pub fn no_effect<T: 'static, E: 'static>() -> Box<dyn Fn(&mut T) -> Vec<E> + Send + Sync> {
        Box::new(|_| Vec::new())
    }

    /// Create an effect that logs a message
    pub fn log_effect<T: 'static, E: 'static>(
        message: String,
    ) -> Box<dyn Fn(&mut T) -> Vec<E> + Send + Sync> {
        Box::new(move |_| {
            println!("{}", message);
            Vec::new()
        })
    }

    /// Combine multiple effects
    pub fn combine_effects<T: 'static, E: 'static>(
        effects: Vec<Box<dyn Fn(&mut T) -> Vec<E> + Send + Sync>>,
    ) -> Box<dyn Fn(&mut T) -> Vec<E> + Send + Sync> {
        Box::new(move |ctx| {
            let mut all_events = Vec::new();
            for effect in &effects {
                let events = effect(ctx);
                all_events.extend(events);
            }
            all_events
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestContext {
        value: i32,
        flag: bool,
    }

    #[test]
    fn test_always_allow() {
        let guard = TransitionRules::always_allow::<TestContext>();
        let ctx = TestContext { value: 42, flag: true };
        assert!(guard(&ctx).is_ok());
    }

    #[test]
    fn test_when_condition() {
        let guard = TransitionRules::when(|ctx: &TestContext| ctx.value > 40);
        
        let ctx1 = TestContext { value: 42, flag: true };
        assert!(guard(&ctx1).is_ok());
        
        let ctx2 = TestContext { value: 30, flag: true };
        assert!(guard(&ctx2).is_err());
    }

    #[test]
    fn test_all_of() {
        let guard = TransitionRules::all_of(vec![
            TransitionRules::when(|ctx: &TestContext| ctx.value > 40),
            TransitionRules::when(|ctx: &TestContext| ctx.flag),
        ]);
        
        let ctx1 = TestContext { value: 42, flag: true };
        assert!(guard(&ctx1).is_ok());
        
        let ctx2 = TestContext { value: 42, flag: false };
        assert!(guard(&ctx2).is_err());
        
        let ctx3 = TestContext { value: 30, flag: true };
        assert!(guard(&ctx3).is_err());
    }

    #[test]
    fn test_any_of() {
        let guard = TransitionRules::any_of(vec![
            TransitionRules::when(|ctx: &TestContext| ctx.value > 40),
            TransitionRules::when(|ctx: &TestContext| ctx.flag),
        ]);
        
        let ctx1 = TestContext { value: 42, flag: false };
        assert!(guard(&ctx1).is_ok()); // First condition passes
        
        let ctx2 = TestContext { value: 30, flag: true };
        assert!(guard(&ctx2).is_ok()); // Second condition passes
        
        let ctx3 = TestContext { value: 30, flag: false };
        assert!(guard(&ctx3).is_err()); // Neither condition passes
    }
} 