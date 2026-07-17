#!/usr/bin/env python3
"""
SQL Export Examples for ClusterAudienceKit

This script demonstrates how to export customer segments as SQL queries
for 8 different warehouse dialects.
"""

from clusteraudiencekit import (
    export_segment_sql,
    export_all_segments_sql,
    get_supported_sql_dialects,
    get_segment_rfm_patterns,
)


def example_1_basic_export():
    """Example 1: Basic segment export for a single dialect."""
    print("=" * 70)
    print("Example 1: Basic Segment Export")
    print("=" * 70)

    sql = export_segment_sql(
        segment_name="Champions",
        dialect="snowflake",
        table_name="analytics.customers"
    )

    print("\nSnowflake SQL for Champions segment:")
    print(sql)
    print()


def example_2_all_dialects():
    """Example 2: Export same segment for all supported dialects."""
    print("=" * 70)
    print("Example 2: Export for All Dialects")
    print("=" * 70)

    dialects = get_supported_sql_dialects()
    segment = "AtRisk"

    print(f"\nExporting '{segment}' segment for all {len(dialects)} dialects:\n")

    for dialect in dialects:
        sql = export_segment_sql(segment, dialect, "customers")
        # Show just the first line to keep output concise
        first_line = sql.split("\n")[0]
        print(f"  {dialect.upper():12} → {first_line}")

    print()


def example_3_custom_columns():
    """Example 3: Export with custom column names."""
    print("=" * 70)
    print("Example 3: Custom Column Mapping")
    print("=" * 70)

    sql = export_segment_sql(
        segment_name="Champions",
        dialect="bigquery",
        table_name="project.dataset.customers",
        customer_id="customer_uuid",
        recency_score="days_since_purchase",
        frequency_score="purchase_count",
        monetary_score="total_spend"
    )

    print("\nBigQuery SQL with custom columns:")
    print(sql)
    print()


def example_4_batch_export():
    """Example 4: Export all 13 segments at once."""
    print("=" * 70)
    print("Example 4: Batch Export All Segments")
    print("=" * 70)

    queries = export_all_segments_sql("snowflake", "raw_data.customers")

    print(f"\nExported {len(queries)} segments for Snowflake:")
    print("\nSegments generated:")
    for segment_name in sorted(queries.keys()):
        print(f"  - {segment_name}")

    print("\nExample: Champions segment SQL:")
    print(queries["Champions"][:200] + "...\n")


def example_5_rfm_patterns():
    """Example 5: Inspect RFM patterns for segments."""
    print("=" * 70)
    print("Example 5: RFM Patterns for Segments")
    print("=" * 70)

    segments_to_inspect = ["Champions", "AtRisk", "Lost", "Promising"]

    for segment in segments_to_inspect:
        patterns = get_segment_rfm_patterns(segment)
        print(f"\n{segment}:")
        print(f"  Patterns (R, F, M): {patterns}")


def example_6_production_workflow():
    """Example 6: Production workflow - batch create segment views."""
    print("=" * 70)
    print("Example 6: Production Workflow")
    print("=" * 70)

    queries = export_all_segments_sql("snowflake", "analytics.customer_data")

    print("\nGenerated SQL to create segment views in Snowflake:\n")
    print("-- Create segment views in Snowflake")
    print("USE DATABASE analytics;\n")

    for segment_name, sql in sorted(queries.items()):
        view_name = f"vw_segment_{segment_name.lower()}"
        print(f"CREATE OR REPLACE VIEW {view_name} AS")
        print(f"{sql}\n")


def example_7_dialect_specific():
    """Example 7: Dialect-specific features."""
    print("=" * 70)
    print("Example 7: Dialect-Specific Optimizations")
    print("=" * 70)

    segment = "Champions"
    table = "customers"

    # Compare how different dialects optimize the same segment
    dialects_to_compare = ["ansi", "postgresql", "sqlserver", "oracle"]

    print(f"\nHow different dialects optimize the '{segment}' segment:\n")

    for dialect in dialects_to_compare:
        sql = export_segment_sql(segment, dialect, table)
        # Extract just the WHERE clause for comparison
        where_start = sql.find("WHERE")
        where_clause = sql[where_start:where_start+150] + "..."
        print(f"{dialect.upper():10} → {where_clause}")

    print()


def example_8_large_scale_export():
    """Example 8: Export queries for data warehouse automation."""
    print("=" * 70)
    print("Example 8: Large-Scale Export for Automation")
    print("=" * 70)

    # This example shows how to export all segments and prepare them for loading

    warehouse_config = {
        "snowflake": {
            "database": "ANALYTICS",
            "schema": "SEGMENTS",
            "table": "RAW_CUSTOMERS"
        },
        "bigquery": {
            "project": "my-project",
            "dataset": "analytics",
            "table": "customers"
        }
    }

    print("\nExporting for multiple warehouses:\n")

    for warehouse, config in warehouse_config.items():
        if warehouse == "snowflake":
            table_name = f"{config['database']}.{config['schema']}.{config['table']}"
        else:  # bigquery
            table_name = f"{config['project']}.{config['dataset']}.{config['table']}"

        queries = export_all_segments_sql(warehouse, table_name)

        print(f"{warehouse.upper()}:")
        print(f"  Table: {table_name}")
        print(f"  Segments: {len(queries)}")
        print(f"  Total SQL length: {sum(len(q) for q in queries.values())} chars")
        print()


def example_9_integration_patterns():
    """Example 9: Common integration patterns."""
    print("=" * 70)
    print("Example 9: Integration Patterns")
    print("=" * 70)

    print("\n1. dbt Integration:")
    print("""
    # In your dbt model
    SELECT
        customer_id,
        CASE
            WHEN (<Champions_WHERE_clause>) THEN 'Champions'
            WHEN (<AtRisk_WHERE_clause>) THEN 'At Risk'
            ELSE 'Other'
        END AS segment
    FROM {{ ref('customers') }}
    """)

    print("\n2. Airflow Integration:")
    print("""
    from airflow.operators.sql import SQLExecuteQueryOperator
    from clusteraudiencekit import export_segment_sql

    sql = export_segment_sql('Champions', 'snowflake', 'customers')

    task = SQLExecuteQueryOperator(
        task_id='segment_champions',
        sql=sql,
        conn_id='snowflake_default'
    )
    """)

    print("\n3. SQL-only Integration:")
    print("""
    -- Export once, save as static SQL file
    -- Then use in any tool (dbt, Looker, Mode, etc.)
    """)

    print()


def example_10_error_handling():
    """Example 10: Error handling and validation."""
    print("=" * 70)
    print("Example 10: Error Handling")
    print("=" * 70)

    # Test with invalid segment
    print("\nHandling invalid segment:")
    try:
        sql = export_segment_sql("InvalidSegment", "snowflake", "customers")
    except Exception as e:
        print(f"  Error: {e}")
        print(f"  This is expected behavior")

    # Show valid segments
    print("\nValid segments available:")
    # Get one valid pattern to show available segments
    try:
        patterns = get_segment_rfm_patterns("Champions")
        print("  Champions ✓")
    except:
        pass

    print()


def main():
    """Run all examples."""
    print("\n" + "=" * 70)
    print("ClusterAudienceKit SQL Export Examples")
    print("=" * 70 + "\n")

    examples = [
        example_1_basic_export,
        example_2_all_dialects,
        example_3_custom_columns,
        example_4_batch_export,
        example_5_rfm_patterns,
        example_6_production_workflow,
        example_7_dialect_specific,
        example_8_large_scale_export,
        example_9_integration_patterns,
        example_10_error_handling,
    ]

    for example in examples:
        try:
            example()
        except Exception as e:
            print(f"Error in {example.__name__}: {e}\n")

    print("=" * 70)
    print("All examples completed!")
    print("=" * 70)
    print("\nFor more information, see SQL_EXPORT.md")


if __name__ == "__main__":
    main()
