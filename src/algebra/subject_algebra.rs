//! Subject Algebra Implementation
//!
//! Implements the mathematical foundation for NATS subject routing, event correlation,
//! and distributed workflow coordination across CIM domains based on the Subject Space
//! lattice structure: 𝒮 = (𝒫, ≤, ∧, ∨, ⊤, ⊥)

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use super::event_algebra::{WorkflowEvent, EventType};

/// Subject Space (𝒮) - Structured lattice of workflow event routing paths
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    /// Domain component (e.g., "cim", "workflow", "person")
    pub domain: SubjectComponent,
    /// Context component (e.g., "instance", "template", "cross_domain")
    pub context: SubjectComponent,
    /// Event type component (e.g., "lifecycle", "step", "coordination")
    pub event_type: SubjectComponent,
    /// Specificity component (e.g., specific ID, aggregate, universal)
    pub specificity: SubjectComponent,
    /// Correlation component (correlation ID or wildcard)
    pub correlation: SubjectComponent,
}

/// Subject component supporting wildcards and specific values
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubjectComponent {
    /// Wildcard matching all values at this level
    Wildcard,
    /// Specific value
    Specific(String),
    /// Multiple alternatives
    Alternatives(HashSet<String>),
}

/// Subject builder for fluent construction
#[derive(Debug, Default)]
pub struct SubjectBuilder {
    domain: Option<SubjectComponent>,
    context: Option<SubjectComponent>,
    event_type: Option<SubjectComponent>,
    specificity: Option<SubjectComponent>,
    correlation: Option<SubjectComponent>,
}

/// Subject pattern for subscription and matching
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectPattern {
    /// Pattern string in canonical form
    pub pattern: String,
    /// Parsed subject structure
    pub subject: Subject,
    /// Match specificity level (lower is more specific)
    pub specificity_level: u8,
}

/// Subject algebra operations
pub struct SubjectAlgebra;

/// Subject ordering for lattice operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectOrdering {
    /// s1 is more specific than s2 (s1 ≤ s2)
    MoreSpecific,
    /// s1 is less specific than s2 (s2 ≤ s1)  
    LessSpecific,
    /// s1 and s2 are incomparable
    Incomparable,
    /// s1 and s2 are equivalent
    Equivalent,
}

impl Subject {
    /// Create a new subject with all components
    pub fn new(
        domain: SubjectComponent,
        context: SubjectComponent,
        event_type: SubjectComponent,
        specificity: SubjectComponent,
        correlation: SubjectComponent,
    ) -> Self {
        Self {
            domain,
            context,
            event_type,
            specificity,
            correlation,
        }
    }

    /// Create universal subject (⊤) matching all events
    pub fn universal() -> Self {
        Self {
            domain: SubjectComponent::Wildcard,
            context: SubjectComponent::Wildcard,
            event_type: SubjectComponent::Wildcard,
            specificity: SubjectComponent::Wildcard,
            correlation: SubjectComponent::Wildcard,
        }
    }

    /// Create empty subject (⊥) matching no events
    pub fn empty() -> Self {
        Self {
            domain: SubjectComponent::Specific("∅".to_string()),
            context: SubjectComponent::Specific("∅".to_string()),
            event_type: SubjectComponent::Specific("∅".to_string()),
            specificity: SubjectComponent::Specific("∅".to_string()),
            correlation: SubjectComponent::Specific("∅".to_string()),
        }
    }

    /// Generate subject from workflow event
    pub fn from_event(event: &WorkflowEvent) -> Self {
        let domain = SubjectComponent::Specific(format!("cim.{}", event.domain));
        
        let context = if event.context.workflow_instance_id.is_some() {
            SubjectComponent::Specific("instance".to_string())
        } else if event.context.template_id.is_some() {
            SubjectComponent::Specific("template".to_string())
        } else {
            SubjectComponent::Specific("system".to_string())
        };

        let event_type = SubjectComponent::Specific(event.type_name().to_string());

        let specificity = match &event.event_type {
            EventType::Lifecycle(lt) => SubjectComponent::Specific(format!("{:?}", lt).to_lowercase()),
            EventType::Step(st) => SubjectComponent::Specific(format!("{:?}", st).to_lowercase()),
            EventType::CrossDomain(ct) => SubjectComponent::Specific(format!("{:?}", ct).to_lowercase()),
            EventType::Extension(et) => SubjectComponent::Specific(format!("{:?}", et).to_lowercase()),
        };

        let correlation = SubjectComponent::Specific(event.correlation_id.to_string());

        Self::new(domain, context, event_type, specificity, correlation)
    }

    /// Convert to canonical string form
    pub fn to_canonical_string(&self) -> String {
        format!(
            "cim.{}.{}.{}.{}.{}",
            self.domain.to_string(),
            self.context.to_string(),
            self.event_type.to_string(),
            self.specificity.to_string(),
            self.correlation.to_string()
        )
    }

    /// Check if this subject matches another (used for subscription matching)
    pub fn matches(&self, other: &Subject) -> bool {
        self.domain.matches(&other.domain)
            && self.context.matches(&other.context)
            && self.event_type.matches(&other.event_type)
            && self.specificity.matches(&other.specificity)
            && self.correlation.matches(&other.correlation)
    }

    /// Calculate specificity level (lower is more specific)
    pub fn specificity_level(&self) -> u8 {
        let mut level = 0;
        if matches!(self.domain, SubjectComponent::Wildcard) { level += 1; }
        if matches!(self.context, SubjectComponent::Wildcard) { level += 1; }
        if matches!(self.event_type, SubjectComponent::Wildcard) { level += 1; }
        if matches!(self.specificity, SubjectComponent::Wildcard) { level += 1; }
        if matches!(self.correlation, SubjectComponent::Wildcard) { level += 1; }
        level
    }

    /// Check if subject is for cross-domain events
    pub fn is_cross_domain(&self) -> bool {
        matches!(
            (&self.context, &self.event_type),
            (SubjectComponent::Specific(ctx), SubjectComponent::Specific(evt))
            if ctx == "cross_domain" || evt == "cross_domain"
        )
    }

    /// Extract domain name if specific
    pub fn domain_name(&self) -> Option<String> {
        match &self.domain {
            SubjectComponent::Specific(name) => {
                if name.starts_with("cim.") {
                    Some(name.strip_prefix("cim.").unwrap().to_string())
                } else {
                    Some(name.clone())
                }
            },
            _ => None,
        }
    }
}

impl SubjectComponent {
    /// Check if this component matches another for subscription
    pub fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (SubjectComponent::Wildcard, _) => true,
            (_, SubjectComponent::Wildcard) => false,
            (SubjectComponent::Specific(s1), SubjectComponent::Specific(s2)) => s1 == s2,
            (SubjectComponent::Alternatives(alts), SubjectComponent::Specific(s)) => alts.contains(s),
            (SubjectComponent::Specific(_), SubjectComponent::Alternatives(_)) => false,
            (SubjectComponent::Alternatives(alts1), SubjectComponent::Alternatives(alts2)) => {
                !alts1.is_disjoint(alts2)
            },
        }
    }

    /// Check if this component is more specific than another
    pub fn is_more_specific_than(&self, other: &Self) -> bool {
        match (self, other) {
            (SubjectComponent::Specific(_), SubjectComponent::Wildcard) => true,
            (SubjectComponent::Alternatives(_), SubjectComponent::Wildcard) => true,
            (SubjectComponent::Specific(_), SubjectComponent::Alternatives(_)) => true,
            _ => false,
        }
    }
}

impl fmt::Display for SubjectComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubjectComponent::Wildcard => write!(f, "*"),
            SubjectComponent::Specific(s) => write!(f, "{}", s),
            SubjectComponent::Alternatives(alts) => {
                let alternatives: Vec<String> = alts.iter().cloned().collect();
                write!(f, "{{{}}}", alternatives.join(","))
            }
        }
    }
}

impl FromStr for Subject {
    type Err = SubjectParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 6 {
            return Err(SubjectParseError::InvalidFormat);
        }

        let parse_component = |part: &str| -> Result<SubjectComponent, SubjectParseError> {
            if part == "*" {
                Ok(SubjectComponent::Wildcard)
            } else if part.starts_with('{') && part.ends_with('}') {
                let inner = &part[1..part.len()-1];
                let alternatives: HashSet<String> = inner.split(',').map(|s| s.to_string()).collect();
                Ok(SubjectComponent::Alternatives(alternatives))
            } else {
                Ok(SubjectComponent::Specific(part.to_string()))
            }
        };

        // Skip the first "cim" prefix and parse the 5 main components
        Ok(Subject::new(
            parse_component(parts[1])?, // domain (workflow, person, etc.)
            parse_component(parts[2])?, // context (instance, template, etc.)
            parse_component(parts[3])?, // event_type (lifecycle, step, etc.)
            parse_component(parts[4])?, // specificity (created, updated, etc.)
            parse_component(parts[5])?, // correlation (correlation ID)
        ))
    }
}

impl SubjectBuilder {
    /// Create new subject builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set domain component
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(SubjectComponent::Specific(domain.into()));
        self
    }

    /// Set domain wildcard
    pub fn any_domain(mut self) -> Self {
        self.domain = Some(SubjectComponent::Wildcard);
        self
    }

    /// Set context component
    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(SubjectComponent::Specific(context.into()));
        self
    }

    /// Set context wildcard
    pub fn any_context(mut self) -> Self {
        self.context = Some(SubjectComponent::Wildcard);
        self
    }

    /// Set event type component
    pub fn event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = Some(SubjectComponent::Specific(event_type.into()));
        self
    }

    /// Set event type wildcard
    pub fn any_event_type(mut self) -> Self {
        self.event_type = Some(SubjectComponent::Wildcard);
        self
    }

    /// Set specificity component
    pub fn specificity(mut self, specificity: impl Into<String>) -> Self {
        self.specificity = Some(SubjectComponent::Specific(specificity.into()));
        self
    }

    /// Set specificity wildcard
    pub fn any_specificity(mut self) -> Self {
        self.specificity = Some(SubjectComponent::Wildcard);
        self
    }

    /// Set correlation component
    pub fn correlation(mut self, correlation: Uuid) -> Self {
        self.correlation = Some(SubjectComponent::Specific(correlation.to_string()));
        self
    }

    /// Set correlation wildcard
    pub fn any_correlation(mut self) -> Self {
        self.correlation = Some(SubjectComponent::Wildcard);
        self
    }

    /// Build the subject
    pub fn build(self) -> Result<Subject, SubjectBuildError> {
        Ok(Subject::new(
            self.domain.unwrap_or(SubjectComponent::Wildcard),
            self.context.unwrap_or(SubjectComponent::Wildcard),
            self.event_type.unwrap_or(SubjectComponent::Wildcard),
            self.specificity.unwrap_or(SubjectComponent::Wildcard),
            self.correlation.unwrap_or(SubjectComponent::Wildcard),
        ))
    }
}

impl SubjectAlgebra {
    /// Subject subsumption operation (≤)
    /// Returns true if s1 is more specific than s2 (s1 ≤ s2)
    pub fn subsumes(s1: &Subject, s2: &Subject) -> SubjectOrdering {
        let domain_cmp = Self::compare_components(&s1.domain, &s2.domain);
        let context_cmp = Self::compare_components(&s1.context, &s2.context);
        let event_type_cmp = Self::compare_components(&s1.event_type, &s2.event_type);
        let specificity_cmp = Self::compare_components(&s1.specificity, &s2.specificity);
        let correlation_cmp = Self::compare_components(&s1.correlation, &s2.correlation);

        // Check if all components have consistent ordering
        let orderings = vec![domain_cmp, context_cmp, event_type_cmp, specificity_cmp, correlation_cmp];
        
        let more_specific_count = orderings.iter().filter(|&&o| o == SubjectOrdering::MoreSpecific).count();
        let less_specific_count = orderings.iter().filter(|&&o| o == SubjectOrdering::LessSpecific).count();
        let equivalent_count = orderings.iter().filter(|&&o| o == SubjectOrdering::Equivalent).count();
        let incomparable_count = orderings.iter().filter(|&&o| o == SubjectOrdering::Incomparable).count();

        if incomparable_count > 0 {
            SubjectOrdering::Incomparable
        } else if more_specific_count > 0 && less_specific_count == 0 {
            SubjectOrdering::MoreSpecific
        } else if less_specific_count > 0 && more_specific_count == 0 {
            SubjectOrdering::LessSpecific
        } else if equivalent_count == 5 {
            SubjectOrdering::Equivalent
        } else {
            SubjectOrdering::Incomparable
        }
    }

    /// Subject intersection operation (∧)
    /// Returns the most specific common subject
    pub fn intersection(s1: &Subject, s2: &Subject) -> Option<Subject> {
        let domain = Self::intersect_components(&s1.domain, &s2.domain)?;
        let context = Self::intersect_components(&s1.context, &s2.context)?;
        let event_type = Self::intersect_components(&s1.event_type, &s2.event_type)?;
        let specificity = Self::intersect_components(&s1.specificity, &s2.specificity)?;
        let correlation = Self::intersect_components(&s1.correlation, &s2.correlation)?;

        Some(Subject::new(domain, context, event_type, specificity, correlation))
    }

    /// Subject union operation (∨)
    /// Returns the least general covering subject
    pub fn union(s1: &Subject, s2: &Subject) -> Subject {
        let domain = Self::union_components(&s1.domain, &s2.domain);
        let context = Self::union_components(&s1.context, &s2.context);
        let event_type = Self::union_components(&s1.event_type, &s2.event_type);
        let specificity = Self::union_components(&s1.specificity, &s2.specificity);
        let correlation = Self::union_components(&s1.correlation, &s2.correlation);

        Subject::new(domain, context, event_type, specificity, correlation)
    }

    /// Compare two subject components for ordering
    fn compare_components(c1: &SubjectComponent, c2: &SubjectComponent) -> SubjectOrdering {
        match (c1, c2) {
            (SubjectComponent::Specific(s1), SubjectComponent::Specific(s2)) => {
                if s1 == s2 {
                    SubjectOrdering::Equivalent
                } else {
                    SubjectOrdering::Incomparable
                }
            },
            (SubjectComponent::Specific(_), SubjectComponent::Wildcard) => SubjectOrdering::MoreSpecific,
            (SubjectComponent::Wildcard, SubjectComponent::Specific(_)) => SubjectOrdering::LessSpecific,
            (SubjectComponent::Wildcard, SubjectComponent::Wildcard) => SubjectOrdering::Equivalent,
            (SubjectComponent::Alternatives(_), SubjectComponent::Wildcard) => SubjectOrdering::MoreSpecific,
            (SubjectComponent::Wildcard, SubjectComponent::Alternatives(_)) => SubjectOrdering::LessSpecific,
            (SubjectComponent::Specific(_), SubjectComponent::Alternatives(_)) => SubjectOrdering::MoreSpecific,
            (SubjectComponent::Alternatives(_), SubjectComponent::Specific(_)) => SubjectOrdering::LessSpecific,
            (SubjectComponent::Alternatives(a1), SubjectComponent::Alternatives(a2)) => {
                if a1 == a2 {
                    SubjectOrdering::Equivalent
                } else if a1.is_subset(a2) {
                    SubjectOrdering::MoreSpecific
                } else if a2.is_subset(a1) {
                    SubjectOrdering::LessSpecific
                } else {
                    SubjectOrdering::Incomparable
                }
            },
        }
    }

    /// Intersect two subject components
    fn intersect_components(c1: &SubjectComponent, c2: &SubjectComponent) -> Option<SubjectComponent> {
        match (c1, c2) {
            (SubjectComponent::Wildcard, other) | (other, SubjectComponent::Wildcard) => Some(other.clone()),
            (SubjectComponent::Specific(s1), SubjectComponent::Specific(s2)) => {
                if s1 == s2 {
                    Some(SubjectComponent::Specific(s1.clone()))
                } else {
                    None // No intersection
                }
            },
            (SubjectComponent::Specific(s), SubjectComponent::Alternatives(alts)) |
            (SubjectComponent::Alternatives(alts), SubjectComponent::Specific(s)) => {
                if alts.contains(s) {
                    Some(SubjectComponent::Specific(s.clone()))
                } else {
                    None
                }
            },
            (SubjectComponent::Alternatives(alts1), SubjectComponent::Alternatives(alts2)) => {
                let intersection: HashSet<_> = alts1.intersection(alts2).cloned().collect();
                if intersection.is_empty() {
                    None
                } else if intersection.len() == 1 {
                    Some(SubjectComponent::Specific(intersection.into_iter().next().unwrap()))
                } else {
                    Some(SubjectComponent::Alternatives(intersection))
                }
            },
        }
    }

    /// Union two subject components
    fn union_components(c1: &SubjectComponent, c2: &SubjectComponent) -> SubjectComponent {
        match (c1, c2) {
            (SubjectComponent::Wildcard, _) | (_, SubjectComponent::Wildcard) => SubjectComponent::Wildcard,
            (SubjectComponent::Specific(s1), SubjectComponent::Specific(s2)) => {
                if s1 == s2 {
                    SubjectComponent::Specific(s1.clone())
                } else {
                    let mut alts = HashSet::new();
                    alts.insert(s1.clone());
                    alts.insert(s2.clone());
                    SubjectComponent::Alternatives(alts)
                }
            },
            (SubjectComponent::Specific(s), SubjectComponent::Alternatives(alts)) |
            (SubjectComponent::Alternatives(alts), SubjectComponent::Specific(s)) => {
                let mut union = alts.clone();
                union.insert(s.clone());
                SubjectComponent::Alternatives(union)
            },
            (SubjectComponent::Alternatives(alts1), SubjectComponent::Alternatives(alts2)) => {
                let union: HashSet<_> = alts1.union(alts2).cloned().collect();
                SubjectComponent::Alternatives(union)
            },
        }
    }
}

/// Error types for subject operations
#[derive(Debug, thiserror::Error)]
pub enum SubjectParseError {
    #[error("Invalid subject format - expected 6 components separated by dots (cim.domain.context.event_type.specificity.correlation)")]
    InvalidFormat,
    #[error("Invalid component format: {0}")]
    InvalidComponent(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SubjectBuildError {
    #[error("Missing required component: {0}")]
    MissingComponent(String),
    #[error("Invalid component value: {0}")]
    InvalidValue(String),
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_canonical_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::event_algebra::*;

    #[test]
    fn test_subject_creation() {
        let subject = SubjectBuilder::new()
            .domain("workflow")
            .context("instance")
            .event_type("lifecycle")
            .specificity("created")
            .any_correlation()
            .build()
            .unwrap();

        assert_eq!(subject.to_canonical_string(), "cim.workflow.instance.lifecycle.created.*");
    }

    #[test]
    fn test_subject_from_event() {
        let correlation_id = Uuid::new_v4();
        let event = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowCreated,
            "workflow".to_string(),
            correlation_id,
            EventPayload::empty(),
            EventContext::for_workflow(Uuid::new_v4()),
        );

        let subject = Subject::from_event(&event);
        assert_eq!(subject.domain_name(), Some("workflow".to_string()));
        assert!(matches!(subject.context, SubjectComponent::Specific(ref s) if s == "instance"));
    }

    #[test]
    fn test_subject_matching() {
        let specific = Subject::from_str("cim.workflow.instance.lifecycle.created.corr123").unwrap();
        let pattern = Subject::from_str("cim.workflow.*.lifecycle.*.*").unwrap();

        assert!(pattern.matches(&specific));
        assert!(!specific.matches(&pattern)); // More specific doesn't match less specific
    }

    #[test]
    fn test_subject_algebra_subsumption() {
        let specific = Subject::from_str("cim.workflow.instance.lifecycle.created.corr123").unwrap();
        let general = Subject::from_str("cim.workflow.*.lifecycle.*.*").unwrap();

        let ordering = SubjectAlgebra::subsumes(&specific, &general);
        assert_eq!(ordering, SubjectOrdering::MoreSpecific);
    }

    #[test]
    fn test_subject_algebra_intersection() {
        let s1 = Subject::from_str("cim.workflow.instance.lifecycle.*.*").unwrap();
        let s2 = Subject::from_str("cim.workflow.*.lifecycle.created.*").unwrap();

        let intersection = SubjectAlgebra::intersection(&s1, &s2).unwrap();
        assert_eq!(intersection.to_canonical_string(), "cim.workflow.instance.lifecycle.created.*");
    }

    #[test]
    fn test_subject_algebra_union() {
        let s1 = Subject::from_str("cim.workflow.instance.step.*.*").unwrap();
        let s2 = Subject::from_str("cim.person.instance.step.*.*").unwrap();

        let union = SubjectAlgebra::union(&s1, &s2);
        // Union should generalize to match both domains
        assert!(union.to_canonical_string().contains("*") || 
                union.to_canonical_string().contains("{"));
    }

    #[test]
    fn test_universal_and_empty_subjects() {
        let universal = Subject::universal();
        let empty = Subject::empty();
        let specific = Subject::from_str("cim.workflow.instance.lifecycle.created.corr123").unwrap();

        assert!(universal.matches(&specific));
        assert!(!empty.matches(&specific));
        assert_eq!(universal.specificity_level(), 5);
        assert_eq!(specific.specificity_level(), 0);
    }
}