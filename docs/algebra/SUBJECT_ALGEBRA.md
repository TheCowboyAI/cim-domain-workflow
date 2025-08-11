# Workflow Subject Algebra

## Mathematical Foundation

The **Workflow Subject Algebra** defines the mathematical structure for NATS subject routing, event correlation, and distributed workflow coordination across CIM domains.

## Subject Space Definition

The Subject Space **𝒮** is defined as a structured lattice of workflow event routing paths:

**𝒮 = (𝒫, ≤, ∧, ∨, ⊤, ⊥)**

Where:
- **𝒫**: Set of all possible subject paths
- **≤**: Hierarchical ordering relation (subsumption)
- **∧**: Subject intersection (common routing)
- **∨**: Subject union (broadcast routing) 
- **⊤**: Universal subject (matches all)
- **⊥**: Empty subject (matches none)

## Subject Structure

### Canonical Form

Every subject s ∈ 𝒮 has the canonical form:

```
s = domain.context.event_type.specificity.correlation

Where:
- domain: CIM domain identifier
- context: workflow or cross-domain context
- event_type: lifecycle, step, coordination, extension
- specificity: instance, aggregate, or universal scope
- correlation: causation chain identifier
```

### Domain Hierarchy

```
𝒮_domain = {
  cim.workflow.*,           // Workflow domain events
  cim.person.*,             // Person domain events  
  cim.document.*,           // Document domain events
  cim.location.*,           // Location domain events
  cim.organization.*,       // Organization domain events
  cim.integration.*         // Cross-domain integration
}
```

### Context Classification

```
𝒮_context = {
  workflow.instance.*,      // Specific workflow instance
  workflow.template.*,      // Template-based operations
  cross_domain.*,           // Inter-domain coordination
  system.*                  // System-level operations
}
```

## Event Type Taxonomy

### Lifecycle Events

```
𝒮_lifecycle = {
  *.*.lifecycle.created.*,
  *.*.lifecycle.started.*,
  *.*.lifecycle.paused.*,
  *.*.lifecycle.resumed.*,
  *.*.lifecycle.completed.*,
  *.*.lifecycle.failed.*,
  *.*.lifecycle.cancelled.*
}
```

### Step Events

```
𝒮_step = {
  *.*.step.created.*,
  *.*.step.started.*,
  *.*.step.completed.*,
  *.*.step.failed.*,
  *.*.step.skipped.*,
  *.*.step.waiting.*
}
```

### Cross-Domain Events

```
𝒮_cross_domain = {
  *.cross_domain.request.*,
  *.cross_domain.response.*,
  *.cross_domain.coordination.*,
  *.cross_domain.transaction.*,
  *.cross_domain.synchronization.*
}
```

## Algebraic Operations

### 1. Subject Subsumption (≤)

**Definition**: s₁ ≤ s₂ if s₁ is more specific than s₂

```
cim.workflow.instance.lifecycle.created.corr123 
≤ cim.workflow.instance.lifecycle.*
≤ cim.workflow.instance.*
≤ cim.workflow.*  
≤ cim.*
≤ *
```

**Properties**:
- **Reflexivity**: s ≤ s
- **Transitivity**: s₁ ≤ s₂ ∧ s₂ ≤ s₃ ⟹ s₁ ≤ s₃
- **Antisymmetry**: s₁ ≤ s₂ ∧ s₂ ≤ s₁ ⟹ s₁ = s₂

### 2. Subject Intersection (∧)

**Definition**: Greatest lower bound of subject routing

```
s₁ ∧ s₂ = most_specific_common_subject(s₁, s₂)
```

**Example**:
```
cim.workflow.instance.lifecycle.* ∧ cim.workflow.*.lifecycle.created.*
= cim.workflow.instance.lifecycle.created.*
```

**Properties**:
- **Commutativity**: s₁ ∧ s₂ = s₂ ∧ s₁
- **Associativity**: (s₁ ∧ s₂) ∧ s₃ = s₁ ∧ (s₂ ∧ s₃)
- **Idempotency**: s ∧ s = s

### 3. Subject Union (∨)

**Definition**: Least upper bound for broadcast routing

```
s₁ ∨ s₂ = least_general_covering_subject(s₁, s₂)
```

**Example**:
```
cim.workflow.instance.step.* ∨ cim.person.instance.step.*  
= *.*.instance.step.*
```

**Properties**:
- **Commutativity**: s₁ ∨ s₂ = s₂ ∨ s₁
- **Associativity**: (s₁ ∨ s₂) ∨ s₃ = s₁ ∨ (s₂ ∨ s₃)
- **Idempotency**: s ∨ s = s

## Subject Lattice Structure

### Bounded Lattice

The subject space forms a **bounded lattice** with:

- **Top Element (⊤)**: `*` (universal subject matching all events)
- **Bottom Element (⊥)**: `∅` (empty subject matching no events)

### Distributive Laws

- **∧ distributes over ∨**: s₁ ∧ (s₂ ∨ s₃) = (s₁ ∧ s₂) ∨ (s₁ ∧ s₃)
- **∨ distributes over ∧**: s₁ ∨ (s₂ ∧ s₃) = (s₁ ∨ s₂) ∧ (s₁ ∨ s₃)

## Correlation Chain Algebra

### Chain Structure

Correlation chains form a **partially ordered set** with concatenation:

```
ℂ = (Chains, ⊆, ∘)

Where:
- Chains: Set of all correlation sequences
- ⊆: Subsequence relation
- ∘: Chain concatenation
```

### Chain Operations

**Concatenation**:
```
c₁ ∘ c₂ = append(c₁, c₂) with temporal ordering preserved
```

**Chain Intersection**:
```
c₁ ∩ c₂ = common_causal_ancestry(c₁, c₂)
```

**Chain Projection**:
```
π_domain(c) = filter_chain_by_domain(c, domain)
```

## Subject Generation Functions

### Template-Based Generation

For workflow template t and instance i:

```
subject_template(t, i) = cim.workflow.template.{t.id}.instance.{i.id}.*
subject_instance(i) = cim.workflow.instance.{i.id}.*
subject_correlation(c) = *.*.*.*.{c.id}
```

### Cross-Domain Generation

For cross-domain operation from domain d₁ to domain d₂:

```
subject_cross_domain(d₁, d₂, op) = 
  cim.integration.cross_domain.{d₁}_to_{d₂}.{op}.*
```

### Event-Specific Generation

For event e with type t in context ctx:

```
subject_event(e, t, ctx) = 
  {e.domain}.{ctx}.{t}.{e.specificity}.{e.correlation_id}
```

## Subscription Patterns

### Hierarchical Subscriptions

```rust
// Subscribe to all workflow lifecycle events
"cim.workflow.*.lifecycle.*"

// Subscribe to specific workflow instance
"cim.workflow.instance.{workflow_id}.*"

// Subscribe to cross-domain coordination
"cim.integration.cross_domain.*"

// Subscribe to specific correlation chain  
"*.*.*.*.{correlation_id}"
```

### Filtered Subscriptions

```rust
// Domain-specific workflow events
"cim.workflow.* AND domain={requesting_domain}"

// Priority event routing
"*.*.*.urgent.* OR *.*.*.critical.*"

// Temporal-based filtering
"*.*.* WHERE timestamp > {since}"
```

## Subject Transformation Rules

### Domain Boundary Crossing

When event crosses from domain d₁ to d₂:

```
transform_subject(s, d₁, d₂) = 
  replace_domain_segment(s, d₁, d₂) ∘ add_cross_domain_marker(s)
```

### Context Elevation

When workflow escalates to system level:

```
elevate_context(s, ctx_old, ctx_new) =
  replace_context_segment(s, ctx_old, ctx_new) ∘ preserve_correlation(s)
```

### Specificity Refinement

When adding instance-specific information:

```
refine_specificity(s, spec_old, spec_new) =
  s with specificity_segment replaced and subsumption preserved
```

## Routing Algebra

### Message Routing Function

For message m with subject s:

```
route(m, s) = {subscribers | subscriber.pattern matches s}
```

### Multi-Cast Routing

For broadcast message to multiple domains:

```
multicast(m, domains) = ⋃_{d ∈ domains} route(m, project_to_domain(s, d))
```

### Conditional Routing

With routing condition c:

```
conditional_route(m, s, c) = 
  if evaluate_condition(c, m.context) 
  then route(m, s) 
  else ∅
```

## Implementation Patterns

### Subject Builder Pattern

```rust
SubjectBuilder::new()
    .domain("workflow")
    .context("instance")
    .event_type("lifecycle")
    .specificity("created")
    .correlation_id("corr123")
    .build() // → "cim.workflow.instance.lifecycle.created.corr123"
```

### Pattern Matching

```rust
match subject {
    "cim.workflow.*.lifecycle.started.*" => handle_workflow_start(),
    "cim.*.cross_domain.request.*" => handle_cross_domain_request(),
    "*.*.*.*.{correlation}" => handle_correlated_event(correlation),
    _ => handle_unknown_event()
}
```

### Subscription Management

```rust
// Hierarchical subscription with automatic refinement
subscription_manager
    .subscribe("cim.workflow.*")
    .refine_on_condition(|event| event.priority == High)
    .route_to(high_priority_handler);
```

## Mathematical Properties

### Lattice Homomorphisms

Subject transformations preserve lattice structure:

```
f: 𝒮₁ → 𝒮₂ is a lattice homomorphism if:
- f(s₁ ∧ s₂) = f(s₁) ∧ f(s₂)
- f(s₁ ∨ s₂) = f(s₁) ∨ f(s₂)
```

### Order Preservation

Subject refinements preserve subsumption:

```
If s₁ ≤ s₂, then refine(s₁, r) ≤ refine(s₂, r)
```

### Correlation Consistency

Correlation chains maintain causal ordering:

```
If e₁ causes e₂, then correlation(e₁) is prefix of correlation(e₂)
```

## Applications

The Subject Algebra enables:

1. **Hierarchical Event Routing**: Efficient message distribution based on subject hierarchy
2. **Cross-Domain Coordination**: Mathematically precise inter-domain communication
3. **Correlation Tracking**: Rigorous causation chain management
4. **Template Instantiation**: Subject generation for template-based workflows
5. **Security Boundaries**: Subject-based access control and isolation
6. **Performance Optimization**: Minimal subject matching through lattice properties

The mathematical foundation ensures that subject routing, event correlation, and cross-domain coordination operate with mathematical precision and predictable behavior across the distributed CIM ecosystem.