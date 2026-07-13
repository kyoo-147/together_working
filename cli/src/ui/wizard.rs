#[derive(Debug, Clone)]
pub struct ContractWizard {
    fields: Vec<WizardField>,
    current: usize,
    status: String,
}

#[derive(Debug, Clone)]
pub struct WizardField {
    pub label: &'static str,
    pub title: &'static str,
    pub placeholder: &'static str,
    pub required: bool,
    pub value: String,
}

impl ContractWizard {
    pub fn new() -> Self {
        Self {
            fields: vec![
                WizardField {
                    label: "title",
                    title: "Title",
                    placeholder: "Short operator-facing task title",
                    required: true,
                    value: "Fix scoped task".to_string(),
                },
                WizardField {
                    label: "department",
                    title: "Department",
                    placeholder: "engineering, integration, review",
                    required: true,
                    value: "engineering".to_string(),
                },
                WizardField {
                    label: "scope",
                    title: "Scope",
                    placeholder: "src/**",
                    required: true,
                    value: "src/**".to_string(),
                },
                WizardField {
                    label: "allowed_files",
                    title: "Allowed files",
                    placeholder: "src/**,cli/src/**",
                    required: true,
                    value: "src/**".to_string(),
                },
                WizardField {
                    label: "denied_files",
                    title: "Denied files",
                    placeholder: ".env,**/secrets/*",
                    required: false,
                    value: ".env,**/secrets/*".to_string(),
                },
                WizardField {
                    label: "success_criteria",
                    title: "Success criteria",
                    placeholder: "tests pass, UI renders cleanly",
                    required: true,
                    value: "task completes".to_string(),
                },
            ],
            current: 0,
            status: "Enter next field | Ctrl+Enter dispatch | Esc cancel".to_string(),
        }
    }

    pub fn fields(&self) -> &[WizardField] {
        &self.fields
    }

    pub fn current(&self) -> usize {
        self.current
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn push_char(&mut self, ch: char) {
        self.fields[self.current].value.push(ch);
    }

    pub fn backspace(&mut self) {
        self.fields[self.current].value.pop();
    }

    pub fn next(&mut self) {
        self.current = (self.current + 1).min(self.fields.len() - 1);
    }

    pub fn previous(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        }
    }

    pub fn set_value(&mut self, label: &str, value: &str) {
        if let Some(field) = self.fields.iter_mut().find(|field| field.label == label) {
            field.value = value.to_string();
        }
    }

    pub fn value(&self, label: &str) -> &str {
        self.fields
            .iter()
            .find(|field| field.label == label)
            .map(|field| field.value.as_str())
            .unwrap_or("")
    }

    pub fn validation_errors(&self) -> Vec<String> {
        self.fields
            .iter()
            .filter(|field| field.required && field.value.trim().is_empty())
            .map(|field| format!("{} is required", field.label))
            .collect()
    }

    pub fn is_valid(&self) -> bool {
        self.validation_errors().is_empty()
    }

    pub fn yaml(&self) -> String {
        let title = self.value("title");
        let department = self.value("department");
        format!(
            "task_id: draft\ntitle: {}\ndepartment: {}\nscope:\n{}allowed_files:\n{}denied_files:\n{}success_criteria:\n{}reviewer_required: true\nverification_required: true\nmerge_authority: codex\nenforcement_mode: strict\nunknown_files_policy: needs_review\n",
            yaml_scalar(title),
            yaml_scalar(department),
            yaml_list(self.value("scope")),
            yaml_list(self.value("allowed_files")),
            yaml_list(self.value("denied_files")),
            yaml_list(self.value("success_criteria")),
        )
    }
}

impl Default for ContractWizard {
    fn default() -> Self {
        Self::new()
    }
}

fn yaml_scalar(value: &str) -> String {
    format!("{:?}", value)
}

fn yaml_list(value: &str) -> String {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| format!("  - {:?}\n", item))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    #[test]
    fn wizard_reports_missing_required_scope_and_success_criteria() {
        let mut wizard = super::ContractWizard::new();
        wizard.set_value("scope", "");
        wizard.set_value("success_criteria", "");

        let errors = wizard.validation_errors();

        assert!(errors.iter().any(|error| error.contains("scope")));
        assert!(errors
            .iter()
            .any(|error| error.contains("success_criteria")));
    }

    #[test]
    fn wizard_generates_canonical_contract_yaml() {
        let mut wizard = super::ContractWizard::new();
        wizard.set_value("title", "Improve cockpit");
        wizard.set_value("allowed_files", "cli/src/**,core/src/events.rs");

        let yaml = wizard.yaml();

        assert!(yaml.contains("title: \"Improve cockpit\""));
        assert!(yaml.contains("allowed_files:"));
        assert!(yaml.contains("- \"cli/src/**\""));
        assert!(yaml.contains("enforcement_mode: strict"));
    }
}
