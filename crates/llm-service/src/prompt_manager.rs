use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    pub template: String,
    pub variables: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PromptManager {
    templates: HashMap<String, PromptTemplate>,
}

impl PromptManager {
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };

        // Add default templates
        manager.add_default_templates();
        manager
    }

    /// Add a prompt template
    pub fn add_template(&mut self, template: PromptTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Get a prompt template by name
    pub fn get_template(&self, name: &str) -> Option<&PromptTemplate> {
        self.templates.get(name)
    }

    /// Render a template with variables
    pub fn render_template(
        &self,
        name: &str,
        variables: &HashMap<String, String>,
    ) -> Option<String> {
        let template = self.templates.get(name)?;

        let mut rendered = template.template.clone();

        // Replace variables in the format {{variable_name}}
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            rendered = rendered.replace(&placeholder, value);
        }

        Some(rendered)
    }

    /// Get all template names
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Remove a template
    pub fn remove_template(&mut self, name: &str) -> Option<PromptTemplate> {
        self.templates.remove(name)
    }

    /// Check if template exists
    pub fn has_template(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }

    /// Validate template variables
    pub fn validate_template_variables(
        &self,
        name: &str,
        variables: &HashMap<String, String>,
    ) -> Result<(), Vec<String>> {
        let template = match self.templates.get(name) {
            Some(t) => t,
            None => return Err(vec![format!("Template '{}' not found", name)]),
        };

        let mut missing_vars = Vec::new();

        for required_var in &template.variables {
            if !variables.contains_key(required_var) {
                missing_vars.push(required_var.clone());
            }
        }

        if missing_vars.is_empty() {
            Ok(())
        } else {
            Err(missing_vars)
        }
    }

    /// Add default system prompt templates
    fn add_default_templates(&mut self) {
        // General assistant template
        self.add_template(PromptTemplate {
            name: "assistant".to_string(),
            template: "You are a helpful AI assistant. {{context}}User: {{user_input}}".to_string(),
            variables: vec!["context".to_string(), "user_input".to_string()],
            description: Some("General purpose assistant prompt".to_string()),
        });

        // Schedule management template
        self.add_template(PromptTemplate {
            name: "schedule_assistant".to_string(),
            template: "You are an AI assistant specialized in schedule and calendar management. Help the user with their scheduling needs.\n\nCurrent time: {{current_time}}\nUser request: {{user_input}}\n\nPlease provide helpful scheduling assistance.".to_string(),
            variables: vec!["current_time".to_string(), "user_input".to_string()],
            description: Some("Schedule and calendar management assistant".to_string()),
        });

        // Email management template
        self.add_template(PromptTemplate {
            name: "email_assistant".to_string(),
            template: "You are an AI assistant that helps with email management and composition.\n\nEmail context: {{email_context}}\nUser request: {{user_input}}\n\nHelp the user with their email-related task.".to_string(),
            variables: vec!["email_context".to_string(), "user_input".to_string()],
            description: Some("Email management and composition assistant".to_string()),
        });

        // Summarization template
        self.add_template(PromptTemplate {
            name: "summarize".to_string(),
            template: "Please provide a concise summary of the following content:\n\n{{content}}\n\nSummary:".to_string(),
            variables: vec!["content".to_string()],
            description: Some("Content summarization prompt".to_string()),
        });

        // Question answering template
        self.add_template(PromptTemplate {
            name: "qa".to_string(),
            template: "Based on the following context, please answer the question.\n\nContext: {{context}}\n\nQuestion: {{question}}\n\nAnswer:".to_string(),
            variables: vec!["context".to_string(), "question".to_string()],
            description: Some("Question answering with context".to_string()),
        });

        // System error template
        self.add_template(PromptTemplate {
            name: "system_error".to_string(),
            template: "I encountered an error while processing your request: {{error_message}}\n\nPlease try rephrasing your request or contact support if the issue persists.".to_string(),
            variables: vec!["error_message".to_string()],
            description: Some("System error response template".to_string()),
        });
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_rendering() {
        let manager = PromptManager::new();

        let mut variables = HashMap::new();
        variables.insert(
            "context".to_string(),
            "The user is asking about weather. ".to_string(),
        );
        variables.insert(
            "user_input".to_string(),
            "What's the weather like?".to_string(),
        );

        let rendered = manager.render_template("assistant", &variables).unwrap();
        assert!(rendered.contains("weather"));
        assert!(rendered.contains("What's the weather like?"));
    }

    #[test]
    fn test_template_validation() {
        let manager = PromptManager::new();

        let mut complete_vars = HashMap::new();
        complete_vars.insert("context".to_string(), "test".to_string());
        complete_vars.insert("user_input".to_string(), "test".to_string());

        let result = manager.validate_template_variables("assistant", &complete_vars);
        assert!(result.is_ok());

        let mut incomplete_vars = HashMap::new();
        incomplete_vars.insert("context".to_string(), "test".to_string());

        let result = manager.validate_template_variables("assistant", &incomplete_vars);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert!(missing.contains(&"user_input".to_string()));
    }

    #[test]
    fn test_custom_template() {
        let mut manager = PromptManager::new();

        let custom_template = PromptTemplate {
            name: "custom".to_string(),
            template: "Hello {{name}}, welcome to {{app}}!".to_string(),
            variables: vec!["name".to_string(), "app".to_string()],
            description: Some("Custom greeting".to_string()),
        };

        manager.add_template(custom_template);

        let mut variables = HashMap::new();
        variables.insert("name".to_string(), "Alice".to_string());
        variables.insert("app".to_string(), "AI Manager".to_string());

        let rendered = manager.render_template("custom", &variables).unwrap();
        assert_eq!(rendered, "Hello Alice, welcome to AI Manager!");
    }

    #[test]
    fn test_template_management() {
        let mut manager = PromptManager::new();

        // Should have default templates
        assert!(manager.has_template("assistant"));
        assert!(!manager.list_templates().is_empty());

        // Test removing template
        let removed = manager.remove_template("assistant");
        assert!(removed.is_some());
        assert!(!manager.has_template("assistant"));
    }
}
