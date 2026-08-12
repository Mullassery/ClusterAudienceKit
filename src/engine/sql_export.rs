//! SQL export module for generating warehouse-native queries from segment definitions
//!
//! Supports 8 SQL dialects:
//! - ANSI SQL (baseline standard)
//! - Snowflake
//! - BigQuery
//! - Redshift
//! - PostgreSQL
//! - Oracle
//! - SQL Server
//! - MySQL

use crate::Result;
use std::collections::HashMap;

/// Supported SQL dialects
#[derive(Clone, Debug, Eq, PartialEq, Hash, Copy)]
pub enum SQLDialect {
    Ansi,
    Snowflake,
    BigQuery,
    Redshift,
    PostgreSQL,
    Oracle,
    SqlServer,
    MySQL,
}

impl SQLDialect {
    pub fn as_str(&self) -> &str {
        match self {
            SQLDialect::Ansi => "ansi",
            SQLDialect::Snowflake => "snowflake",
            SQLDialect::BigQuery => "bigquery",
            SQLDialect::Redshift => "redshift",
            SQLDialect::PostgreSQL => "postgresql",
            SQLDialect::Oracle => "oracle",
            SQLDialect::SqlServer => "sqlserver",
            SQLDialect::MySQL => "mysql",
        }
    }

    // Named `from_str` (not the `std::str::FromStr` trait) to match the
    // simple `Option`-returning style already used throughout this codebase
    // and its existing call sites/tests; implementing the real trait would
    // be a larger, purely cosmetic churn for no behavior change.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ansi" => Some(SQLDialect::Ansi),
            "snowflake" => Some(SQLDialect::Snowflake),
            "bigquery" => Some(SQLDialect::BigQuery),
            "redshift" => Some(SQLDialect::Redshift),
            "postgresql" | "postgres" => Some(SQLDialect::PostgreSQL),
            "oracle" => Some(SQLDialect::Oracle),
            "sqlserver" | "mssql" | "sql_server" => Some(SQLDialect::SqlServer),
            "mysql" => Some(SQLDialect::MySQL),
            _ => None,
        }
    }
}

/// RFM score pattern for a segment
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct RFMPattern {
    recency: u32,
    frequency: u32,
    monetary: u32,
}

/// Segment definition with RFM patterns
#[derive(Clone, Debug)]
pub struct SegmentDefinition {
    pub name: String,
    pub description: String,
    patterns: Vec<RFMPattern>,
}

/// SQL query column mapping
#[derive(Clone, Debug)]
pub struct ColumnMapping {
    pub customer_id: String,
    pub recency_score: String,
    pub frequency_score: String,
    pub monetary_score: String,
}

impl Default for ColumnMapping {
    fn default() -> Self {
        Self {
            customer_id: "customer_id".to_string(),
            recency_score: "recency_score".to_string(),
            frequency_score: "frequency_score".to_string(),
            monetary_score: "monetary_score".to_string(),
        }
    }
}

/// Generate segment definitions from RFM classification rules
fn get_segment_definitions() -> HashMap<String, SegmentDefinition> {
    let mut segments = HashMap::new();

    // Champions: Top performers across all dimensions
    segments.insert(
        "Champions".to_string(),
        SegmentDefinition {
            name: "Champions".to_string(),
            description: "Best customers with highest value and engagement".to_string(),
            patterns: vec![
                RFMPattern {
                    recency: 5,
                    frequency: 5,
                    monetary: 5,
                },
                RFMPattern {
                    recency: 5,
                    frequency: 5,
                    monetary: 4,
                },
                RFMPattern {
                    recency: 5,
                    frequency: 4,
                    monetary: 5,
                },
                RFMPattern {
                    recency: 4,
                    frequency: 5,
                    monetary: 5,
                },
            ],
        },
    );

    // VIP: High monetary, very recent activity
    segments.insert(
        "VIP".to_string(),
        SegmentDefinition {
            name: "VIP".to_string(),
            description: "Premium customers with exceptional value metrics".to_string(),
            patterns: vec![
                RFMPattern {
                    recency: 5,
                    frequency: 3,
                    monetary: 5,
                },
                RFMPattern {
                    recency: 5,
                    frequency: 2,
                    monetary: 5,
                },
                RFMPattern {
                    recency: 5,
                    frequency: 1,
                    monetary: 5,
                },
            ],
        },
    );

    // Loyal Customers: Consistent across all dimensions
    segments.insert(
        "LoyalCustomers".to_string(),
        SegmentDefinition {
            name: "Loyal Customers".to_string(),
            description: "Consistent, valuable customers with steady engagement".to_string(),
            patterns: vec![
                RFMPattern {
                    recency: 4,
                    frequency: 4,
                    monetary: 4,
                },
                RFMPattern {
                    recency: 4,
                    frequency: 4,
                    monetary: 5,
                },
                RFMPattern {
                    recency: 4,
                    frequency: 5,
                    monetary: 4,
                },
            ],
        },
    );

    // Potential Loyalists: High monetary + recent but lower frequency
    segments.insert(
        "PotentialLoyalists".to_string(),
        SegmentDefinition {
            name: "Potential Loyalists".to_string(),
            description: "Recent high-value customers with growth potential".to_string(),
            patterns: vec![
                RFMPattern {
                    recency: 5,
                    frequency: 3,
                    monetary: 4,
                },
                RFMPattern {
                    recency: 4,
                    frequency: 3,
                    monetary: 5,
                },
            ],
        },
    );

    // Cannot Lose Them: High monetary but declining engagement
    segments.insert(
        "CannotLose".to_string(),
        SegmentDefinition {
            name: "Cannot Lose Them".to_string(),
            description: "High-value customers at risk of churn - critical retention".to_string(),
            patterns: vec![
                RFMPattern {
                    recency: 3,
                    frequency: 4,
                    monetary: 5,
                },
                RFMPattern {
                    recency: 3,
                    frequency: 5,
                    monetary: 5,
                },
                RFMPattern {
                    recency: 2,
                    frequency: 4,
                    monetary: 5,
                },
                RFMPattern {
                    recency: 2,
                    frequency: 5,
                    monetary: 5,
                },
            ],
        },
    );

    // At Risk: Used to be good but now declining
    segments.insert(
        "AtRisk".to_string(),
        SegmentDefinition {
            name: "At Risk".to_string(),
            description: "Once valuable customers showing declining engagement".to_string(),
            patterns: vec![
                RFMPattern {
                    recency: 2,
                    frequency: 2,
                    monetary: 4,
                },
                RFMPattern {
                    recency: 2,
                    frequency: 2,
                    monetary: 5,
                },
                RFMPattern {
                    recency: 2,
                    frequency: 3,
                    monetary: 4,
                },
                RFMPattern {
                    recency: 2,
                    frequency: 3,
                    monetary: 5,
                },
                RFMPattern {
                    recency: 3,
                    frequency: 2,
                    monetary: 4,
                },
                RFMPattern {
                    recency: 3,
                    frequency: 2,
                    monetary: 5,
                },
            ],
        },
    );

    // About to Sleep: Low recency + moderate frequency
    segments.insert(
        "AboutToSleep".to_string(),
        SegmentDefinition {
            name: "About to Sleep".to_string(),
            description: "Low engagement customers at risk of dormancy".to_string(),
            patterns: vec![
                RFMPattern {
                    recency: 2,
                    frequency: 2,
                    monetary: 3,
                },
                RFMPattern {
                    recency: 2,
                    frequency: 2,
                    monetary: 2,
                },
                RFMPattern {
                    recency: 2,
                    frequency: 3,
                    monetary: 2,
                },
                RFMPattern {
                    recency: 2,
                    frequency: 3,
                    monetary: 3,
                },
            ],
        },
    );

    // New Customers: High frequency + very recent but lower monetary
    segments.insert(
        "NewCustomers".to_string(),
        SegmentDefinition {
            name: "New Customers".to_string(),
            description: "Recently acquired customers with high potential".to_string(),
            patterns: vec![
                RFMPattern {
                    recency: 5,
                    frequency: 4,
                    monetary: 3,
                },
                RFMPattern {
                    recency: 5,
                    frequency: 4,
                    monetary: 2,
                },
                RFMPattern {
                    recency: 5,
                    frequency: 3,
                    monetary: 3,
                },
                RFMPattern {
                    recency: 5,
                    frequency: 3,
                    monetary: 2,
                },
            ],
        },
    );

    // Promising: Recent + moderate frequency + moderate monetary
    segments.insert(
        "Promising".to_string(),
        SegmentDefinition {
            name: "Promising".to_string(),
            description: "New customers showing early signs of high value".to_string(),
            patterns: vec![
                RFMPattern {
                    recency: 5,
                    frequency: 2,
                    monetary: 3,
                },
                RFMPattern {
                    recency: 5,
                    frequency: 2,
                    monetary: 4,
                },
            ],
        },
    );

    // Need Attention: Low frequency despite recent activity
    segments.insert(
        "NeedAttention".to_string(),
        SegmentDefinition {
            name: "Need Attention".to_string(),
            description: "Moderate-value customers requiring engagement".to_string(),
            patterns: vec![
                RFMPattern {
                    recency: 4,
                    frequency: 2,
                    monetary: 2,
                },
                RFMPattern {
                    recency: 4,
                    frequency: 3,
                    monetary: 2,
                },
                RFMPattern {
                    recency: 4,
                    frequency: 3,
                    monetary: 3,
                },
            ],
        },
    );

    // Lost: Low across all dimensions
    segments.insert(
        "Lost".to_string(),
        SegmentDefinition {
            name: "Lost".to_string(),
            description: "Inactive customers unlikely to return".to_string(),
            patterns: vec![
                RFMPattern {
                    recency: 1,
                    frequency: 1,
                    monetary: 1,
                },
                RFMPattern {
                    recency: 1,
                    frequency: 1,
                    monetary: 2,
                },
                RFMPattern {
                    recency: 1,
                    frequency: 2,
                    monetary: 1,
                },
            ],
        },
    );

    // At Risk - Sleeping: Used to purchase but now gone cold
    segments.insert(
        "AtRiskSleeping".to_string(),
        SegmentDefinition {
            name: "At Risk - Sleeping".to_string(),
            description: "Previously inactive customers showing re-engagement".to_string(),
            patterns: vec![
                RFMPattern {
                    recency: 1,
                    frequency: 3,
                    monetary: 4,
                },
                RFMPattern {
                    recency: 1,
                    frequency: 3,
                    monetary: 5,
                },
                RFMPattern {
                    recency: 1,
                    frequency: 4,
                    monetary: 4,
                },
                RFMPattern {
                    recency: 1,
                    frequency: 4,
                    monetary: 5,
                },
            ],
        },
    );

    // Hibernating: Very low recency but used to be valuable
    segments.insert(
        "Hibernating".to_string(),
        SegmentDefinition {
            name: "Hibernating".to_string(),
            description: "Inactive but potentially recoverable customers".to_string(),
            patterns: vec![
                RFMPattern {
                    recency: 1,
                    frequency: 2,
                    monetary: 3,
                },
                RFMPattern {
                    recency: 1,
                    frequency: 2,
                    monetary: 4,
                },
                RFMPattern {
                    recency: 1,
                    frequency: 2,
                    monetary: 5,
                },
                RFMPattern {
                    recency: 1,
                    frequency: 3,
                    monetary: 2,
                },
                RFMPattern {
                    recency: 1,
                    frequency: 3,
                    monetary: 3,
                },
            ],
        },
    );

    segments
}

/// Validate a SQL identifier (table name or column name) against a strict
/// allow-list before it's interpolated into a query string. Table names may
/// be schema-qualified with dots (e.g. `project.dataset.customers` for
/// BigQuery); each dot-separated segment must otherwise be alphanumeric or
/// underscore, and may not start with a digit. This closes a SQL-injection
/// vector: `table_name` and the column names in `ColumnMapping` previously
/// flowed straight into `format!()`-built SQL with no validation, so a
/// caller-supplied value like `customers; DROP TABLE users; --` would have
/// been emitted verbatim.
fn validate_identifier(identifier: &str) -> Result<()> {
    if identifier.is_empty() {
        return Err(crate::ClusterClusterAudienceKitError::InvalidConfig(
            "SQL identifier cannot be empty".to_string(),
        ));
    }

    for part in identifier.split('.') {
        let mut chars = part.chars();
        let valid = match chars.next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
            }
            _ => false,
        };
        if !valid {
            return Err(crate::ClusterClusterAudienceKitError::InvalidConfig(
                format!(
                    "Invalid SQL identifier '{}': identifiers must be alphanumeric/underscore \
                 (optionally dot-qualified) and cannot start with a digit",
                    identifier
                ),
            ));
        }
    }

    Ok(())
}

fn validate_column_mapping(mapping: &ColumnMapping) -> Result<()> {
    validate_identifier(&mapping.customer_id)?;
    validate_identifier(&mapping.recency_score)?;
    validate_identifier(&mapping.frequency_score)?;
    validate_identifier(&mapping.monetary_score)?;
    Ok(())
}

/// SQL query builder
pub struct SQLExporter;

impl SQLExporter {
    /// Generate SQL query for a specific segment
    pub fn export_segment(
        segment_name: &str,
        dialect: SQLDialect,
        table_name: &str,
        column_mapping: &ColumnMapping,
    ) -> Result<String> {
        validate_identifier(table_name)?;
        validate_column_mapping(column_mapping)?;

        let definitions = get_segment_definitions();

        let segment = definitions.get(segment_name).ok_or_else(|| {
            let msg = format!(
                "Segment '{}' not found. Available segments: {:?}",
                segment_name,
                definitions.keys().collect::<Vec<_>>()
            );
            crate::ClusterClusterAudienceKitError::InvalidConfig(msg)
        })?;

        let where_clause = Self::build_where_clause(&segment.patterns, column_mapping, dialect);
        let query = Self::build_query(
            table_name,
            &column_mapping.customer_id,
            segment_name,
            &where_clause,
            dialect,
        );

        Ok(query)
    }

    /// Generate SQL queries for all segments
    pub fn export_all_segments(
        dialect: SQLDialect,
        table_name: &str,
        column_mapping: &ColumnMapping,
    ) -> Result<HashMap<String, String>> {
        let definitions = get_segment_definitions();
        let mut queries = HashMap::new();

        for key in definitions.keys() {
            let query = Self::export_segment(key, dialect, table_name, column_mapping)?;
            queries.insert(key.clone(), query);
        }

        Ok(queries)
    }

    /// Get list of supported dialects
    pub fn supported_dialects() -> Vec<&'static str> {
        vec![
            "ansi",
            "snowflake",
            "bigquery",
            "redshift",
            "postgresql",
            "oracle",
            "sqlserver",
            "mysql",
        ]
    }

    /// Get segment pattern details
    pub fn get_segment_patterns(segment_name: &str) -> Result<Vec<(u32, u32, u32)>> {
        let definitions = get_segment_definitions();

        let segment = definitions.get(segment_name).ok_or_else(|| {
            let msg = format!("Segment '{}' not found", segment_name);
            crate::ClusterClusterAudienceKitError::InvalidConfig(msg)
        })?;

        Ok(segment
            .patterns
            .iter()
            .map(|p| (p.recency, p.frequency, p.monetary))
            .collect())
    }

    fn build_where_clause(
        patterns: &[RFMPattern],
        mapping: &ColumnMapping,
        dialect: SQLDialect,
    ) -> String {
        let r = &mapping.recency_score;
        let f = &mapping.frequency_score;
        let m = &mapping.monetary_score;

        match dialect {
            SQLDialect::Snowflake | SQLDialect::BigQuery | SQLDialect::Ansi => {
                // Standard: Use OR conditions
                let conditions: Vec<String> = patterns
                    .iter()
                    .map(|p| {
                        format!(
                            "({} = {} AND {} = {} AND {} = {})",
                            r, p.recency, f, p.frequency, m, p.monetary
                        )
                    })
                    .collect();
                conditions.join(" OR ")
            }
            SQLDialect::Redshift | SQLDialect::PostgreSQL => {
                // PostgreSQL-based: Support IN clause optimization
                Self::optimize_patterns_in_clause(patterns, mapping)
            }
            SQLDialect::Oracle => {
                // Oracle: Use IN clause with composite conditions
                let conditions: Vec<String> = patterns
                    .iter()
                    .map(|p| {
                        format!(
                            "({}={} AND {}={} AND {}={})",
                            r, p.recency, f, p.frequency, m, p.monetary
                        )
                    })
                    .collect();
                conditions.join(" OR ")
            }
            SQLDialect::SqlServer => {
                // SQL Server: Optimize with IN for monetary when possible
                Self::optimize_patterns_in_clause(patterns, mapping)
            }
            SQLDialect::MySQL => {
                // MySQL: Simple OR conditions
                let conditions: Vec<String> = patterns
                    .iter()
                    .map(|p| {
                        format!(
                            "({} = {} AND {} = {} AND {} = {})",
                            r, p.recency, f, p.frequency, m, p.monetary
                        )
                    })
                    .collect();
                conditions.join(" OR ")
            }
        }
    }

    fn optimize_patterns_in_clause(patterns: &[RFMPattern], mapping: &ColumnMapping) -> String {
        let r = &mapping.recency_score;
        let f = &mapping.frequency_score;
        let m = &mapping.monetary_score;

        let mut grouped: HashMap<(u32, u32), Vec<u32>> = HashMap::new();
        for p in patterns {
            grouped
                .entry((p.recency, p.frequency))
                .or_default()
                .push(p.monetary);
        }

        let conditions: Vec<String> = grouped
            .iter()
            .map(|((rec, freq), monetaries)| {
                if monetaries.len() > 1 {
                    format!(
                        "({} = {} AND {} = {} AND {} IN ({}))",
                        r,
                        rec,
                        f,
                        freq,
                        m,
                        monetaries
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                } else {
                    format!(
                        "({} = {} AND {} = {} AND {} = {})",
                        r, rec, f, freq, m, monetaries[0]
                    )
                }
            })
            .collect();

        conditions.join(" OR ")
    }

    fn build_query(
        table_name: &str,
        customer_id_col: &str,
        segment_name: &str,
        where_clause: &str,
        dialect: SQLDialect,
    ) -> String {
        let select_part = match dialect {
            SQLDialect::BigQuery => format!(
                "SELECT {} AS customer_id, '{}' AS segment",
                customer_id_col, segment_name
            ),
            _ => format!("SELECT {}, '{}' AS segment", customer_id_col, segment_name),
        };

        format!(
            "{}\nFROM {}\nWHERE {};",
            select_part, table_name, where_clause
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_dialects() {
        let dialects = SQLExporter::supported_dialects();
        assert_eq!(dialects.len(), 8);
        assert!(dialects.contains(&"snowflake"));
        assert!(dialects.contains(&"bigquery"));
    }

    #[test]
    fn test_sql_dialect_from_str() {
        assert_eq!(
            SQLDialect::from_str("snowflake"),
            Some(SQLDialect::Snowflake)
        );
        assert_eq!(SQLDialect::from_str("bigquery"), Some(SQLDialect::BigQuery));
        assert_eq!(
            SQLDialect::from_str("postgresql"),
            Some(SQLDialect::PostgreSQL)
        );
        assert_eq!(
            SQLDialect::from_str("postgres"),
            Some(SQLDialect::PostgreSQL)
        );
    }

    #[test]
    fn test_export_champions_ansi() {
        let mapping = ColumnMapping::default();
        let sql = SQLExporter::export_segment("Champions", SQLDialect::Ansi, "customers", &mapping)
            .unwrap();

        assert!(sql.contains("Champions"));
        assert!(sql.contains("FROM customers"));
        assert!(sql.contains("WHERE"));
        assert!(sql.contains("recency_score = 5"));
        assert!(sql.contains("frequency_score = 5"));
    }

    #[test]
    fn test_export_champions_snowflake() {
        let mapping = ColumnMapping::default();
        let sql =
            SQLExporter::export_segment("Champions", SQLDialect::Snowflake, "customers", &mapping)
                .unwrap();

        assert!(sql.contains("Champions"));
        assert!(sql.contains("customers"));
    }

    #[test]
    fn test_export_champions_bigquery() {
        let mapping = ColumnMapping::default();
        let sql = SQLExporter::export_segment(
            "Champions",
            SQLDialect::BigQuery,
            "project.dataset.customers",
            &mapping,
        )
        .unwrap();

        assert!(sql.contains("Champions"));
        assert!(sql.contains("project.dataset.customers"));
    }

    #[test]
    fn test_export_lost_segment() {
        let mapping = ColumnMapping::default();
        let sql =
            SQLExporter::export_segment("Lost", SQLDialect::Ansi, "customers", &mapping).unwrap();

        assert!(sql.contains("Lost"));
        assert!(sql.contains("recency_score = 1"));
    }

    #[test]
    fn test_all_dialects() {
        let mapping = ColumnMapping::default();
        for dialect in &[
            SQLDialect::Ansi,
            SQLDialect::Snowflake,
            SQLDialect::BigQuery,
            SQLDialect::Redshift,
            SQLDialect::PostgreSQL,
            SQLDialect::Oracle,
            SQLDialect::SqlServer,
            SQLDialect::MySQL,
        ] {
            let sql = SQLExporter::export_segment("Champions", *dialect, "customers", &mapping)
                .unwrap_or_else(|_| panic!("Failed to generate SQL for {:?}", dialect));

            assert!(!sql.is_empty());
            assert!(sql.contains("Champions"));
        }
    }

    #[test]
    fn test_custom_column_mapping() {
        let mapping = ColumnMapping {
            customer_id: "cust_id".to_string(),
            recency_score: "r_score".to_string(),
            frequency_score: "f_score".to_string(),
            monetary_score: "m_score".to_string(),
        };

        let sql = SQLExporter::export_segment(
            "Champions",
            SQLDialect::Ansi,
            "customer_segments",
            &mapping,
        )
        .unwrap();

        assert!(sql.contains("cust_id"));
        assert!(sql.contains("r_score"));
        assert!(sql.contains("f_score"));
        assert!(sql.contains("m_score"));
    }

    #[test]
    fn test_get_segment_patterns() {
        let patterns = SQLExporter::get_segment_patterns("Champions").unwrap();
        assert_eq!(patterns.len(), 4);
        assert!(patterns.contains(&(5, 5, 5)));
    }

    #[test]
    fn test_export_all_segments() {
        let mapping = ColumnMapping::default();
        let queries =
            SQLExporter::export_all_segments(SQLDialect::Snowflake, "customers", &mapping).unwrap();

        assert_eq!(queries.len(), 13);
        assert!(queries.contains_key("Champions"));
        assert!(queries.contains_key("Lost"));
    }

    #[test]
    fn test_rejects_sql_injection_in_table_name() {
        let mapping = ColumnMapping::default();
        let result = SQLExporter::export_segment(
            "Champions",
            SQLDialect::Ansi,
            "customers; DROP TABLE users; --",
            &mapping,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_rejects_sql_injection_in_column_mapping() {
        let mapping = ColumnMapping {
            customer_id: "customer_id".to_string(),
            recency_score: "r_score' OR '1'='1".to_string(),
            frequency_score: "frequency_score".to_string(),
            monetary_score: "monetary_score".to_string(),
        };
        let result =
            SQLExporter::export_segment("Champions", SQLDialect::Ansi, "customers", &mapping);
        assert!(result.is_err());
    }

    #[test]
    fn test_accepts_schema_qualified_table_name() {
        // BigQuery-style dot-qualified table names must still work.
        let mapping = ColumnMapping::default();
        let result = SQLExporter::export_segment(
            "Champions",
            SQLDialect::BigQuery,
            "project.dataset.customers",
            &mapping,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_rejects_empty_table_name() {
        let mapping = ColumnMapping::default();
        let result = SQLExporter::export_segment("Champions", SQLDialect::Ansi, "", &mapping);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_segment() {
        let mapping = ColumnMapping::default();
        let result =
            SQLExporter::export_segment("InvalidSegment", SQLDialect::Ansi, "customers", &mapping);

        assert!(result.is_err());
    }
}
