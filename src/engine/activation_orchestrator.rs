//! Activation orchestration and batch processing

use crate::engine::activation::*;
use crate::Result;
use std::collections::HashMap;

/// Activation queue for batching
#[derive(Clone, Debug)]
pub struct ActivationQueue {
    messages: Vec<ActivationMessage>,
    max_size: usize,
}

impl ActivationQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_size,
        }
    }

    pub fn push(&mut self, message: ActivationMessage) -> Result<bool> {
        if self.messages.len() >= self.max_size {
            return Ok(false);
        }

        self.messages.push(message);
        Ok(true)
    }

    pub fn is_full(&self) -> bool {
        self.messages.len() >= self.max_size
    }

    pub fn size(&self) -> usize {
        self.messages.len()
    }

    pub fn flush(&mut self) -> Result<Vec<ActivationMessage>> {
        let messages = self.messages.drain(..).collect();
        Ok(messages)
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

/// Retry configuration for failed activations
#[derive(Clone, Debug)]
pub struct RetryPolicy {
    pub max_retries: usize,
    pub backoff_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff_ms: 1000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Activation orchestrator for managing batch sends
pub struct ActivationOrchestrator {
    config: BatchActivationConfig,
    retry_policy: RetryPolicy,
    platforms: HashMap<Platform, PlatformCredential>,
    results: Vec<ActivationResult>,
}

impl ActivationOrchestrator {
    pub fn new(config: BatchActivationConfig) -> Self {
        Self {
            config,
            retry_policy: RetryPolicy::default(),
            platforms: HashMap::new(),
            results: Vec::new(),
        }
    }

    pub fn register_platform(&mut self, credential: PlatformCredential) -> Result<()> {
        self.platforms
            .insert(credential.platform.clone(), credential);
        Ok(())
    }

    pub fn get_platform(&self, platform: &Platform) -> Option<&PlatformCredential> {
        self.platforms.get(platform)
    }

    pub fn has_platform(&self, platform: &Platform) -> bool {
        self.platforms.contains_key(platform)
    }

    pub fn platform_count(&self) -> usize {
        self.platforms.len()
    }

    /// Process a batch of activation messages
    pub fn process_batch(
        &mut self,
        messages: Vec<ActivationMessage>,
    ) -> Result<Vec<ActivationResult>> {
        let mut results = Vec::new();

        for message in messages {
            if !ActivationEngine::is_eligible(&message.customer, &message.platform)? {
                let result = ActivationResult::failure(
                    format!("msg_{}", message.customer.customer_id),
                    message.platform.clone(),
                    message.customer.customer_id.clone(),
                    "Customer not eligible for platform".to_string(),
                );
                results.push(result);
                continue;
            }

            let result = self.process_message(message)?;
            results.push(result);
        }

        self.results.extend(results.clone());
        Ok(results)
    }

    fn process_message(&self, message: ActivationMessage) -> Result<ActivationResult> {
        if !self.has_platform(&message.platform) {
            return Ok(ActivationResult::failure(
                format!("msg_{}", message.customer.customer_id),
                message.platform.clone(),
                message.customer.customer_id.clone(),
                "Platform not configured".to_string(),
            ));
        }

        let payload = ActivationEngine::build_payload(&message)?;

        if payload.is_empty() {
            return Ok(ActivationResult::failure(
                format!("msg_{}", message.customer.customer_id),
                message.platform.clone(),
                message.customer.customer_id.clone(),
                "Empty payload".to_string(),
            ));
        }

        Ok(ActivationResult::success(
            format!("msg_{}", message.customer.customer_id),
            message.platform.clone(),
            message.customer.customer_id.clone(),
        ))
    }

    /// Get all results collected
    pub fn get_results(&self) -> Vec<ActivationResult> {
        self.results.clone()
    }

    /// Get statistics for all results
    pub fn get_stats(&self) -> Result<ActivationStats> {
        ActivationEngine::calculate_stats(&self.results)
    }

    /// Clear results
    pub fn clear_results(&mut self) {
        self.results.clear();
    }

    /// Get success rate
    pub fn success_rate(&self) -> Result<f64> {
        if self.results.is_empty() {
            return Ok(0.0);
        }

        let successful = self.results.iter().filter(|r| r.success).count();
        Ok(successful as f64 / self.results.len() as f64)
    }
}

/// Segment activation manager
pub struct SegmentActivationManager;

impl SegmentActivationManager {
    /// Generate activation messages from customer segment
    pub fn activate_segment(
        customers: &[CustomerRecord],
        segment_name: String,
        platforms: &[Platform],
        event_type: ActivationEvent,
    ) -> Result<Vec<ActivationMessage>> {
        let mut messages = Vec::new();

        for customer in customers {
            for platform in platforms {
                let mut customer_with_segment = customer.clone();
                customer_with_segment.segment = Some(segment_name.clone());

                let msg = ActivationMessage::new(
                    customer_with_segment,
                    event_type.clone(),
                    platform.clone(),
                );

                messages.push(msg);
            }
        }

        Ok(messages)
    }

    /// Generate removal messages for segment
    pub fn deactivate_segment(
        customers: &[CustomerRecord],
        segment_name: String,
        platforms: &[Platform],
    ) -> Result<Vec<ActivationMessage>> {
        Self::activate_segment(
            customers,
            segment_name,
            platforms,
            ActivationEvent::SegmentRemoval,
        )
    }

    /// Update customer attributes across platforms
    pub fn update_attributes(
        customer: &CustomerRecord,
        platforms: &[Platform],
    ) -> Result<Vec<ActivationMessage>> {
        let mut messages = Vec::new();

        for platform in platforms {
            let msg = ActivationMessage::new(
                customer.clone(),
                ActivationEvent::PropertyUpdate,
                platform.clone(),
            );

            messages.push(msg);
        }

        Ok(messages)
    }
}

/// Webhook manager for event-driven activation
pub struct WebhookManager {
    triggers: HashMap<ActivationEvent, Vec<WebhookTrigger>>,
}

impl WebhookManager {
    pub fn new() -> Self {
        Self {
            triggers: HashMap::new(),
        }
    }

    pub fn register_trigger(&mut self, trigger: WebhookTrigger) -> Result<()> {
        self.triggers
            .entry(trigger.event_type.clone())
            .or_default()
            .push(trigger);

        Ok(())
    }

    pub fn get_triggers(&self, event_type: &ActivationEvent) -> Vec<WebhookTrigger> {
        self.triggers
            .get(event_type)
            .map(|t| t.iter().filter(|w| w.active).cloned().collect())
            .unwrap_or_default()
    }

    pub fn trigger_count(&self) -> usize {
        self.triggers.values().map(|v| v.len()).sum()
    }

    pub fn active_trigger_count(&self) -> usize {
        self.triggers
            .values()
            .flat_map(|v| v.iter())
            .filter(|t| t.active)
            .count()
    }
}

impl Default for WebhookManager {
    fn default() -> Self {
        Self::new()
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
            segment: None,
        }
    }

    #[test]
    fn test_activation_queue() {
        let mut queue = ActivationQueue::new(3);
        let customer = create_test_customer();

        let msg = ActivationMessage::new(
            customer,
            ActivationEvent::SegmentAssignment,
            Platform::Braze,
        );
        assert!(queue.push(msg).unwrap());
        assert_eq!(queue.size(), 1);
        assert!(!queue.is_full());
    }

    #[test]
    fn test_queue_capacity() {
        let mut queue = ActivationQueue::new(2);
        let customer = create_test_customer();

        let msg1 = ActivationMessage::new(
            customer.clone(),
            ActivationEvent::SegmentAssignment,
            Platform::Braze,
        );
        let msg2 = ActivationMessage::new(
            customer.clone(),
            ActivationEvent::SegmentAssignment,
            Platform::Braze,
        );
        let msg3 = ActivationMessage::new(
            customer,
            ActivationEvent::SegmentAssignment,
            Platform::Braze,
        );

        queue.push(msg1).unwrap();
        queue.push(msg2).unwrap();

        assert!(queue.is_full());
        assert!(!queue.push(msg3).unwrap());
    }

    #[test]
    fn test_queue_flush() {
        let mut queue = ActivationQueue::new(3);
        let customer = create_test_customer();

        let msg = ActivationMessage::new(
            customer,
            ActivationEvent::SegmentAssignment,
            Platform::Braze,
        );
        queue.push(msg).unwrap();

        let flushed = queue.flush().unwrap();
        assert_eq!(flushed.len(), 1);
        assert_eq!(queue.size(), 0);
    }

    #[test]
    fn test_orchestrator_register_platform() {
        let mut orch = ActivationOrchestrator::new(BatchActivationConfig::default());
        let cred = PlatformCredential::new(Platform::Braze, "key_123".to_string());

        orch.register_platform(cred).unwrap();
        assert!(orch.has_platform(&Platform::Braze));
        assert_eq!(orch.platform_count(), 1);
    }

    #[test]
    fn test_orchestrator_process_batch() {
        let mut orch = ActivationOrchestrator::new(BatchActivationConfig::default());
        let cred = PlatformCredential::new(Platform::Braze, "key_123".to_string());
        orch.register_platform(cred).unwrap();

        let customer = create_test_customer();
        let msg = ActivationMessage::new(
            customer,
            ActivationEvent::SegmentAssignment,
            Platform::Braze,
        );

        let results = orch.process_batch(vec![msg]).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_orchestrator_stats() {
        let mut orch = ActivationOrchestrator::new(BatchActivationConfig::default());
        let cred = PlatformCredential::new(Platform::Braze, "key_123".to_string());
        orch.register_platform(cred).unwrap();

        let customer = create_test_customer();
        let msg = ActivationMessage::new(
            customer,
            ActivationEvent::SegmentAssignment,
            Platform::Braze,
        );

        orch.process_batch(vec![msg]).unwrap();
        let stats = orch.get_stats().unwrap();

        assert_eq!(stats.total_messages, 1);
        assert_eq!(stats.successful, 1);
    }

    #[test]
    fn test_segment_activation_manager() {
        let customers = vec![create_test_customer()];
        let platforms = vec![Platform::Braze, Platform::Iterable];

        let messages = SegmentActivationManager::activate_segment(
            &customers,
            "Champions".to_string(),
            &platforms,
            ActivationEvent::SegmentAssignment,
        )
        .unwrap();

        assert_eq!(messages.len(), 2); // 1 customer × 2 platforms
    }

    #[test]
    fn test_segment_deactivation() {
        let customers = vec![create_test_customer()];
        let platforms = vec![Platform::Braze];

        let messages = SegmentActivationManager::deactivate_segment(
            &customers,
            "Champions".to_string(),
            &platforms,
        )
        .unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].event_type, ActivationEvent::SegmentRemoval);
    }

    #[test]
    fn test_update_attributes() {
        let customer = create_test_customer();
        let platforms = vec![Platform::Braze, Platform::Iterable];

        let messages = SegmentActivationManager::update_attributes(&customer, &platforms).unwrap();

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].event_type, ActivationEvent::PropertyUpdate);
    }

    #[test]
    fn test_webhook_manager() {
        let mut manager = WebhookManager::new();

        let trigger = WebhookTrigger::new(
            "https://example.com/webhook".to_string(),
            ActivationEvent::SegmentAssignment,
        );

        manager.register_trigger(trigger).unwrap();
        assert_eq!(manager.trigger_count(), 1);
        assert_eq!(manager.active_trigger_count(), 1);
    }

    #[test]
    fn test_webhook_get_triggers() {
        let mut manager = WebhookManager::new();

        let trigger = WebhookTrigger::new(
            "https://example.com/webhook".to_string(),
            ActivationEvent::SegmentAssignment,
        );

        manager.register_trigger(trigger).unwrap();

        let triggers = manager.get_triggers(&ActivationEvent::SegmentAssignment);
        assert_eq!(triggers.len(), 1);

        let other_triggers = manager.get_triggers(&ActivationEvent::SegmentRemoval);
        assert_eq!(other_triggers.len(), 0);
    }

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();

        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.backoff_ms, 1000);
        assert_eq!(policy.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_success_rate() {
        let mut orch = ActivationOrchestrator::new(BatchActivationConfig::default());
        let cred = PlatformCredential::new(Platform::Braze, "key_123".to_string());
        orch.register_platform(cred).unwrap();

        let customer = create_test_customer();
        let msg = ActivationMessage::new(
            customer,
            ActivationEvent::SegmentAssignment,
            Platform::Braze,
        );

        orch.process_batch(vec![msg]).unwrap();
        let success_rate = orch.success_rate().unwrap();

        assert!(success_rate > 0.0 && success_rate <= 1.0);
    }
}
