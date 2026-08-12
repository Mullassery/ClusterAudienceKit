//! Behavioral segmentation framework with rule-based filtering

use crate::Result;
use std::collections::HashMap;

/// Comparison operator for rules
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComparisonOp {
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    Equal,
    NotEqual,
    In,
    Contains,
}

impl std::fmt::Display for ComparisonOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonOp::GreaterThan => write!(f, ">"),
            ComparisonOp::LessThan => write!(f, "<"),
            ComparisonOp::GreaterOrEqual => write!(f, ">="),
            ComparisonOp::LessOrEqual => write!(f, "<="),
            ComparisonOp::Equal => write!(f, "="),
            ComparisonOp::NotEqual => write!(f, "!="),
            ComparisonOp::In => write!(f, "IN"),
            ComparisonOp::Contains => write!(f, "CONTAINS"),
        }
    }
}

/// Logical operator for combining conditions
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LogicalOp {
    And,
    Or,
    Not,
}

impl std::fmt::Display for LogicalOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicalOp::And => write!(f, "AND"),
            LogicalOp::Or => write!(f, "OR"),
            LogicalOp::Not => write!(f, "NOT"),
        }
    }
}

/// Aggregate function for conditions
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AggregateFunction {
    Sum,
    Average,
    Count,
    Min,
    Max,
}

impl std::fmt::Display for AggregateFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AggregateFunction::Sum => write!(f, "SUM"),
            AggregateFunction::Average => write!(f, "AVG"),
            AggregateFunction::Count => write!(f, "COUNT"),
            AggregateFunction::Min => write!(f, "MIN"),
            AggregateFunction::Max => write!(f, "MAX"),
        }
    }
}

/// Value type for rule conditions
#[derive(Clone, Debug, PartialEq)]
pub enum RuleValue {
    Number(f64),
    String(String),
    List(Vec<String>),
}

impl std::fmt::Display for RuleValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleValue::Number(n) => write!(f, "{}", n),
            RuleValue::String(s) => write!(f, "'{}'", s),
            RuleValue::List(items) => {
                write!(f, "({})", items.join(", "))
            }
        }
    }
}

/// Individual condition in a rule
#[derive(Clone, Debug)]
pub struct Condition {
    pub field: String,
    pub operator: ComparisonOp,
    pub value: RuleValue,
    pub aggregate: Option<AggregateFunction>,
    pub time_window_days: Option<u32>,
}

impl Condition {
    pub fn new(field: &str, operator: ComparisonOp, value: RuleValue) -> Self {
        Self {
            field: field.to_string(),
            operator,
            value,
            aggregate: None,
            time_window_days: None,
        }
    }

    pub fn with_aggregate(mut self, agg: AggregateFunction) -> Self {
        self.aggregate = Some(agg);
        self
    }

    pub fn with_time_window(mut self, days: u32) -> Self {
        self.time_window_days = Some(days);
        self
    }

    pub fn to_sql(&self) -> String {
        let mut sql = String::new();

        // Add aggregate if present
        if let Some(agg) = &self.aggregate {
            sql.push_str(&format!("{}(", agg));
        }

        sql.push_str(&self.field);

        if self.aggregate.is_some() {
            sql.push(')');
        }

        // Add time window if present
        if let Some(days) = self.time_window_days {
            sql.push_str(&format!(" in last {} days", days));
        }

        // Add operator and value
        match &self.value {
            RuleValue::List(_) => {
                sql.push_str(&format!(" {} {}", self.operator, self.value));
            }
            _ => {
                sql.push_str(&format!(" {} {}", self.operator, self.value));
            }
        }

        sql
    }

    pub fn evaluate(&self, value: f64) -> bool {
        match &self.value {
            RuleValue::Number(n) => match self.operator {
                ComparisonOp::GreaterThan => value > *n,
                ComparisonOp::LessThan => value < *n,
                ComparisonOp::GreaterOrEqual => value >= *n,
                ComparisonOp::LessOrEqual => value <= *n,
                ComparisonOp::Equal => (value - n).abs() < 1e-10,
                ComparisonOp::NotEqual => (value - n).abs() >= 1e-10,
                _ => false,
            },
            _ => false,
        }
    }
}

/// Behavioral rule with conditions
#[derive(Clone, Debug)]
pub struct BehavioralRule {
    pub name: String,
    pub description: String,
    pub conditions: Vec<Condition>,
    pub logic: LogicalOp,
}

impl BehavioralRule {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            conditions: Vec::new(),
            logic: LogicalOp::And,
        }
    }

    pub fn add_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    pub fn with_logic(mut self, logic: LogicalOp) -> Self {
        self.logic = logic;
        self
    }

    pub fn to_sql(&self) -> String {
        if self.conditions.is_empty() {
            return "1=1".to_string();
        }

        let condition_sqls: Vec<String> = self.conditions.iter().map(|c| c.to_sql()).collect();

        let operator_str = match self.logic {
            LogicalOp::And => "AND",
            LogicalOp::Or => "OR",
            LogicalOp::Not => "NOT",
        };

        if self.logic == LogicalOp::Not {
            format!("NOT ({})", condition_sqls.join(" OR "))
        } else {
            condition_sqls.join(&format!(" {} ", operator_str))
        }
    }
}

/// Behavioral segment with rules
#[derive(Clone, Debug)]
pub struct BehavioralSegment {
    pub name: String,
    pub description: String,
    pub rules: Vec<BehavioralRule>,
    pub priority: u32,
}

impl BehavioralSegment {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            rules: Vec::new(),
            priority: 0,
        }
    }

    pub fn add_rule(mut self, rule: BehavioralRule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn to_sql(&self) -> String {
        if self.rules.is_empty() {
            return "1=1".to_string();
        }

        let rule_sqls: Vec<String> = self.rules.iter().map(|r| r.to_sql()).collect();
        format!("({})", rule_sqls.join(" AND "))
    }

    pub fn matches(&self, customer_data: &HashMap<String, f64>) -> bool {
        self.rules.iter().all(|rule| match rule.logic {
            LogicalOp::And => rule.conditions.iter().all(|cond| {
                if let Some(value) = customer_data.get(&cond.field) {
                    cond.evaluate(*value)
                } else {
                    false
                }
            }),
            LogicalOp::Or => rule.conditions.iter().any(|cond| {
                if let Some(value) = customer_data.get(&cond.field) {
                    cond.evaluate(*value)
                } else {
                    false
                }
            }),
            LogicalOp::Not => !rule.conditions.iter().any(|cond| {
                if let Some(value) = customer_data.get(&cond.field) {
                    cond.evaluate(*value)
                } else {
                    false
                }
            }),
        })
    }
}

/// Behavioral segmentation engine
pub struct BehavioralSegmenter {
    segments: Vec<BehavioralSegment>,
}

impl BehavioralSegmenter {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn add_segment(mut self, segment: BehavioralSegment) -> Self {
        self.segments.push(segment);
        self
    }

    pub fn classify(&self, customer_data: &HashMap<String, f64>) -> Result<Vec<String>> {
        let mut matched = Vec::new();
        for segment in &self.segments {
            if segment.matches(customer_data) {
                matched.push(segment.name.clone());
            }
        }
        Ok(matched)
    }

    pub fn classify_primary(&self, customer_data: &HashMap<String, f64>) -> Result<Option<String>> {
        Ok(self
            .segments
            .iter()
            .filter(|s| s.matches(customer_data))
            .max_by_key(|s| s.priority)
            .map(|s| s.name.clone()))
    }

    pub fn get_segment_sql(&self, name: &str) -> Result<Option<String>> {
        Ok(self
            .segments
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.to_sql()))
    }

    pub fn export_sql(&self) -> Result<String> {
        let mut sql = String::new();
        for segment in &self.segments {
            sql.push_str(&format!(
                "-- Segment: {}\nSELECT * FROM customers WHERE {};\n\n",
                segment.name,
                segment.to_sql()
            ));
        }
        Ok(sql)
    }
}

impl Default for BehavioralSegmenter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_evaluation() {
        let cond = Condition::new("spend", ComparisonOp::GreaterThan, RuleValue::Number(100.0));
        assert!(cond.evaluate(150.0));
        assert!(!cond.evaluate(50.0));
    }

    #[test]
    fn test_condition_to_sql() {
        let cond = Condition::new("spend", ComparisonOp::GreaterThan, RuleValue::Number(100.0));
        let sql = cond.to_sql();
        assert!(sql.contains("spend"));
        assert!(sql.contains(">"));
    }

    #[test]
    fn test_rule_with_aggregate() {
        let cond = Condition::new(
            "purchase_amount",
            ComparisonOp::GreaterThan,
            RuleValue::Number(1000.0),
        )
        .with_aggregate(AggregateFunction::Sum);
        let sql = cond.to_sql();
        assert!(sql.contains("SUM"));
    }

    #[test]
    fn test_rule_with_time_window() {
        let cond = Condition::new(
            "last_purchase",
            ComparisonOp::LessThan,
            RuleValue::Number(30.0),
        )
        .with_time_window(90);
        let sql = cond.to_sql();
        assert!(sql.contains("last 90 days"));
    }

    #[test]
    fn test_behavioral_rule_and_logic() {
        let rule = BehavioralRule::new("high_spender", "Customers spending > 1000")
            .add_condition(Condition::new(
                "total_spend",
                ComparisonOp::GreaterThan,
                RuleValue::Number(1000.0),
            ))
            .add_condition(Condition::new(
                "purchase_count",
                ComparisonOp::GreaterThan,
                RuleValue::Number(5.0),
            ))
            .with_logic(LogicalOp::And);

        let sql = rule.to_sql();
        assert!(sql.contains("AND"));
    }

    #[test]
    fn test_behavioral_segment_matching() {
        let rule = BehavioralRule::new("high_spender", "High spending customers")
            .add_condition(Condition::new(
                "spend",
                ComparisonOp::GreaterThan,
                RuleValue::Number(100.0),
            ))
            .with_logic(LogicalOp::And);

        let segment = BehavioralSegment::new("VIP", "VIP Customers").add_rule(rule);

        let mut customer1 = HashMap::new();
        customer1.insert("spend".to_string(), 150.0);
        assert!(segment.matches(&customer1));

        let mut customer2 = HashMap::new();
        customer2.insert("spend".to_string(), 50.0);
        assert!(!segment.matches(&customer2));
    }

    #[test]
    fn test_segmenter_classify() {
        let rule = BehavioralRule::new("high_spender", "High spending")
            .add_condition(Condition::new(
                "spend",
                ComparisonOp::GreaterThan,
                RuleValue::Number(100.0),
            ))
            .with_logic(LogicalOp::And);

        let segment = BehavioralSegment::new("VIP", "VIP")
            .add_rule(rule)
            .with_priority(1);

        let segmenter = BehavioralSegmenter::new().add_segment(segment);

        let mut customer = HashMap::new();
        customer.insert("spend".to_string(), 150.0);

        let result = segmenter.classify(&customer).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "VIP");
    }

    #[test]
    fn test_segment_sql_generation() {
        let rule = BehavioralRule::new("active", "Active customers")
            .add_condition(Condition::new(
                "days_since_purchase",
                ComparisonOp::LessThan,
                RuleValue::Number(30.0),
            ))
            .with_logic(LogicalOp::And);

        let segment = BehavioralSegment::new("Active", "Active Customers").add_rule(rule);

        let sql = segment.to_sql();
        assert!(sql.contains("days_since_purchase"));
        assert!(sql.contains("<"));
    }

    #[test]
    fn test_comparison_operators() {
        assert_eq!(ComparisonOp::GreaterThan.to_string(), ">");
        assert_eq!(ComparisonOp::Equal.to_string(), "=");
        assert_eq!(ComparisonOp::In.to_string(), "IN");
    }

    #[test]
    fn test_logical_operators() {
        assert_eq!(LogicalOp::And.to_string(), "AND");
        assert_eq!(LogicalOp::Or.to_string(), "OR");
        assert_eq!(LogicalOp::Not.to_string(), "NOT");
    }
}
