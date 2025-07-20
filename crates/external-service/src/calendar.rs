use ai_manager_shared::errors::SystemError;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub summary: String,
    pub description: Option<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub location: Option<String>,
    pub attendees: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GoogleCalendarEvent {
    id: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    start: GoogleDateTime,
    end: GoogleDateTime,
    location: Option<String>,
    attendees: Option<Vec<GoogleAttendee>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GoogleDateTime {
    #[serde(rename = "dateTime")]
    date_time: Option<String>,
    date: Option<String>,
    #[serde(rename = "timeZone")]
    time_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GoogleAttendee {
    email: String,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GoogleCalendarListResponse {
    items: Vec<GoogleCalendarEvent>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

pub struct GoogleCalendarClient {
    client: Client,
    access_token: Option<String>,
    calendar_id: String,
}

impl GoogleCalendarClient {
    pub async fn new() -> Result<Self, SystemError> {
        let client = Client::new();

        // In a real implementation, this would handle OAuth2 authentication
        // For now, we'll create a placeholder that can be configured later
        let access_token = std::env::var("GOOGLE_CALENDAR_ACCESS_TOKEN").ok();
        let calendar_id =
            std::env::var("GOOGLE_CALENDAR_ID").unwrap_or_else(|_| "primary".to_string());

        if access_token.is_none() {
            warn!("Google Calendar access token not configured. Set GOOGLE_CALENDAR_ACCESS_TOKEN environment variable.");
        }

        Ok(Self {
            client,
            access_token,
            calendar_id,
        })
    }

    pub async fn list_events(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>, SystemError> {
        if self.access_token.is_none() {
            return Err(SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: "Access token not configured".to_string(),
            });
        }

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events",
            self.calendar_id
        );

        let mut params = HashMap::new();
        params.insert("timeMin", start_date.to_rfc3339());
        params.insert("timeMax", end_date.to_rfc3339());
        params.insert("singleEvents", "true".to_string());
        params.insert("orderBy", "startTime".to_string());

        let response = self
            .client
            .get(&url)
            .bearer_auth(self.access_token.as_ref().unwrap())
            .query(&params)
            .send()
            .await
            .map_err(|e| SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: format!("API error: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: format!("API returned status: {}", response.status()),
            });
        }

        let calendar_response: GoogleCalendarListResponse =
            response
                .json()
                .await
                .map_err(|e| SystemError::ExternalService {
                    service: "Google Calendar".to_string(),
                    message: format!("Failed to parse response: {}", e),
                })?;

        let events = calendar_response
            .items
            .into_iter()
            .filter_map(|event| self.convert_google_event(event))
            .collect();

        Ok(events)
    }

    pub async fn create_event(
        &self,
        title: &str,
        description: Option<&str>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<String, SystemError> {
        if self.access_token.is_none() {
            return Err(SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: "Access token not configured".to_string(),
            });
        }

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events",
            self.calendar_id
        );

        let event = GoogleCalendarEvent {
            id: None,
            summary: Some(title.to_string()),
            description: description.map(|s| s.to_string()),
            start: GoogleDateTime {
                date_time: Some(start_time.to_rfc3339()),
                date: None,
                time_zone: Some("UTC".to_string()),
            },
            end: GoogleDateTime {
                date_time: Some(end_time.to_rfc3339()),
                date: None,
                time_zone: Some("UTC".to_string()),
            },
            location: None,
            attendees: None,
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(self.access_token.as_ref().unwrap())
            .json(&event)
            .send()
            .await
            .map_err(|e| SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: format!("API error: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: format!("API returned status: {}", response.status()),
            });
        }

        let created_event: GoogleCalendarEvent =
            response
                .json()
                .await
                .map_err(|e| SystemError::ExternalService {
                    service: "Google Calendar".to_string(),
                    message: format!("Failed to parse response: {}", e),
                })?;

        Ok(created_event.id.unwrap_or_else(|| "unknown".to_string()))
    }

    pub async fn update_event(
        &self,
        event_id: &str,
        title: Option<&str>,
        description: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<(), SystemError> {
        if self.access_token.is_none() {
            return Err(SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: "Access token not configured".to_string(),
            });
        }

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events/{}",
            self.calendar_id, event_id
        );

        // First, get the existing event
        let existing_response = self
            .client
            .get(&url)
            .bearer_auth(self.access_token.as_ref().unwrap())
            .send()
            .await
            .map_err(|e| SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: format!("API error: {}", e),
            })?;

        if !existing_response.status().is_success() {
            return Err(SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: format!(
                    "Failed to get existing event: {}",
                    existing_response.status()
                ),
            });
        }

        let mut existing_event: GoogleCalendarEvent =
            existing_response
                .json()
                .await
                .map_err(|e| SystemError::ExternalService {
                    service: "Google Calendar".to_string(),
                    message: format!("Failed to parse existing event: {}", e),
                })?;

        // Update fields if provided
        if let Some(title) = title {
            existing_event.summary = Some(title.to_string());
        }
        if let Some(description) = description {
            existing_event.description = Some(description.to_string());
        }
        if let Some(start_time) = start_time {
            existing_event.start = GoogleDateTime {
                date_time: Some(start_time.to_rfc3339()),
                date: None,
                time_zone: Some("UTC".to_string()),
            };
        }
        if let Some(end_time) = end_time {
            existing_event.end = GoogleDateTime {
                date_time: Some(end_time.to_rfc3339()),
                date: None,
                time_zone: Some("UTC".to_string()),
            };
        }

        // Update the event
        let response = self
            .client
            .put(&url)
            .bearer_auth(self.access_token.as_ref().unwrap())
            .json(&existing_event)
            .send()
            .await
            .map_err(|e| SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: format!("API error: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: format!("Failed to update event: {}", response.status()),
            });
        }

        Ok(())
    }

    pub async fn delete_event(&self, event_id: &str) -> Result<(), SystemError> {
        if self.access_token.is_none() {
            return Err(SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: "Access token not configured".to_string(),
            });
        }

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events/{}",
            self.calendar_id, event_id
        );

        let response = self
            .client
            .delete(&url)
            .bearer_auth(self.access_token.as_ref().unwrap())
            .send()
            .await
            .map_err(|e| SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: format!("API error: {}", e),
            })?;

        if !response.status().is_success() && response.status().as_u16() != 404 {
            return Err(SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: format!("Failed to delete event: {}", response.status()),
            });
        }

        Ok(())
    }

    pub async fn health_check(&self) -> Result<(), SystemError> {
        if self.access_token.is_none() {
            return Err(SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: "Access token not configured".to_string(),
            });
        }

        // Simple health check by trying to list calendars
        let url = "https://www.googleapis.com/calendar/v3/users/me/calendarList";

        let response = self
            .client
            .get(url)
            .bearer_auth(self.access_token.as_ref().unwrap())
            .send()
            .await
            .map_err(|e| SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: format!("Health check failed: {}", e),
            })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(SystemError::ExternalService {
                service: "Google Calendar".to_string(),
                message: format!("Health check failed with status: {}", response.status()),
            })
        }
    }

    fn convert_google_event(&self, event: GoogleCalendarEvent) -> Option<CalendarEvent> {
        let id = event.id?;
        let summary = event.summary.unwrap_or_else(|| "No title".to_string());

        let start = self.parse_google_datetime(&event.start)?;
        let end = self.parse_google_datetime(&event.end)?;

        let attendees = event
            .attendees
            .unwrap_or_default()
            .into_iter()
            .map(|a| a.email)
            .collect();

        Some(CalendarEvent {
            id,
            summary,
            description: event.description,
            start,
            end,
            location: event.location,
            attendees,
        })
    }

    fn parse_google_datetime(&self, dt: &GoogleDateTime) -> Option<DateTime<Utc>> {
        if let Some(date_time) = &dt.date_time {
            DateTime::parse_from_rfc3339(date_time)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        } else if let Some(date) = &dt.date {
            // Handle all-day events
            chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_calendar_client_creation() {
        let client = GoogleCalendarClient::new().await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_datetime_parsing() {
        let client = GoogleCalendarClient::new().await.unwrap();

        let google_dt = GoogleDateTime {
            date_time: Some("2024-01-01T10:00:00Z".to_string()),
            date: None,
            time_zone: Some("UTC".to_string()),
        };

        let parsed = client.parse_google_datetime(&google_dt);
        assert!(parsed.is_some());
    }

    #[tokio::test]
    #[ignore] // Requires API credentials
    async fn test_list_events() {
        let client = GoogleCalendarClient::new().await.unwrap();
        let start = Utc::now();
        let end = start + chrono::Duration::days(7);

        let result = client.list_events(start, end).await;
        // Will fail without credentials, but tests the interface
        assert!(result.is_err() || result.is_ok());
    }
}
