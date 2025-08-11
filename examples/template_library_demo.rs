//! Template Library Demonstration
//!
//! Shows how to use the standard template library to create workflows
//! from pre-built templates across different domains.

use cim_domain_workflow::composition::{
    StandardTemplateLibrary, TemplateLibraryService, WorkflowTemplate, 
    TemplateId, TemplateVersion,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📚 Template Library Demonstration");
    println!("==================================");
    
    // Create template library service
    println!("🔧 Initializing template library service...");
    let service = TemplateLibraryService::new();
    
    // Example 1: Browse available templates
    println!("\n📋 Example 1: Browse available templates");
    let all_templates = service.list_all_templates();
    println!("✅ Found {} templates in library:", all_templates.len());
    
    // Group by categories
    let mut categories: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for template in &all_templates {
        *categories.entry(template.metadata.category.clone()).or_insert(0) += 1;
    }
    
    for (category, count) in categories {
        println!("   📁 {}: {} templates", category, count);
    }
    
    // Example 2: Get specific approval templates
    println!("\n📋 Example 2: Approval workflow templates");
    let approval_templates = service.get_templates_by_category("Approval Workflows");
    
    for template in approval_templates {
        println!("   📄 {} ({})", template.name, template.id.name);
        println!("      Description: {}", template.description);
        println!("      Domains: {:?}", template.target_domains);
        println!("      Parameters: {}", template.parameters.len());
        println!();
    }
    
    // Example 3: Demonstrate single approval template usage
    println!("📋 Example 3: Single approval template details");
    if let Some(single_approval) = service.get_template(
        "approval", 
        "single-approval", 
        &TemplateVersion::new(1, 0, 0)
    ) {
        println!("   📄 Template: {}", single_approval.name);
        println!("   📝 Description: {}", single_approval.description);
        println!("   🏷️  Version: {}.{}.{}", 
                single_approval.version.major, 
                single_approval.version.minor, 
                single_approval.version.patch);
        
        println!("   📋 Steps:");
        for (i, step) in single_approval.steps.iter().enumerate() {
            println!("      {}. {} ({:?})", i + 1, step.name_template, step.step_type);
            println!("         {}", step.description_template);
        }
        
        println!("   ⚙️  Parameters:");
        for (name, param) in &single_approval.parameters {
            let required = if param.required { "required" } else { "optional" };
            println!("      • {} ({:?}, {}): {}", 
                    name, param.param_type, required, param.description);
        }
        
        println!("   💡 Examples:");
        for example in &single_approval.metadata.examples {
            println!("      • {}: {}", example.name, example.description);
            println!("        Parameters: {:?}", example.parameters);
            println!("        Expected: {}", example.expected_outcome);
        }
    } else {
        println!("   ❌ Single approval template not found");
    }
    
    // Example 4: Demonstrate multi-level approval template
    println!("\n📋 Example 4: Multi-level approval template details");
    if let Some(multi_approval) = service.get_template(
        "approval", 
        "multi-level-approval", 
        &TemplateVersion::new(1, 0, 0)
    ) {
        println!("   📄 Template: {}", multi_approval.name);
        println!("   📝 Description: {}", multi_approval.description);
        
        println!("   📋 Workflow Steps:");
        for (i, step) in multi_approval.steps.iter().enumerate() {
            println!("      {}. {} ({:?})", i + 1, step.name_template, step.step_type);
            println!("         Dependencies: {:?}", step.dependencies);
        }
        
        println!("   💡 Use Case Example:");
        if let Some(example) = multi_approval.metadata.examples.first() {
            println!("      Scenario: {}", example.description);
            if let Some(levels) = example.parameters.get("approval_levels") {
                println!("      Approval Chain: {}", serde_json::to_string_pretty(levels)?);
            }
        }
    } else {
        println!("   ❌ Multi-level approval template not found");
    }
    
    // Example 5: Show peer review template
    println!("\n📋 Example 5: Review workflow templates");
    let review_templates = service.get_templates_by_category("Review Workflows");
    
    for template in review_templates {
        println!("   📄 {} (v{}.{})", 
                template.name, 
                template.version.major, 
                template.version.minor);
        println!("      For domains: {:?}", template.target_domains);
        println!("      Steps: {}", template.steps.len());
        
        // Show key configuration options
        for step in &template.steps {
            if !step.configuration.is_empty() {
                println!("      Configuration for '{}': {:?}", step.name_template, step.configuration);
            }
        }
        println!();
    }
    
    // Example 6: Show user management templates  
    println!("📋 Example 6: User management templates");
    let user_templates = service.get_templates_by_category("User Management");
    
    for template in user_templates {
        println!("   📄 {} ({})", template.name, template.id.name);
        println!("      Target domains: {:?}", template.target_domains);
        println!("      Author: {}", template.metadata.author);
        
        if let Some(doc_url) = &template.metadata.documentation_url {
            println!("      Documentation: {}", doc_url);
        }
        println!();
    }
    
    // Example 7: Template library statistics
    println!("📋 Example 7: Template library statistics");
    let library = StandardTemplateLibrary::new();
    let all_lib_templates = library.list_templates();
    
    println!("   📊 Library Statistics:");
    println!("      • Total templates: {}", all_lib_templates.len());
    
    // Count by domain
    let mut domain_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for template in &all_lib_templates {
        for domain in &template.target_domains {
            *domain_counts.entry(domain.clone()).or_insert(0) += 1;
        }
    }
    
    println!("      • Templates by domain:");
    for (domain, count) in domain_counts {
        println!("        - {}: {} templates", domain, count);
    }
    
    // Count by step type
    let mut step_type_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for template in &all_lib_templates {
        for step in &template.steps {
            let step_type = format!("{:?}", step.step_type);
            *step_type_counts.entry(step_type).or_insert(0) += 1;
        }
    }
    
    println!("      • Steps by type:");
    for (step_type, count) in step_type_counts {
        println!("        - {}: {} steps", step_type, count);
    }
    
    // Example 8: Template customization possibilities
    println!("\n📋 Example 8: Template customization capabilities");
    if let Some(conditional_approval) = service.get_template(
        "approval", 
        "conditional-approval", 
        &TemplateVersion::new(1, 0, 0)
    ) {
        println!("   📄 Conditional Approval Template Flexibility:");
        println!("      This template demonstrates dynamic workflow routing");
        println!("      based on runtime conditions and context data.");
        println!();
        
        if let Some(example) = conditional_approval.metadata.examples.first() {
            println!("      Example Routing Rules:");
            if let Some(rules) = example.parameters.get("approval_rules") {
                println!("{}", serde_json::to_string_pretty(rules)?);
            }
            println!();
            
            println!("      Context-Aware Processing:");
            if let Some(context) = example.parameters.get("context_data") {
                println!("{}", serde_json::to_string_pretty(context)?);
            }
        }
    }
    
    println!("\n🎯 Template Library Demonstration Complete!");
    println!("The standard library provides:");
    println!("• Pre-built workflow templates for common patterns");
    println!("• Domain-specific workflows (approval, review, user mgmt)");  
    println!("• Flexible parameterization for customization");
    println!("• Rich metadata including examples and documentation");
    println!("• Type-safe template instantiation and validation");
    
    Ok(())
}