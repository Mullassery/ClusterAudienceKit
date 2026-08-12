//! Platform-specific API adapters for activation

use crate::engine::activation::*;
use crate::Result;
use std::collections::HashMap;

/// Braze API adapter
pub struct BrazeAdapter {
    credential: PlatformCredential,
}

impl BrazeAdapter {
    pub fn new(credential: PlatformCredential) -> Self {
        Self { credential }
    }

    /// Format event for Braze API (track users endpoint)
    pub fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        let mut event = HashMap::new();

        event.insert("external_id".to_string(), msg.customer.customer_id.clone());

        if let Some(email) = &msg.customer.email {
            event.insert("email".to_string(), email.clone());
        }

        event.insert(
            "custom_attributes".to_string(),
            format!("{:?}", msg.customer.attributes),
        );

        if let Some(segment) = &msg.customer.segment {
            event.insert("segment".to_string(), segment.clone());
        }

        Ok(event)
    }

    pub fn get_endpoint(&self) -> String {
        self.credential
            .endpoint
            .clone()
            .unwrap_or_else(|| ActivationEngine::get_platform_endpoint(&Platform::Braze))
    }

    pub fn get_auth_header(&self) -> Result<(String, String)> {
        Ok((
            "Authorization".to_string(),
            format!("Bearer {}", self.credential.api_key),
        ))
    }
}

/// Iterable API adapter
pub struct IterableAdapter {
    credential: PlatformCredential,
}

impl IterableAdapter {
    pub fn new(credential: PlatformCredential) -> Self {
        Self { credential }
    }

    pub fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        let mut event = HashMap::new();

        event.insert(
            "email".to_string(),
            msg.customer.email.as_ref().cloned().unwrap_or_default(),
        );
        event.insert(
            "dataFields".to_string(),
            format!("{:?}", msg.customer.attributes),
        );

        if let Some(segment) = &msg.customer.segment {
            event.insert("listId".to_string(), segment.clone());
        }

        Ok(event)
    }

    pub fn get_endpoint(&self) -> String {
        self.credential
            .endpoint
            .clone()
            .unwrap_or_else(|| ActivationEngine::get_platform_endpoint(&Platform::Iterable))
    }

    pub fn get_auth_header(&self) -> Result<(String, String)> {
        Ok(("Api-Key".to_string(), self.credential.api_key.clone()))
    }
}

/// Klaviyo API adapter
pub struct KlaviyoAdapter {
    credential: PlatformCredential,
}

impl KlaviyoAdapter {
    pub fn new(credential: PlatformCredential) -> Self {
        Self { credential }
    }

    pub fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        let mut event = HashMap::new();

        event.insert(
            "email".to_string(),
            msg.customer.email.as_ref().cloned().unwrap_or_default(),
        );

        if let Some(phone) = &msg.customer.phone {
            event.insert("phone_number".to_string(), phone.clone());
        }

        event.insert(
            "properties".to_string(),
            format!("{:?}", msg.customer.attributes),
        );

        if let Some(segment) = &msg.customer.segment {
            event.insert("list_id".to_string(), segment.clone());
        }

        Ok(event)
    }

    pub fn get_endpoint(&self) -> String {
        self.credential
            .endpoint
            .clone()
            .unwrap_or_else(|| ActivationEngine::get_platform_endpoint(&Platform::Klaviyo))
    }

    pub fn get_auth_header(&self) -> Result<(String, String)> {
        Ok((
            "Authorization".to_string(),
            format!("Klaviyo-API-Key {}", self.credential.api_key),
        ))
    }
}

/// Salesforce API adapter
pub struct SalesforceAdapter {
    credential: PlatformCredential,
}

impl SalesforceAdapter {
    pub fn new(credential: PlatformCredential) -> Self {
        Self { credential }
    }

    pub fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        let mut event = HashMap::new();

        if let Some(email) = &msg.customer.email {
            event.insert("Email".to_string(), email.clone());
        }

        event.insert(
            "External_Id__c".to_string(),
            msg.customer.customer_id.clone(),
        );

        if let Some(segment) = &msg.customer.segment {
            event.insert("Segment__c".to_string(), segment.clone());
        }

        for (key, value) in &msg.customer.attributes {
            event.insert(key.clone(), value.clone());
        }

        Ok(event)
    }

    pub fn get_endpoint(&self) -> String {
        self.credential
            .endpoint
            .clone()
            .unwrap_or_else(|| ActivationEngine::get_platform_endpoint(&Platform::Salesforce))
    }

    pub fn get_auth_header(&self) -> Result<(String, String)> {
        Ok((
            "Authorization".to_string(),
            format!("Bearer {}", self.credential.api_key),
        ))
    }
}

/// HubSpot API adapter
pub struct HubSpotAdapter {
    credential: PlatformCredential,
}

impl HubSpotAdapter {
    pub fn new(credential: PlatformCredential) -> Self {
        Self { credential }
    }

    pub fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        let mut event = HashMap::new();

        if let Some(email) = &msg.customer.email {
            event.insert("email".to_string(), email.clone());
        }

        event.insert(
            "hs_lead_status".to_string(),
            msg.customer.segment.as_ref().cloned().unwrap_or_default(),
        );

        for (key, value) in &msg.customer.attributes {
            event.insert(format!("custom_{}", key), value.clone());
        }

        Ok(event)
    }

    pub fn get_endpoint(&self) -> String {
        self.credential
            .endpoint
            .clone()
            .unwrap_or_else(|| ActivationEngine::get_platform_endpoint(&Platform::HubSpot))
    }

    pub fn get_auth_header(&self) -> Result<(String, String)> {
        Ok((
            "Authorization".to_string(),
            format!("Bearer {}", self.credential.api_key),
        ))
    }
}

/// Segment API adapter
pub struct SegmentAdapter {
    credential: PlatformCredential,
}

impl SegmentAdapter {
    pub fn new(credential: PlatformCredential) -> Self {
        Self { credential }
    }

    pub fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        let mut event = HashMap::new();

        event.insert("userId".to_string(), msg.customer.customer_id.clone());

        if let Some(email) = &msg.customer.email {
            event.insert("email".to_string(), email.clone());
        }

        let mut traits = msg.customer.attributes.clone();
        if let Some(segment) = &msg.customer.segment {
            traits.insert("segment".to_string(), segment.clone());
        }

        event.insert("traits".to_string(), format!("{:?}", traits));

        Ok(event)
    }

    pub fn get_endpoint(&self) -> String {
        self.credential
            .endpoint
            .clone()
            .unwrap_or_else(|| ActivationEngine::get_platform_endpoint(&Platform::Segment))
    }

    pub fn get_auth_header(&self) -> Result<(String, String)> {
        Ok((
            "Authorization".to_string(),
            format!("Basic {}", self.credential.api_key),
        ))
    }
}

/// RudderStack API adapter
pub struct RudderStackAdapter {
    credential: PlatformCredential,
}

impl RudderStackAdapter {
    pub fn new(credential: PlatformCredential) -> Self {
        Self { credential }
    }

    pub fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        let mut event = HashMap::new();

        if let Some(email) = &msg.customer.email {
            event.insert("email".to_string(), email.clone());
        }

        event.insert("anonymousId".to_string(), msg.customer.customer_id.clone());

        let mut traits = msg.customer.attributes.clone();
        if let Some(segment) = &msg.customer.segment {
            traits.insert("segment".to_string(), segment.clone());
        }

        event.insert("traits".to_string(), format!("{:?}", traits));

        Ok(event)
    }

    pub fn get_endpoint(&self) -> String {
        self.credential
            .endpoint
            .clone()
            .unwrap_or_else(|| ActivationEngine::get_platform_endpoint(&Platform::RudderStack))
    }

    pub fn get_auth_header(&self) -> Result<(String, String)> {
        Ok((
            "Authorization".to_string(),
            format!("Bearer {}", self.credential.api_key),
        ))
    }
}

/// Unified adapter factory
pub struct AdapterFactory;

impl AdapterFactory {
    pub fn create_adapter(credential: PlatformCredential) -> Result<Box<dyn PlatformAdapter>> {
        match credential.platform {
            Platform::Braze => Ok(Box::new(BrazeAdapter::new(credential))),
            Platform::Iterable => Ok(Box::new(IterableAdapter::new(credential))),
            Platform::Klaviyo => Ok(Box::new(KlaviyoAdapter::new(credential))),
            Platform::Salesforce => Ok(Box::new(SalesforceAdapter::new(credential))),
            Platform::HubSpot => Ok(Box::new(HubSpotAdapter::new(credential))),
            Platform::Segment => Ok(Box::new(SegmentAdapter::new(credential))),
            Platform::RudderStack => Ok(Box::new(RudderStackAdapter::new(credential))),
            Platform::Custom => Ok(Box::new(CustomAdapter::new(credential))),
        }
    }
}

/// Trait for platform adapters
pub trait PlatformAdapter {
    fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>>;
    fn get_endpoint(&self) -> String;
    fn get_auth_header(&self) -> Result<(String, String)>;
}

impl PlatformAdapter for BrazeAdapter {
    fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        Self::format_event(self, msg)
    }

    fn get_endpoint(&self) -> String {
        Self::get_endpoint(self)
    }

    fn get_auth_header(&self) -> Result<(String, String)> {
        Self::get_auth_header(self)
    }
}

impl PlatformAdapter for IterableAdapter {
    fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        Self::format_event(self, msg)
    }

    fn get_endpoint(&self) -> String {
        Self::get_endpoint(self)
    }

    fn get_auth_header(&self) -> Result<(String, String)> {
        Self::get_auth_header(self)
    }
}

impl PlatformAdapter for KlaviyoAdapter {
    fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        Self::format_event(self, msg)
    }

    fn get_endpoint(&self) -> String {
        Self::get_endpoint(self)
    }

    fn get_auth_header(&self) -> Result<(String, String)> {
        Self::get_auth_header(self)
    }
}

impl PlatformAdapter for SalesforceAdapter {
    fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        Self::format_event(self, msg)
    }

    fn get_endpoint(&self) -> String {
        Self::get_endpoint(self)
    }

    fn get_auth_header(&self) -> Result<(String, String)> {
        Self::get_auth_header(self)
    }
}

impl PlatformAdapter for HubSpotAdapter {
    fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        Self::format_event(self, msg)
    }

    fn get_endpoint(&self) -> String {
        Self::get_endpoint(self)
    }

    fn get_auth_header(&self) -> Result<(String, String)> {
        Self::get_auth_header(self)
    }
}

impl PlatformAdapter for SegmentAdapter {
    fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        Self::format_event(self, msg)
    }

    fn get_endpoint(&self) -> String {
        Self::get_endpoint(self)
    }

    fn get_auth_header(&self) -> Result<(String, String)> {
        Self::get_auth_header(self)
    }
}

impl PlatformAdapter for RudderStackAdapter {
    fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        Self::format_event(self, msg)
    }

    fn get_endpoint(&self) -> String {
        Self::get_endpoint(self)
    }

    fn get_auth_header(&self) -> Result<(String, String)> {
        Self::get_auth_header(self)
    }
}

/// Custom platform adapter (pass-through)
pub struct CustomAdapter {
    credential: PlatformCredential,
}

impl CustomAdapter {
    pub fn new(credential: PlatformCredential) -> Self {
        Self { credential }
    }
}

impl PlatformAdapter for CustomAdapter {
    fn format_event(&self, msg: &ActivationMessage) -> Result<HashMap<String, String>> {
        ActivationEngine::build_payload(msg)
    }

    fn get_endpoint(&self) -> String {
        self.credential
            .endpoint
            .clone()
            .unwrap_or_else(|| "https://example.com/activate".to_string())
    }

    fn get_auth_header(&self) -> Result<(String, String)> {
        Ok((
            "Authorization".to_string(),
            format!("Bearer {}", self.credential.api_key),
        ))
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
    fn test_braze_adapter() {
        let cred = PlatformCredential::new(Platform::Braze, "key_123".to_string());
        let adapter = BrazeAdapter::new(cred);
        let customer = create_test_customer();
        let msg = ActivationMessage::new(
            customer,
            ActivationEvent::SegmentAssignment,
            Platform::Braze,
        );

        let formatted = adapter.format_event(&msg).unwrap();
        assert!(formatted.contains_key("external_id"));
        assert_eq!(formatted.get("external_id"), Some(&"cust_123".to_string()));
    }

    #[test]
    fn test_iterable_adapter() {
        let cred = PlatformCredential::new(Platform::Iterable, "key_123".to_string());
        let adapter = IterableAdapter::new(cred);
        let customer = create_test_customer();
        let msg = ActivationMessage::new(
            customer,
            ActivationEvent::SegmentAssignment,
            Platform::Iterable,
        );

        let formatted = adapter.format_event(&msg).unwrap();
        assert!(formatted.contains_key("email"));
        assert_eq!(
            formatted.get("email"),
            Some(&"test@example.com".to_string())
        );
    }

    #[test]
    fn test_klaviyo_adapter() {
        let cred = PlatformCredential::new(Platform::Klaviyo, "key_123".to_string());
        let adapter = KlaviyoAdapter::new(cred);
        let customer = create_test_customer();
        let msg = ActivationMessage::new(
            customer,
            ActivationEvent::SegmentAssignment,
            Platform::Klaviyo,
        );

        let formatted = adapter.format_event(&msg).unwrap();
        assert!(formatted.contains_key("phone_number"));
        assert!(formatted.contains_key("list_id"));
    }

    #[test]
    fn test_salesforce_adapter() {
        let cred = PlatformCredential::new(Platform::Salesforce, "key_123".to_string());
        let adapter = SalesforceAdapter::new(cred);
        let customer = create_test_customer();
        let msg = ActivationMessage::new(
            customer,
            ActivationEvent::SegmentAssignment,
            Platform::Salesforce,
        );

        let formatted = adapter.format_event(&msg).unwrap();
        assert!(formatted.contains_key("Email"));
        assert!(formatted.contains_key("External_Id__c"));
    }

    #[test]
    fn test_hubspot_adapter() {
        let cred = PlatformCredential::new(Platform::HubSpot, "key_123".to_string());
        let adapter = HubSpotAdapter::new(cred);
        let customer = create_test_customer();
        let msg = ActivationMessage::new(
            customer,
            ActivationEvent::SegmentAssignment,
            Platform::HubSpot,
        );

        let formatted = adapter.format_event(&msg).unwrap();
        assert!(formatted.contains_key("email"));
        assert!(formatted.contains_key("hs_lead_status"));
    }

    #[test]
    fn test_segment_adapter() {
        let cred = PlatformCredential::new(Platform::Segment, "key_123".to_string());
        let adapter = SegmentAdapter::new(cred);
        let customer = create_test_customer();
        let msg = ActivationMessage::new(
            customer,
            ActivationEvent::SegmentAssignment,
            Platform::Segment,
        );

        let formatted = adapter.format_event(&msg).unwrap();
        assert!(formatted.contains_key("userId"));
        assert!(formatted.contains_key("traits"));
    }

    #[test]
    fn test_rudderstack_adapter() {
        let cred = PlatformCredential::new(Platform::RudderStack, "key_123".to_string());
        let adapter = RudderStackAdapter::new(cred);
        let customer = create_test_customer();
        let msg = ActivationMessage::new(
            customer,
            ActivationEvent::SegmentAssignment,
            Platform::RudderStack,
        );

        let formatted = adapter.format_event(&msg).unwrap();
        assert!(formatted.contains_key("anonymousId"));
        assert!(formatted.contains_key("traits"));
    }

    #[test]
    fn test_adapter_factory() {
        let cred = PlatformCredential::new(Platform::Braze, "key_123".to_string());
        let adapter = AdapterFactory::create_adapter(cred).unwrap();
        assert_eq!(
            adapter.get_endpoint(),
            ActivationEngine::get_platform_endpoint(&Platform::Braze)
        );
    }

    #[test]
    fn test_adapter_auth_headers() {
        let cred_braze = PlatformCredential::new(Platform::Braze, "key_123".to_string());
        let adapter_braze = BrazeAdapter::new(cred_braze);
        let (header, value) = adapter_braze.get_auth_header().unwrap();
        assert_eq!(header, "Authorization");
        assert!(value.contains("Bearer"));

        let cred_iterable = PlatformCredential::new(Platform::Iterable, "key_456".to_string());
        let adapter_iterable = IterableAdapter::new(cred_iterable);
        let (header, _value) = adapter_iterable.get_auth_header().unwrap();
        assert_eq!(header, "Api-Key");
    }
}
