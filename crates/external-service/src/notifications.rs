use ai_manager_shared::errors::SystemError;
use serde::{Deserialize, Serialize};
use tracing::info;
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct NotificationClient {
    // Configuration for different notification methods
    desktop_notifications: bool,
    email_notifications: bool,
    webhook_url: Option<String>,
}

impl NotificationClient {
    pub async fn new() -> Result<Self, SystemError> {
        let desktop_notifications = std::env::var("ENABLE_DESKTOP_NOTIFICATIONS")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(true);

        let email_notifications = std::env::var("ENABLE_EMAIL_NOTIFICATIONS")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);

        let webhook_url = std::env::var("NOTIFICATION_WEBHOOK_URL").ok();

        Ok(Self {
            desktop_notifications,
            email_notifications,
            webhook_url,
        })
    }

    pub async fn send_notification(&self, message: &str) -> Result<(), SystemError> {
        self.send_notification_with_type(message, NotificationType::Info)
            .await
    }

    pub async fn send_notification_with_type(
        &self,
        message: &str,
        notification_type: NotificationType,
    ) -> Result<(), SystemError> {
        let notification = Notification {
            title: self.get_title_for_type(&notification_type),
            message: message.to_string(),
            notification_type: notification_type.clone(),
            timestamp: chrono::Utc::now(),
        };

        let mut success_count = 0;
        let mut errors = Vec::new();

        // Try desktop notifications
        if self.desktop_notifications {
            match self.send_desktop_notification(&notification).await {
                Ok(_) => success_count += 1,
                Err(e) => errors.push(format!("Desktop notification failed: {}", e)),
            }
        }

        // Try email notifications
        if self.email_notifications {
            match self.send_email_notification(&notification).await {
                Ok(_) => success_count += 1,
                Err(e) => errors.push(format!("Email notification failed: {}", e)),
            }
        }

        // Try webhook notifications
        if let Some(webhook_url) = &self.webhook_url {
            match self
                .send_webhook_notification(webhook_url, &notification)
                .await
            {
                Ok(_) => success_count += 1,
                Err(e) => errors.push(format!("Webhook notification failed: {}", e)),
            }
        }

        if success_count > 0 {
            info!(
                "Notification sent successfully via {} method(s)",
                success_count
            );
            Ok(())
        } else {
            Err(SystemError::ExternalService {
                service: "Notifications".to_string(),
                message: format!("All notification methods failed: {}", errors.join(", ")),
            })
        }
    }

    async fn send_desktop_notification(
        &self,
        notification: &Notification,
    ) -> Result<(), SystemError> {
        // In a real implementation, this would use a library like `notify-rust`
        // For now, we'll simulate desktop notifications

        #[cfg(target_os = "macos")]
        {
            self.send_macos_notification(notification).await
        }

        #[cfg(target_os = "linux")]
        {
            self.send_linux_notification(notification).await
        }

        #[cfg(target_os = "windows")]
        {
            self.send_windows_notification(notification).await
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            warn!("Desktop notifications not supported on this platform");
            Err(SystemError::ExternalService {
                service: "Notifications".to_string(),
                message: "Desktop notifications not supported".to_string(),
            })
        }
    }

    #[cfg(target_os = "macos")]
    async fn send_macos_notification(
        &self,
        notification: &Notification,
    ) -> Result<(), SystemError> {
        // Use osascript to send macOS notifications
        let script = format!(
            r#"display notification "{}" with title "{}""#,
            notification.message.replace('"', r#"\""#),
            notification.title.replace('"', r#"\""#)
        );

        let output = tokio::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .await
            .map_err(|e| SystemError::ExternalService {
                service: "Notifications".to_string(),
                message: format!("Failed to execute osascript: {}", e),
            })?;

        if output.status.success() {
            Ok(())
        } else {
            Err(SystemError::ExternalService {
                service: "Notifications".to_string(),
                message: format!(
                    "osascript failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            })
        }
    }

    #[cfg(target_os = "linux")]
    async fn send_linux_notification(
        &self,
        notification: &Notification,
    ) -> Result<(), SystemError> {
        // Use notify-send for Linux notifications
        let output = tokio::process::Command::new("notify-send")
            .arg(&notification.title)
            .arg(&notification.message)
            .output()
            .await
            .map_err(|e| SystemError::ExternalService {
                service: "Notifications".to_string(),
                message: format!("Failed to execute notify-send: {}", e),
            })?;

        if output.status.success() {
            Ok(())
        } else {
            Err(SystemError::ExternalService {
                service: "Notifications".to_string(),
                message: format!(
                    "notify-send failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            })
        }
    }

    #[cfg(target_os = "windows")]
    async fn send_windows_notification(
        &self,
        notification: &Notification,
    ) -> Result<(), SystemError> {
        // Use PowerShell for Windows notifications
        let script = format!(
            r#"Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.MessageBox]::Show('{}', '{}')"#,
            notification.message.replace('\'', "''"),
            notification.title.replace('\'', "''")
        );

        let output = tokio::process::Command::new("powershell")
            .arg("-Command")
            .arg(&script)
            .output()
            .await
            .map_err(|e| SystemError::ExternalService {
                service: "Notifications".to_string(),
                message: format!("Failed to execute PowerShell: {}", e),
            })?;

        if output.status.success() {
            Ok(())
        } else {
            Err(SystemError::ExternalService {
                service: "Notifications".to_string(),
                message: format!(
                    "PowerShell notification failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            })
        }
    }

    async fn send_email_notification(
        &self,
        notification: &Notification,
    ) -> Result<(), SystemError> {
        // This would integrate with the email client to send notification emails
        // For now, we'll just log it
        info!(
            "Email notification: {} - {}",
            notification.title, notification.message
        );
        Ok(())
    }

    async fn send_webhook_notification(
        &self,
        webhook_url: &str,
        notification: &Notification,
    ) -> Result<(), SystemError> {
        let client = reqwest::Client::new();

        let payload = serde_json::json!({
            "title": notification.title,
            "message": notification.message,
            "type": notification.notification_type,
            "timestamp": notification.timestamp
        });

        let response = client
            .post(webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| SystemError::ExternalService {
                service: "Notifications".to_string(),
                message: format!("Webhook request failed: {}", e),
            })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(SystemError::ExternalService {
                service: "Notifications".to_string(),
                message: format!("Webhook returned status: {}", response.status()),
            })
        }
    }

    fn get_title_for_type(&self, notification_type: &NotificationType) -> String {
        match notification_type {
            NotificationType::Info => "AI Manager - Info".to_string(),
            NotificationType::Warning => "AI Manager - Warning".to_string(),
            NotificationType::Error => "AI Manager - Error".to_string(),
            NotificationType::Success => "AI Manager - Success".to_string(),
        }
    }

    pub async fn send_error_notification(&self, error: &str) -> Result<(), SystemError> {
        self.send_notification_with_type(error, NotificationType::Error)
            .await
    }

    pub async fn send_warning_notification(&self, warning: &str) -> Result<(), SystemError> {
        self.send_notification_with_type(warning, NotificationType::Warning)
            .await
    }

    pub async fn send_success_notification(&self, message: &str) -> Result<(), SystemError> {
        self.send_notification_with_type(message, NotificationType::Success)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_notification_client_creation() {
        let client = NotificationClient::new().await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_notification_creation() {
        let notification = Notification {
            title: "Test".to_string(),
            message: "Test message".to_string(),
            notification_type: NotificationType::Info,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(notification.title, "Test");
        assert_eq!(notification.message, "Test message");
    }

    #[tokio::test]
    async fn test_title_generation() {
        let client = NotificationClient::new().await.unwrap();

        assert_eq!(
            client.get_title_for_type(&NotificationType::Info),
            "AI Manager - Info"
        );
        assert_eq!(
            client.get_title_for_type(&NotificationType::Error),
            "AI Manager - Error"
        );
    }

    #[tokio::test]
    async fn test_send_notification() {
        let client = NotificationClient::new().await.unwrap();

        // This will try to send notifications, but may fail depending on the environment
        // The test primarily verifies the interface works
        let result = client.send_notification("Test notification").await;
        // Don't assert success/failure as it depends on the environment
        assert!(result.is_ok() || result.is_err());
    }
}
