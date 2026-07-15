//! Enterprise platform activation and audience sync

use crate::Result;
use std::collections::HashMap;

/// Supported activation platforms
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Platform {
    Braze,
    Iterable,
    Klaviyo,
    Segment,
    Salesforce,
    HubSpot,
    RudderStack,
    Custom,
}

impl Platform {
    pub fn as_str(&self) -> &str {
        match self {
            Platform::Braze => "braze",
            Platform::Iterable => "iterable",
            Platform::Klaviyo => "klaviyo",
            Platform::Segment => "segment",
            Platform::Salesforce => "salesforce",
            Platform::HubSpot => "hubspot",
            Platform::RudderStack => "rudderstack",
            Platform::Custom => "custom",
        }
    }
}

/// Activation event types
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ActivationEvent {
    SegmentAssignment,
    SegmentRemoval,
    PropertyUpdate,
    ListAdd,
    ListRemove,
}

/// Customer record for activation
#[derive(Clone, Debug)]
pub struct CustomerRecord {
    pub customer_id: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub external_id: Option<String>,
    pub attributes: HashMap<String, String>,
    pub segment: Option<String>,
}

/// Activation message to send to platform
#[derive(Clone, Debug)]
pub struct ActivationMessage {
    pub customer: CustomerRecord,
    pub event_type: ActivationEvent,
    pub platform: Platform,
    pub timestamp: i64,
    pub properties: HashMap<String, String>,
}

impl ActivationMessage {
    pub fn new(customer: CustomerRecord, event_type: ActivationEvent, platform: Platform) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        Self {
            customer,
            event_type,
            platform,
            timestamp: now,
            properties: HashMap::new(),
        }
    }

    pub fn with_property(mut self, key: String, value: String) -> Self {
        self.properties.insert(key, value);
        self
    }
}

/// Platform credential configuration
#[derive(Clone, Debug)]
pub struct PlatformCredential {
    pub platform: Platform,
    pub api_key: String,
    pub api_secret: Option<String>,
    pub endpoint: Option<String>,
    pub workspace_id: Option<String>,
}

impl PlatformCredential {
    pub fn new(platform: Platform, api_key: String) -> Self {
        Self {
            platform,
            api_key,
            api_secret: None,
            endpoint: None,
            workspace_id: None,
        }
    }

    pub fn with_secret(mut self, secret: String) -> Self {
        self.api_secret = Some(secret);
        self
    }

    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = Some(endpoint);
        self
    }

    pub fn with_workspace(mut self, workspace_id: String) -> Self {
        self.workspace_id = Some(workspace_id);
        self
    }
}

/// Activation result
#[derive(Clone, Debug)]
pub struct ActivationResult {
    pub message_id: String,
    pub platform: Platform,
    pub customer_id: String,
    pub success: bool,
    pub error: Option<String>,
    pub timestamp: i64,
}

impl ActivationResult {
    pub fn success(message_id: String, platform: Platform, customer_id: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        Self {
            message_id,
            platform,
            customer_id,
            success: true,
            error: None,
            timestamp: now,
        }
    }

    pub fn failure(
        message_id: String,
        platform: Platform,
        customer_id: String,
        error: String,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        Self {
            message_id,
            platform,
            customer_id,
            success: false,
            error: Some(error),
            timestamp: now,
        }
    }
}

/// Batch activation settings
#[derive(Clone, Debug)]
pub struct BatchActivationConfig {
    pub batch_size: usize,
    pub max_retries: usize,
    pub retry_delay_ms: u64,
    pub timeout_ms: u64,
}

impl Default for BatchActivationConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            max_retries: 3,
            retry_delay_ms: 1000,
            timeout_ms: 30000,
        }
    }
}

/// Webhook trigger configuration
#[derive(Clone, Debug)]
pub struct WebhookTrigger {
    pub url: String,
    pub event_type: ActivationEvent,
    pub headers: HashMap<String, String>,
    pub active: bool,
}

impl WebhookTrigger {
    pub fn new(url: String, event_type: ActivationEvent) -> Self {
        Self {
            url,
            event_type,
            headers: HashMap::new(),
            active: true,
        }
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
}

/// Activation statistics
#[derive(Clone, Debug)]
pub struct ActivationStats {
    pub total_messages: usize,
    pub successful: usize,
    pub failed: usize,
    pub retried: usize,
    pub skipped: usize,
    pub avg_latency_ms: f64,
}

impl Default for ActivationStats {
    fn default() -> Self {
        Self {
            total_messages: 0,
            successful: 0,
            failed: 0,
            retried: 0,
            skipped: 0,
            avg_latency_ms: 0.0,
        }
    }
}

/// Core activation engine
pub struct ActivationEngine;

impl ActivationEngine {
    /// Format customer record for platform-specific requirements
    pub fn format_for_platform(
        customer: &CustomerRecord,
        platform: &Platform,
    ) -> Result<String> {
        let mut formatted = format!("customer_id:{}", customer.customer_id);

        match platform {
            Platform::Braze => {
                if let Some(email) = &customer.email {
                    formatted.push_str(&format!("|email:{}", email));
                }
                if let Some(external_id) = &customer.external_id {
                    formatted.push_str(&format!("|external_id:{}", external_id));
                }
                if let Some(segment) = &customer.segment {
                    formatted.push_str(&format!("|segment:{}", segment));
                }
            }
            Platform::Iterable => {
                if let Some(email) = &customer.email {
                    formatted.push_str(&format!("|email:{}", email));
                }
                for (key, value) in &customer.attributes {
                    formatted.push_str(&format!("|{}:{}", key, value));
                }
            }
            Platform::Klaviyo => {
                if let Some(email) = &customer.email {
                    formatted.push_str(&format!("|email:{}", email));
                }
                if let Some(phone) = &customer.phone {
                    formatted.push_str(&format!("|phone:{}", phone));
                }
                if let Some(segment) = &customer.segment {
                    formatted.push_str(&format!("|list:{}", segment));
                }
            }
            Platform::Salesforce => {
                formatted.push_str("|object:Contact");
                if let Some(email) = &customer.email {
                    formatted.push_str(&format!("|Email:{}", email));
                }
            }
            Platform::HubSpot => {
                formatted.push_str("|object:contact");
                if let Some(email) = &customer.email {
                    formatted.push_str(&format!("|email:{}", email));
                }
                if let Some(segment) = &customer.segment {
                    formatted.push_str(&format!("|hs_lead_status:{}", segment));
                }
            }
            Platform::Segment => {
                if let Some(email) = &customer.email {
                    formatted.push_str(&format!("|email:{}", email));
                }
                for (key, value) in &customer.attributes {
                    formatted.push_str(&format!("|traits_{}:{}", key, value));
                }
            }
            Platform::RudderStack => {
                if let Some(email) = &customer.email {
                    formatted.push_str(&format!("|email:{}", email));
                }
                if let Some(segment) = &customer.segment {
                    formatted.push_str(&format!("|anonymousId:{}", segment));
                }
            }
            Platform::Custom => {
                // Pass through as-is for custom platforms
            }
        }

        Ok(formatted)
    }

    /// Validate customer record has required fields for platform
    pub fn validate_for_platform(
        customer: &CustomerRecord,
        platform: &Platform,
    ) -> Result<bool> {
        match platform {
            Platform::Braze => Ok(customer.email.is_some() || customer.external_id.is_some()),
            Platform::Iterable => Ok(customer.email.is_some()),
            Platform::Klaviyo => Ok(customer.email.is_some()),
            Platform::Salesforce => Ok(customer.email.is_some()),
            Platform::HubSpot => Ok(customer.email.is_some()),
            Platform::Segment => Ok(true), // Any customer ID works
            Platform::RudderStack => Ok(true),
            Platform::Custom => Ok(true),
        }
    }

    /// Build activation message payload
    pub fn build_payload(msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        let mut payload = HashMap::new();

        payload.insert("customer_id".to_string(), msg.customer.customer_id.clone());
        payload.insert("event_type".to_string(), format!("{:?}", msg.event_type));
        payload.insert("platform".to_string(), msg.platform.as_str().to_string());
        payload.insert("timestamp".to_string(), msg.timestamp.to_string());

        if let Some(email) = &msg.customer.email {
            payload.insert("email".to_string(), email.clone());
        }

        if let Some(segment) = &msg.customer.segment {
            payload.insert("segment".to_string(), segment.clone());
        }

        for (key, value) in &msg.properties {
            payload.insert(key.clone(), value.clone());
        }

        Ok(payload)
    }

    /// Check if customer is eligible for activation
    pub fn is_eligible(customer: &CustomerRecord, platform: &Platform) -> Result<bool> {
        if customer.customer_id.is_empty() {
            return Ok(false);
        }

        Self::validate_for_platform(customer, platform)
    }

    /// Calculate activation statistics from results
    pub fn calculate_stats(results: &[ActivationResult]) -> Result<ActivationStats> {
        let mut stats = ActivationStats::default();

        stats.total_messages = results.len();
        stats.successful = results.iter().filter(|r| r.success).count();
        stats.failed = results.iter().filter(|r| !r.success).count();

        if stats.total_messages > 0 {
            stats.avg_latency_ms = 0.0; // Would be calculated from actual latencies
        }

        Ok(stats)
    }

    /// Get platform endpoint for activation
    pub fn get_platform_endpoint(platform: &Platform) -> String {
        match platform {
            Platform::Braze => "https://rest.iad-01.braze.com/users/track".to_string(),
            Platform::Iterable => "https://api.iterable.com/api/users/bulkUpdate".to_string(),
            Platform::Klaviyo => "https://a.klaviyo.com/api/v1/person".to_string(),
            Platform::Segment => "https://api.segment.com/v1/batch".to_string(),
            Platform::Salesforce => "https://instance.salesforce.com/services/data/v57.0/sobjects/Contact".to_string(),
            Platform::HubSpot => "https://api.hubapi.com/crm/v3/objects/contacts".to_string(),
            Platform::RudderStack => "https://api.rudderstack.com/v1/identify".to_string(),
            Platform::Custom => "https://example.com/activate".to_string(),
        }
    }

    /// Deduplicate activation messages by customer and event
    pub fn deduplicate_messages(messages: &[ActivationMessage]) -> Result<Vec<ActivationMessage>> {
        let mut seen = std::collections::HashSet::new();
        let mut deduplicated = Vec::new();

        for msg in messages {
            let key = format!(
                "{}:{}:{:?}",
                msg.customer.customer_id, msg.platform.as_str(), msg.event_type
            );

            if !seen.contains(&key) {
                seen.insert(key);
                deduplicated.push(msg.clone());
            }
        }

        Ok(deduplicated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_customer() -> CustomerRecord {
        CustomerRecord {
            customer_id: "cust_123".to_string(),
            email: Some("test@example.com".to_string()),
            phone: Some("+1234567890".to_string()),
            external_id: Some("ext_456".to_string()),
            attributes: {
                let mut m = HashMap::new();
                m.insert("tier".to_string(), "gold".to_string());
                m
            },
            segment: Some("Champions".to_string()),
        }
    }

    #[test]
    fn test_platform_credential_creation() {
        let cred = PlatformCredential::new(Platform::Braze, "api_key_123".to_string())
            .with_secret("secret_456".to_string())
            .with_endpoint("https://api.braze.com".to_string());

        assert_eq!(cred.platform, Platform::Braze);
        assert_eq!(cred.api_key, "api_key_123");
        assert_eq!(cred.api_secret, Some("secret_456".to_string()));
    }

    #[test]
    fn test_activation_message_creation() {
        let customer = create_test_customer();
        let msg = ActivationMessage::new(
            customer,
            ActivationEvent::SegmentAssignment,
            Platform::Braze,
        )
        .with_property("action".to_string(), "assign".to_string());

        assert_eq!(msg.customer.customer_id, "cust_123");
        assert_eq!(msg.event_type, ActivationEvent::SegmentAssignment);
        assert_eq!(msg.platform, Platform::Braze);
        assert!(msg.properties.contains_key("action"));
    }

    #[test]
    fn test_format_for_braze() {
        let customer = create_test_customer();
        let formatted = ActivationEngine::format_for_platform(&customer, &Platform::Braze).unwrap();

        assert!(formatted.contains("customer_id:cust_123"));
        assert!(formatted.contains("email:test@example.com"));
        assert!(formatted.contains("segment:Champions"));
    }

    #[test]
    fn test_format_for_klaviyo() {
        let customer = create_test_customer();
        let formatted = ActivationEngine::format_for_platform(&customer, &Platform::Klaviyo).unwrap();

        assert!(formatted.contains("customer_id:cust_123"));
        assert!(formatted.contains("email:test@example.com"));
        assert!(formatted.contains("phone:+1234567890"));
        assert!(formatted.contains("list:Champions"));
    }

    #[test]
    fn test_validate_for_platform() {
        let customer = create_test_customer();

        assert!(ActivationEngine::validate_for_platform(&customer, &Platform::Braze).unwrap());
        assert!(ActivationEngine::validate_for_platform(&customer, &Platform::Iterable).unwrap());
        assert!(ActivationEngine::validate_for_platform(&customer, &Platform::Klaviyo).unwrap());
    }

    #[test]
    fn test_is_eligible() {
        let customer = create_test_customer();

        assert!(ActivationEngine::is_eligible(&customer, &Platform::Braze).unwrap());

        let invalid = CustomerRecord {
            customer_id: "".to_string(),
            email: None,
            phone: None,
            external_id: None,
            attributes: HashMap::new(),
            segment: None,
        };

        assert!(!ActivationEngine::is_eligible(&invalid, &Platform::Braze).unwrap());
    }

    #[test]
    fn test_activation_result_success() {
        let result = ActivationResult::success(
            "msg_123".to_string(),
            Platform::Braze,
            "cust_123".to_string(),
        );

        assert!(result.success);
        assert_eq!(result.platform, Platform::Braze);
        assert_eq!(result.error, None);
    }

    #[test]
    fn test_activation_result_failure() {
        let result = ActivationResult::failure(
            "msg_123".to_string(),
            Platform::Braze,
            "cust_123".to_string(),
            "API error".to_string(),
        );

        assert!(!result.success);
        assert_eq!(result.error, Some("API error".to_string()));
    }

    #[test]
    fn test_build_payload() {
        let customer = create_test_customer();
        let msg = ActivationMessage::new(
            customer,
            ActivationEvent::SegmentAssignment,
            Platform::Braze,
        );

        let payload = ActivationEngine::build_payload(&msg).unwrap();

        assert_eq!(payload.get("customer_id"), Some(&"cust_123".to_string()));
        assert_eq!(payload.get("email"), Some(&"test@example.com".to_string()));
    }

    #[test]
    fn test_deduplicate_messages() {
        let mut customer1 = create_test_customer();
        let mut customer2 = create_test_customer();
        customer2.customer_id = "cust_456".to_string();

        let msg1 = ActivationMessage::new(
            customer1,
            ActivationEvent::SegmentAssignment,
            Platform::Braze,
        );
        let msg2 = ActivationMessage::new(
            customer2,
            ActivationEvent::SegmentAssignment,
            Platform::Braze,
        );

        let messages = vec![msg1.clone(), msg2.clone(), msg1];
        let dedup = ActivationEngine::deduplicate_messages(&messages).unwrap();

        assert_eq!(dedup.len(), 2);
    }

    #[test]
    fn test_webhook_trigger_creation() {
        let webhook =
            WebhookTrigger::new("https://example.com/webhook".to_string(), ActivationEvent::SegmentAssignment)
                .with_header("Authorization".to_string(), "Bearer token".to_string());

        assert_eq!(webhook.url, "https://example.com/webhook");
        assert!(webhook.active);
        assert!(webhook.headers.contains_key("Authorization"));
    }

    #[test]
    fn test_batch_config_defaults() {
        let config = BatchActivationConfig::default();

        assert_eq!(config.batch_size, 1000);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout_ms, 30000);
    }

    #[test]
    fn test_calculate_stats() {
        let results = vec![
            ActivationResult::success("msg_1".to_string(), Platform::Braze, "cust_1".to_string()),
            ActivationResult::success("msg_2".to_string(), Platform::Braze, "cust_2".to_string()),
            ActivationResult::failure(
                "msg_3".to_string(),
                Platform::Braze,
                "cust_3".to_string(),
                "error".to_string(),
            ),
        ];

        let stats = ActivationEngine::calculate_stats(&results).unwrap();

        assert_eq!(stats.total_messages, 3);
        assert_eq!(stats.successful, 2);
        assert_eq!(stats.failed, 1);
    }
}
