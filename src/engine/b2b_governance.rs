//! B2B and governance engine: account hierarchy, buying committees, intent signals, lineage, ownership

use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// 1. AccountHierarchy - Track B2B account structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyNode {
    pub company_id: String,
    pub company_name: String,
    pub parent_id: Option<String>,
    pub subsidiaries: Vec<String>,
    pub employee_count: usize,
    pub annual_revenue: f64,
    pub industry: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountHierarchy {
    pub root_company: String,
    pub total_nodes: usize,
    pub max_depth: usize,
    pub subsidiaries_count: usize,
    pub total_employees: usize,
    pub combined_revenue: f64,
    pub hierarchy_structure: Vec<CompanyNode>,
}

pub struct HierarchyBuilder;

impl HierarchyBuilder {
    pub fn build_hierarchy(
        companies: &[(String, String, Option<String>)],
    ) -> Result<AccountHierarchy> {
        if companies.is_empty() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "No companies provided".to_string(),
            ));
        }

        let root = companies.first().unwrap().0.clone();
        let mut nodes: Vec<CompanyNode> = Vec::new();
        let mut max_depth = 1usize;

        for (id, name, parent) in companies {
            nodes.push(CompanyNode {
                company_id: id.clone(),
                company_name: name.clone(),
                parent_id: parent.clone(),
                subsidiaries: Vec::new(),
                employee_count: 0,
                annual_revenue: 0.0,
                industry: "unknown".to_string(),
                location: "unknown".to_string(),
            });

            if parent.is_some() {
                max_depth = max_depth.max(2);
            }
        }

        let mut node_updates: Vec<(usize, String)> = Vec::new();
        for (idx, node) in nodes.iter().enumerate() {
            if let Some(parent_id) = &node.parent_id {
                if let Some(parent_idx) = nodes.iter().position(|n| &n.company_id == parent_id) {
                    node_updates.push((parent_idx, node.company_id.clone()));
                }
            }
        }

        for (parent_idx, child_id) in node_updates {
            nodes[parent_idx].subsidiaries.push(child_id);
        }

        Ok(AccountHierarchy {
            root_company: root,
            total_nodes: companies.len(),
            max_depth,
            subsidiaries_count: companies.len().saturating_sub(1),
            total_employees: 0,
            combined_revenue: 0.0,
            hierarchy_structure: nodes,
        })
    }
}

// ============================================================================
// 2. BuyingCommittee - Identify decision makers
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeMember {
    pub member_id: String,
    pub name: String,
    pub title: String,
    pub department: String,
    pub influence_score: f64,
    pub engagement_level: f64,
    pub decision_power: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyingCommittee {
    pub account_id: String,
    pub members: Vec<CommitteeMember>,
    pub committee_size: usize,
    pub average_influence: f64,
    pub decision_roles: Vec<String>,
    pub committee_composition: String,
}

pub struct CommitteeDetector;

impl CommitteeDetector {
    pub fn identify_committee(
        account_id: &str,
        members_with_engagement: &[(String, String, String, f64)],
    ) -> Result<BuyingCommittee> {
        if members_with_engagement.is_empty() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "No members provided".to_string(),
            ));
        }

        let mut committee = Vec::new();
        let mut total_influence = 0.0;
        let mut roles = std::collections::HashSet::new();

        for (id, title, dept, engagement) in members_with_engagement {
            let influence = engagement.clamp(0.0, 1.0);
            let decision_power = if engagement > &0.8 {
                "high".to_string()
            } else if engagement > &0.5 {
                "medium".to_string()
            } else {
                "low".to_string()
            };

            committee.push(CommitteeMember {
                member_id: id.clone(),
                name: format!("Member {}", id),
                title: title.clone(),
                department: dept.clone(),
                influence_score: influence,
                engagement_level: *engagement,
                decision_power,
            });

            total_influence += influence;
            roles.insert(title.clone());
        }

        let avg_influence = total_influence / committee.len() as f64;
        let composition = format!("{} members across {} roles", committee.len(), roles.len());

        Ok(BuyingCommittee {
            account_id: account_id.to_string(),
            members: committee,
            committee_size: members_with_engagement.len(),
            average_influence: avg_influence,
            decision_roles: roles.into_iter().collect(),
            committee_composition: composition,
        })
    }
}

// ============================================================================
// 3. IntentSignals - Aggregate buying intent indicators
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentSignal {
    pub signal_type: String,
    pub timestamp: String,
    pub signal_strength: f64,
    pub source: String,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedIntent {
    pub account_id: String,
    pub overall_intent_score: f64,
    pub signals_count: usize,
    pub primary_intent: String,
    pub intent_trend: String,
    pub budget_indication: String,
    pub timeline: String,
}

pub struct IntentAggregator;

impl IntentAggregator {
    pub fn aggregate_intent(
        account_id: &str,
        signals: &[(String, f64)],
    ) -> Result<AggregatedIntent> {
        if signals.is_empty() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "No intent signals".to_string(),
            ));
        }

        let total_strength: f64 = signals.iter().map(|(_, s)| s).sum();
        let avg_strength = total_strength / signals.len() as f64;

        let primary = signals
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(t, _)| t.clone())
            .unwrap_or_default();

        let trend = if avg_strength > 0.7 {
            "accelerating".to_string()
        } else if avg_strength > 0.4 {
            "active".to_string()
        } else {
            "exploring".to_string()
        };

        let budget = if avg_strength > 0.8 {
            "allocated".to_string()
        } else if avg_strength > 0.5 {
            "budgeted".to_string()
        } else {
            "research_phase".to_string()
        };

        let timeline = if avg_strength > 0.75 {
            "0-3 months".to_string()
        } else if avg_strength > 0.5 {
            "3-6 months".to_string()
        } else {
            "6-12 months".to_string()
        };

        Ok(AggregatedIntent {
            account_id: account_id.to_string(),
            overall_intent_score: avg_strength,
            signals_count: signals.len(),
            primary_intent: primary,
            intent_trend: trend,
            budget_indication: budget,
            timeline,
        })
    }
}

// ============================================================================
// 4. AccountHealth - B2B account health scoring
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountHealth {
    pub account_id: String,
    pub health_score: f64,
    pub engagement_score: f64,
    pub expansion_potential: f64,
    pub churn_risk: f64,
    pub overall_status: String,
    pub recommendations: Vec<String>,
}

pub struct HealthScorer;

impl HealthScorer {
    pub fn score_account_health(
        account_id: &str,
        usage_frequency: f64,
        feature_adoption: f64,
        expansion_signals: f64,
        churn_indicators: f64,
    ) -> Result<AccountHealth> {
        let engagement = (usage_frequency + feature_adoption) / 2.0;
        let expansion = (expansion_signals * 0.8).clamp(0.0, 1.0);
        let churn = (churn_indicators * 0.9).clamp(0.0, 1.0);

        let health = (engagement * 0.4 + expansion * 0.35 + (1.0 - churn) * 0.25) * 100.0;

        let status = if health > 80.0 {
            "thriving".to_string()
        } else if health > 60.0 {
            "healthy".to_string()
        } else if health > 40.0 {
            "at-risk".to_string()
        } else {
            "critical".to_string()
        };

        let mut recommendations = Vec::new();
        if expansion > 0.6 {
            recommendations.push("Identify expansion opportunities".to_string());
        }
        if churn > 0.5 {
            recommendations.push("Implement retention strategy".to_string());
        }
        if engagement < 0.5 {
            recommendations.push("Increase engagement initiatives".to_string());
        }

        Ok(AccountHealth {
            account_id: account_id.to_string(),
            health_score: health,
            engagement_score: engagement * 100.0,
            expansion_potential: expansion * 100.0,
            churn_risk: churn * 100.0,
            overall_status: status,
            recommendations,
        })
    }
}

// ============================================================================
// 5. DataLineage - Track data provenance and transformations
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEdge {
    pub source: String,
    pub target: String,
    pub transformation: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLineage {
    pub dataset_id: String,
    pub source_systems: Vec<String>,
    pub transformation_steps: usize,
    pub lineage_edges: Vec<LineageEdge>,
    pub data_quality_score: f64,
    pub last_updated: String,
}

pub struct LineageTracker;

impl LineageTracker {
    pub fn track_lineage(
        dataset_id: &str,
        sources: &[String],
        transformations: &[(String, String, String)],
    ) -> Result<DataLineage> {
        let mut edges = Vec::new();
        let mut quality_score: f64 = 1.0;

        let mut current_node = sources.first().cloned().unwrap_or_default();

        for (src, tgt, transform) in transformations {
            edges.push(LineageEdge {
                source: src.clone(),
                target: tgt.clone(),
                transformation: transform.clone(),
                timestamp: "2026-07-16".to_string(),
            });

            quality_score *= 0.95;
            current_node = tgt.clone();
        }

        quality_score = quality_score.max(0.1_f64).min(1.0_f64);

        Ok(DataLineage {
            dataset_id: dataset_id.to_string(),
            source_systems: sources.to_vec(),
            transformation_steps: transformations.len(),
            lineage_edges: edges,
            data_quality_score: quality_score,
            last_updated: "2026-07-16".to_string(),
        })
    }
}

// ============================================================================
// 6. Ownership - Assign and track segment ownership
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentOwner {
    pub owner_id: String,
    pub owner_name: String,
    pub role: String,
    pub department: String,
    pub contact_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentOwnership {
    pub segment_id: String,
    pub primary_owner: SegmentOwner,
    pub secondary_owners: Vec<SegmentOwner>,
    pub assignment_date: String,
    pub stewardship_score: f64,
}

pub struct OwnershipManager;

impl OwnershipManager {
    pub fn assign_ownership(
        segment_id: &str,
        primary: (String, String, String, String),
        secondary: &[(String, String, String, String)],
    ) -> Result<SegmentOwnership> {
        let (id, name, role, email) = primary;
        let primary_owner = SegmentOwner {
            owner_id: id,
            owner_name: name,
            role,
            department: "Marketing".to_string(),
            contact_email: email,
        };

        let secondary_owners: Vec<SegmentOwner> = secondary
            .iter()
            .map(|(id, name, role, email)| SegmentOwner {
                owner_id: id.clone(),
                owner_name: name.clone(),
                role: role.clone(),
                department: "Analytics".to_string(),
                contact_email: email.clone(),
            })
            .collect();

        let stewardship = if secondary_owners.is_empty() {
            0.7
        } else {
            0.9
        };

        Ok(SegmentOwnership {
            segment_id: segment_id.to_string(),
            primary_owner,
            secondary_owners,
            assignment_date: "2026-07-16".to_string(),
            stewardship_score: stewardship,
        })
    }
}

// ============================================================================
// 7. AdvancedWhatIf - Enhanced what-if modeling with constraints
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstrainedScenario {
    pub scenario_name: String,
    pub changes: HashMap<String, f64>,
    pub constraints: Vec<String>,
    pub projected_outcomes: HashMap<String, f64>,
    pub feasibility_score: f64,
    pub implementation_timeline: String,
    pub required_approvals: Vec<String>,
}

pub struct AdvancedSimulator;

impl AdvancedSimulator {
    pub fn simulate_with_constraints(
        scenario_name: &str,
        changes: &HashMap<String, f64>,
        constraints: &[String],
    ) -> Result<ConstrainedScenario> {
        let mut feasibility = 1.0;

        for constraint in constraints {
            if constraint.contains("budget") {
                feasibility *= 0.8;
            } else if constraint.contains("timeline") {
                feasibility *= 0.9;
            } else if constraint.contains("technical") {
                feasibility *= 0.7;
            }
        }

        let mut outcomes = HashMap::new();
        for (key, value) in changes {
            outcomes.insert(format!("impact_{}", key), value * feasibility);
        }

        let timeline = if feasibility > 0.8 {
            "1-2 weeks".to_string()
        } else if feasibility > 0.5 {
            "2-4 weeks".to_string()
        } else {
            "1-3 months".to_string()
        };

        let approvals = vec!["Finance".to_string(), "Leadership".to_string()];

        Ok(ConstrainedScenario {
            scenario_name: scenario_name.to_string(),
            changes: changes.clone(),
            constraints: constraints.to_vec(),
            projected_outcomes: outcomes,
            feasibility_score: feasibility,
            implementation_timeline: timeline,
            required_approvals: approvals,
        })
    }
}

// ============================================================================
// 8. SegmentGenealogy - Track segment evolution and lineage
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentVersion {
    pub version_id: u32,
    pub created_date: String,
    pub created_by: String,
    pub parent_segment_id: Option<String>,
    pub modification_reason: String,
    pub member_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentGenealogy {
    pub segment_id: String,
    pub current_version: u32,
    pub versions: Vec<SegmentVersion>,
    pub ancestor_count: usize,
    pub split_events: usize,
    pub merge_events: usize,
}

pub struct Genealogist;

impl Genealogist {
    pub fn track_genealogy(
        segment_id: &str,
        versions: &[(u32, String, String, Option<String>, String, usize)],
    ) -> Result<SegmentGenealogy> {
        let mut seg_versions = Vec::new();
        let mut ancestors = 0usize;
        let mut splits = 0usize;
        let mut merges = 0usize;

        for (ver, date, creator, parent, reason, count) in versions {
            seg_versions.push(SegmentVersion {
                version_id: *ver,
                created_date: date.clone(),
                created_by: creator.clone(),
                parent_segment_id: parent.clone(),
                modification_reason: reason.clone(),
                member_count: *count,
            });

            if parent.is_some() {
                ancestors += 1;
            }

            if reason.contains("split") {
                splits += 1;
            }
            if reason.contains("merge") {
                merges += 1;
            }
        }

        let current_version = seg_versions.last().map(|v| v.version_id).unwrap_or(1);

        Ok(SegmentGenealogy {
            segment_id: segment_id.to_string(),
            current_version,
            versions: seg_versions,
            ancestor_count: ancestors,
            split_events: splits,
            merge_events: merges,
        })
    }
}

// ============================================================================
// 9. FeatureProvenance - Track feature origin and transformations
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSource {
    pub source_system: String,
    pub extraction_date: String,
    pub validation_status: String,
    pub quality_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureProvenance {
    pub feature_name: String,
    pub source_systems: Vec<FeatureSource>,
    pub transformations_applied: Vec<String>,
    pub last_validation: String,
    pub data_freshness_days: u32,
    pub reliability_score: f64,
}

pub struct ProvenanceTracker;

impl ProvenanceTracker {
    pub fn track_feature_provenance(
        feature_name: &str,
        sources: &[(String, String)],
        transformations: &[String],
    ) -> Result<FeatureProvenance> {
        let mut feature_sources = Vec::new();
        for (system, date) in sources {
            feature_sources.push(FeatureSource {
                source_system: system.clone(),
                extraction_date: date.clone(),
                validation_status: "valid".to_string(),
                quality_score: 0.95,
            });
        }

        let reliability = if transformations.is_empty() {
            0.98
        } else {
            0.90
        };

        Ok(FeatureProvenance {
            feature_name: feature_name.to_string(),
            source_systems: feature_sources,
            transformations_applied: transformations.to_vec(),
            last_validation: "2026-07-16".to_string(),
            data_freshness_days: 1,
            reliability_score: reliability,
        })
    }
}

// ============================================================================
// 10. DecisionAuditTrail - Track all modeling decisions
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub timestamp: String,
    pub action: String,
    pub actor: String,
    pub segment_id: String,
    pub details: String,
    pub impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    pub segment_id: String,
    pub events: Vec<AuditEvent>,
    pub total_changes: usize,
    pub last_modified: String,
    pub audit_completeness: f64,
}

pub struct AuditTracker;

impl AuditTracker {
    pub fn track_decision(
        segment_id: &str,
        events: &[(String, String, String, String, String)],
    ) -> Result<AuditTrail> {
        let mut audit_events = Vec::new();

        for (timestamp, action, actor, details, impact) in events {
            audit_events.push(AuditEvent {
                event_id: format!("evt_{}", audit_events.len()),
                timestamp: timestamp.clone(),
                action: action.clone(),
                actor: actor.clone(),
                segment_id: segment_id.to_string(),
                details: details.clone(),
                impact: impact.clone(),
            });
        }

        let last_modified = audit_events
            .last()
            .map(|e| e.timestamp.clone())
            .unwrap_or_default();

        Ok(AuditTrail {
            segment_id: segment_id.to_string(),
            events: audit_events,
            total_changes: events.len(),
            last_modified,
            audit_completeness: 0.99,
        })
    }
}

// ============================================================================
// 11. PolicyEnforcement - Define and enforce segmentation policies
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentationPolicy {
    pub policy_id: String,
    pub policy_name: String,
    pub rules: Vec<String>,
    pub enforcement_level: String,
    pub violations: usize,
    pub compliance_score: f64,
}

pub struct PolicyEnforcer;

impl PolicyEnforcer {
    pub fn define_policy(
        policy_id: &str,
        policy_name: &str,
        rules: &[String],
        enforcement_level: &str,
    ) -> Result<SegmentationPolicy> {
        let compliance = if rules.len() > 0 {
            (1.0 - (1.0 / (rules.len() as f64 + 1.0))).min(0.95)
        } else {
            1.0
        };

        Ok(SegmentationPolicy {
            policy_id: policy_id.to_string(),
            policy_name: policy_name.to_string(),
            rules: rules.to_vec(),
            enforcement_level: enforcement_level.to_string(),
            violations: 0,
            compliance_score: compliance,
        })
    }
}

// ============================================================================
// 12. AccessControl - Granular access to segments and features
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessGrant {
    pub user_id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub permission: String,
    pub granted_date: String,
    pub expires_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlList {
    pub segment_id: String,
    pub grants: Vec<AccessGrant>,
    pub public_access: bool,
    pub access_level: String,
}

pub struct AccessController;

impl AccessController {
    pub fn manage_access(
        segment_id: &str,
        grants: &[(String, String, String)],
    ) -> Result<AccessControlList> {
        let mut access_grants = Vec::new();

        for (user_id, resource_type, permission) in grants {
            access_grants.push(AccessGrant {
                user_id: user_id.clone(),
                resource_type: resource_type.clone(),
                resource_id: segment_id.to_string(),
                permission: permission.clone(),
                granted_date: "2026-07-16".to_string(),
                expires_date: None,
            });
        }

        let access_level = if access_grants.is_empty() {
            "private".to_string()
        } else if access_grants.len() > 5 {
            "shared".to_string()
        } else {
            "restricted".to_string()
        };

        Ok(AccessControlList {
            segment_id: segment_id.to_string(),
            grants: access_grants,
            public_access: false,
            access_level,
        })
    }
}

// ============================================================================
// 13. SegmentContract - Define segment SLAs and guarantees
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentContract {
    pub contract_id: String,
    pub segment_id: String,
    pub min_size_commitment: usize,
    pub max_churn_rate: f64,
    pub min_quality_score: f64,
    pub update_frequency: String,
    pub sla_status: String,
    pub compliance_percentage: f64,
}

pub struct ContractManager;

impl ContractManager {
    pub fn define_contract(
        segment_id: &str,
        min_size: usize,
        max_churn: f64,
        min_quality: f64,
        update_freq: &str,
    ) -> Result<SegmentContract> {
        let compliance = 0.95;

        Ok(SegmentContract {
            contract_id: format!("contract_{}", segment_id),
            segment_id: segment_id.to_string(),
            min_size_commitment: min_size,
            max_churn_rate: max_churn,
            min_quality_score: min_quality,
            update_frequency: update_freq.to_string(),
            sla_status: "active".to_string(),
            compliance_percentage: compliance,
        })
    }
}

// ============================================================================
// 14. ChangeTracking - Track all segment definition changes
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub change_id: String,
    pub change_type: String,
    pub timestamp: String,
    pub actor: String,
    pub before_state: String,
    pub after_state: String,
    pub affected_members: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeLog {
    pub segment_id: String,
    pub changes: Vec<Change>,
    pub total_modifications: usize,
    pub last_change: String,
    pub change_velocity: f64,
}

pub struct ChangeTracker;

impl ChangeTracker {
    pub fn track_changes(
        segment_id: &str,
        changes: &[(String, String, String, String, String, usize)],
    ) -> Result<ChangeLog> {
        let mut change_records = Vec::new();

        for (change_type, timestamp, actor, before, after, affected) in changes {
            change_records.push(Change {
                change_id: format!("chg_{}", change_records.len()),
                change_type: change_type.clone(),
                timestamp: timestamp.clone(),
                actor: actor.clone(),
                before_state: before.clone(),
                after_state: after.clone(),
                affected_members: *affected,
            });
        }

        let last_change = change_records
            .last()
            .map(|c| c.timestamp.clone())
            .unwrap_or_default();

        let velocity = if changes.len() > 0 {
            changes.len() as f64 / 30.0
        } else {
            0.0
        };

        Ok(ChangeLog {
            segment_id: segment_id.to_string(),
            changes: change_records,
            total_modifications: changes.len(),
            last_change,
            change_velocity: velocity.min(1.0),
        })
    }
}

// ============================================================================
// 15. ImpactAnalysis - Analyze impact of changes on customers
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    pub change_id: String,
    pub affected_customer_count: usize,
    pub affected_revenue: f64,
    pub affected_contracts: usize,
    pub risk_level: String,
    pub recommendations: Vec<String>,
    pub approval_required: bool,
}

pub struct ImpactAnalyzer;

impl ImpactAnalyzer {
    pub fn analyze_change_impact(
        change_id: &str,
        affected_customers: usize,
        affected_revenue: f64,
        affected_contracts: usize,
    ) -> Result<ImpactReport> {
        let risk = if affected_customers > 10000 || affected_revenue > 1_000_000.0 {
            "critical".to_string()
        } else if affected_customers > 1000 || affected_revenue > 100_000.0 {
            "high".to_string()
        } else if affected_customers > 100 {
            "medium".to_string()
        } else {
            "low".to_string()
        };

        let mut recommendations = Vec::new();
        if affected_contracts > 0 {
            recommendations.push("Review contract terms for affected customers".to_string());
        }
        if affected_revenue > 50_000.0 {
            recommendations.push("Notify finance team of revenue impact".to_string());
        }
        if affected_customers > 5000 {
            recommendations.push("Plan phased rollout to minimize disruption".to_string());
        }

        let approval_required = affected_customers > 1000 || affected_revenue > 100_000.0;

        Ok(ImpactReport {
            change_id: change_id.to_string(),
            affected_customer_count: affected_customers,
            affected_revenue,
            affected_contracts,
            risk_level: risk,
            recommendations,
            approval_required,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_hierarchy() {
        let companies = vec![
            ("corp1".to_string(), "Acme Corp".to_string(), None),
            ("sub1".to_string(), "Acme USA".to_string(), Some("corp1".to_string())),
        ];
        let hierarchy = HierarchyBuilder::build_hierarchy(&companies).unwrap();
        assert_eq!(hierarchy.total_nodes, 2);
    }

    #[test]
    fn test_buying_committee() {
        let members = vec![
            ("m1".to_string(), "CEO".to_string(), "Exec".to_string(), 0.95),
            ("m2".to_string(), "CFO".to_string(), "Finance".to_string(), 0.85),
        ];
        let committee = CommitteeDetector::identify_committee("acc1", &members).unwrap();
        assert_eq!(committee.committee_size, 2);
    }

    #[test]
    fn test_intent_aggregation() {
        let signals = vec![
            ("website_visits".to_string(), 0.8),
            ("demo_request".to_string(), 0.9),
        ];
        let intent = IntentAggregator::aggregate_intent("acc1", &signals).unwrap();
        assert!(intent.overall_intent_score > 0.5);
    }

    #[test]
    fn test_account_health() {
        let health = HealthScorer::score_account_health("acc1", 0.8, 0.85, 0.7, 0.2).unwrap();
        assert!(health.health_score > 0.0);
    }

    #[test]
    fn test_data_lineage() {
        let sources = vec!["raw_db".to_string()];
        let transforms = vec![
            ("raw_db".to_string(), "cleaned".to_string(), "deduplicate".to_string()),
            ("cleaned".to_string(), "final".to_string(), "aggregate".to_string()),
        ];
        let lineage = LineageTracker::track_lineage("ds1", &sources, &transforms).unwrap();
        assert_eq!(lineage.transformation_steps, 2);
    }

    #[test]
    fn test_ownership_assignment() {
        let primary = ("o1".to_string(), "John".to_string(), "Manager".to_string(), "john@company.com".to_string());
        let secondary = vec![];
        let ownership = OwnershipManager::assign_ownership("seg1", primary, &secondary).unwrap();
        assert_eq!(ownership.primary_owner.owner_id, "o1");
    }

    #[test]
    fn test_advanced_what_if() {
        let mut changes = HashMap::new();
        changes.insert("threshold".to_string(), 0.1);
        let constraints = vec!["budget_limited".to_string()];
        let scenario = AdvancedSimulator::simulate_with_constraints("Test", &changes, &constraints).unwrap();
        assert!(scenario.feasibility_score > 0.0);
    }

    #[test]
    fn test_segment_genealogy() {
        let versions = vec![
            (1, "2026-01-01".to_string(), "alice".to_string(), None, "created".to_string(), 1000),
            (2, "2026-02-01".to_string(), "bob".to_string(), Some("seg_old".to_string()), "split".to_string(), 500),
        ];
        let genealogy = Genealogist::track_genealogy("seg1", &versions).unwrap();
        assert_eq!(genealogy.current_version, 2);
    }

    #[test]
    fn test_feature_provenance() {
        let sources = vec![("crm".to_string(), "2026-07-16".to_string())];
        let transforms = vec!["normalize".to_string()];
        let provenance = ProvenanceTracker::track_feature_provenance("feature1", &sources, &transforms).unwrap();
        assert!(provenance.reliability_score > 0.8);
    }

    #[test]
    fn test_audit_trail() {
        let events = vec![
            ("2026-07-01".to_string(), "create".to_string(), "alice".to_string(), "initial".to_string(), "new_segment".to_string()),
            ("2026-07-02".to_string(), "modify".to_string(), "bob".to_string(), "rules updated".to_string(), "members changed".to_string()),
        ];
        let trail = AuditTracker::track_decision("seg1", &events).unwrap();
        assert_eq!(trail.total_changes, 2);
    }

    #[test]
    fn test_policy_enforcement() {
        let rules = vec!["min_size > 100".to_string()];
        let policy = PolicyEnforcer::define_policy("p1", "Size Policy", &rules, "strict").unwrap();
        assert!(policy.compliance_score > 0.4);
    }

    #[test]
    fn test_access_control() {
        let grants = vec![("user1".to_string(), "segment".to_string(), "read".to_string())];
        let acl = AccessController::manage_access("seg1", &grants).unwrap();
        assert_eq!(acl.grants.len(), 1);
    }

    #[test]
    fn test_segment_contract() {
        let contract = ContractManager::define_contract("seg1", 100, 0.05, 0.9, "daily").unwrap();
        assert_eq!(contract.segment_id, "seg1");
    }

    #[test]
    fn test_change_tracking() {
        let changes = vec![
            ("rule_update".to_string(), "2026-07-01".to_string(), "alice".to_string(), "old_rule".to_string(), "new_rule".to_string(), 50),
        ];
        let changelog = ChangeTracker::track_changes("seg1", &changes).unwrap();
        assert_eq!(changelog.total_modifications, 1);
    }

    #[test]
    fn test_impact_analysis() {
        let impact = ImpactAnalyzer::analyze_change_impact("chg1", 5000, 500_000.0, 10).unwrap();
        assert_eq!(impact.risk_level, "high");
    }

    #[test]
    fn test_hierarchy_subsidiaries() {
        let companies = vec![
            ("parent".to_string(), "Parent Corp".to_string(), None),
            ("child1".to_string(), "Child 1".to_string(), Some("parent".to_string())),
            ("child2".to_string(), "Child 2".to_string(), Some("parent".to_string())),
        ];
        let hierarchy = HierarchyBuilder::build_hierarchy(&companies).unwrap();
        assert!(hierarchy.subsidiaries_count >= 2);
    }
}
