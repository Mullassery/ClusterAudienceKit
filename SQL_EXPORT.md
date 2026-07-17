# ClusterAudienceKit SQL Export

Export customer segment definitions as native SQL queries for your data warehouse.

## Overview

The SQL export feature converts RFM-based customer segments into executable SQL queries that can run directly in your warehouse without Python round-tripping. This enables:

- **Data warehouse native segmentation** — Apply segments directly in SQL workflows
- **8 SQL dialects supported** — Snowflake, BigQuery, Redshift, PostgreSQL, Oracle, SQL Server, MySQL, ANSI SQL
- **Zero Python dependency** — Run queries directly in your warehouse
- **Automatic optimization** — Dialect-specific optimizations (IN clauses, function usage, etc.)
- **Custom column mapping** — Map to your table's actual column names

## Quick Start

```python
from clusteraudiencekit import export_segment_sql

# Export Champions segment for Snowflake
sql = export_segment_sql(
    segment_name="Champions",
    dialect="snowflake",
    table_name="customer_data",
    customer_id="customer_id",
    recency_score="rfm_recency",
    frequency_score="rfm_frequency",
    monetary_score="rfm_monetary"
)

print(sql)
# SELECT customer_id, 'Champions' AS segment
# FROM customer_data
# WHERE (rfm_recency = 5 AND rfm_frequency = 5 AND rfm_monetary = 5) 
#    OR (rfm_recency = 5 AND rfm_frequency = 5 AND rfm_monetary = 4)
#    ...
```

## Supported Dialects

### 1. ANSI SQL (Baseline)
Standard SQL, compatible with most warehouses:
```sql
SELECT customer_id, 'Champions' AS segment
FROM customers
WHERE (recency_score = 5 AND frequency_score = 5 AND monetary_score = 5) 
   OR (recency_score = 5 AND frequency_score = 5 AND monetary_score = 4)
   ...;
```

### 2. Snowflake
Fully optimized for Snowflake syntax:
```sql
SELECT customer_id, 'Champions' AS segment
FROM customers
WHERE (recency_score = 5 AND frequency_score = 5 AND monetary_score = 5) 
   OR (recency_score = 5 AND frequency_score = 5 AND monetary_score = 4)
   ...;
```

### 3. BigQuery
Google BigQuery optimized syntax:
```sql
SELECT customer_id AS customer_id, 'Champions' AS segment
FROM customers
WHERE (recency_score = 5 AND frequency_score = 5 AND monetary_score = 5)
   OR (recency_score = 5 AND frequency_score = 5 AND monetary_score = 4)
   ...;
```

### 4. Redshift
Amazon Redshift optimized queries with IN clause optimization:
```sql
SELECT customer_id, 'Champions' AS segment
FROM customers
WHERE (recency_score = 5 AND frequency_score = 5 AND monetary_score IN (5, 4))
   OR (recency_score = 5 AND frequency_score = 4 AND monetary_score = 5)
   ...;
```

### 5. PostgreSQL
PostgreSQL with IN clause optimization:
```sql
SELECT customer_id, 'Champions' AS segment
FROM customers
WHERE (recency_score = 5 AND frequency_score = 5 AND monetary_score IN (5, 4))
   OR (recency_score = 5 AND frequency_score = 4 AND monetary_score = 5)
   ...;
```

### 6. Oracle
Oracle SQL specific syntax:
```sql
SELECT customer_id, 'Champions' AS segment
FROM customers
WHERE (recency_score=5 AND frequency_score=5 AND monetary_score=5)
   OR (recency_score=5 AND frequency_score=5 AND monetary_score=4)
   ...;
```

### 7. SQL Server
T-SQL optimized with IN clause support:
```sql
SELECT customer_id, 'Champions' AS segment
FROM customers
WHERE (recency_score = 5 AND frequency_score = 5 AND monetary_score IN (5, 4))
   OR (recency_score = 5 AND frequency_score = 4 AND monetary_score = 5)
   ...;
```

### 8. MySQL
MySQL compatible SQL:
```sql
SELECT customer_id, 'Champions' AS segment
FROM customers
WHERE (recency_score = 5 AND frequency_score = 5 AND monetary_score = 5) 
   OR (recency_score = 5 AND frequency_score = 5 AND monetary_score = 4)
   ...;
```

## API Reference

### `export_segment_sql(segment_name, dialect, table_name, **kwargs)`

Export a single segment as SQL query.

**Parameters:**
- `segment_name` (str): Segment name (e.g., "Champions", "AtRisk", "Lost")
- `dialect` (str): SQL dialect ("ansi", "snowflake", "bigquery", "redshift", "postgresql", "oracle", "sqlserver", "mysql")
- `table_name` (str): Table name in your warehouse (e.g., "customers", "schema.table")
- `customer_id` (str, optional): Customer ID column name (default: "customer_id")
- `recency_score` (str, optional): Recency score column name (default: "recency_score")
- `frequency_score` (str, optional): Frequency score column name (default: "frequency_score")
- `monetary_score` (str, optional): Monetary score column name (default: "monetary_score")

**Returns:** SQL query string

### `export_all_segments_sql(dialect, table_name, **kwargs)`

Export all 13 segments as SQL queries.

**Parameters:**
- `dialect` (str): SQL dialect
- `table_name` (str): Table name
- `**kwargs`: Same column mapping options as `export_segment_sql`

**Returns:** Dict[segment_name, sql_query]

### `get_supported_sql_dialects()`

Get list of supported SQL dialects.

**Returns:** List[str] of dialect names

### `get_segment_rfm_patterns(segment_name)`

Get RFM patterns for a segment.

**Parameters:**
- `segment_name` (str): Segment name

**Returns:** List[Tuple[recency, frequency, monetary]]

## Segments

ClusterAudienceKit generates 13 customer segments:

| Segment | Description | RFM Pattern |
|---------|-------------|------------|
| **Champions** | Best customers, highest value | High R, High F, High M |
| **VIP** | Premium customers | High R, Any F, High M |
| **Loyal Customers** | Consistent, valuable | High R, High F, High M |
| **Potential Loyalists** | Recent high-value | High R, Low-Mid F, High M |
| **Cannot Lose** | High value, declining | Low R, High F, High M |
| **At Risk** | Was valuable, now declining | Low R, Low-Mid F, High M |
| **About to Sleep** | Dormant risk | Low R, Low-Mid F, Low-Mid M |
| **New Customers** | Recent, high activity | High R, High F, Low M |
| **Promising** | Recent, moderate value | High R, Low F, Mid-High M |
| **Need Attention** | Recent, low frequency | Mid-High R, Low F, Low M |
| **Lost** | Inactive | Low R, Low F, Low M |
| **At Risk - Sleeping** | Was valuable, gone cold | Low R, High F, High M |
| **Hibernating** | Very inactive, high value | Very Low R, Low-Mid F, High M |

## Usage Examples

### Example 1: Export Champions for Snowflake

```python
from clusteraudiencekit import export_segment_sql

sql = export_segment_sql(
    "Champions",
    "snowflake",
    "analytics.customer_segments"
)

# Use in warehouse
# INSERT INTO my_segments SELECT * FROM (
# <sql>
# )
```

### Example 2: Custom Column Mapping

```python
sql = export_segment_sql(
    "AtRisk",
    "bigquery",
    "project.dataset.customers",
    customer_id="cust_id",
    recency_score="r_score",
    frequency_score="f_score",
    monetary_score="m_score"
)
```

### Example 3: Batch Export All Segments

```python
from clusteraudiencekit import export_all_segments_sql

queries = export_all_segments_sql(
    "redshift",
    "analytics.customers"
)

# queries is a dict like:
# {
#     "Champions": "SELECT ...",
#     "AtRisk": "SELECT ...",
#     ...
# }

# Create views for each segment
for segment_name, sql in queries.items():
    view_name = f"segment_{segment_name.lower()}"
    create_view_sql = f"CREATE VIEW {view_name} AS {sql}"
    # Execute in Redshift
    cursor.execute(create_view_sql)
```

### Example 4: Apply Segments in Production Workflow

```python
from clusteraudiencekit import export_all_segments_sql

# Get all segment queries for your warehouse
queries = export_all_segments_sql("snowflake", "raw.customers")

# Create a staging table with segment assignments
staging_sql = """
CREATE OR REPLACE TABLE staging.customer_segments AS
"""

# Add each segment's customers
for segment_name, select_sql in queries.items():
    if segment_name != "Champions":  # Skip first
        staging_sql += " UNION ALL "
    staging_sql += select_sql

# Merge into production table
merge_sql = """
MERGE INTO prod.customer_segments target
USING staging.customer_segments source
ON target.customer_id = source.customer_id
WHEN MATCHED THEN UPDATE SET segment = source.segment
WHEN NOT MATCHED THEN INSERT *;
"""

# Execute both queries
```

## Warehouse-Specific Notes

### Snowflake
- Uses standard SQL syntax
- Supports IN clauses and complex WHERE conditions
- Consider case-sensitivity for column names (Snowflake uppercases by default)

### BigQuery
- Uses standard SQL
- May need project.dataset.table notation
- Supports IN clauses and subqueries

### Redshift
- PostgreSQL dialect with optimizations
- Uses IN clauses for efficiency
- Consider VACUUM and ANALYZE for performance

### PostgreSQL
- Standard SQL with IN clause optimization
- Consider indexing on RFM score columns
- Use EXPLAIN to optimize query execution

### Oracle
- Uses Oracle-specific syntax
- Compact spacing (recency_score=5 vs recency_score = 5)
- Consider partitioning for large tables

### SQL Server
- T-SQL optimizations
- Supports IN clauses
- Use execution plans to optimize performance

### MySQL
- Standard SQL compatibility
- Ensure RFM columns are properly indexed
- Consider query performance with large datasets

## Performance Tips

1. **Index RFM Columns**: Create indices on recency_score, frequency_score, monetary_score
   ```sql
   CREATE INDEX idx_rfm ON customers (recency_score, frequency_score, monetary_score);
   ```

2. **Partition Large Tables**: Use time-based or customer-based partitioning
   ```sql
   CREATE TABLE customers (
       ...
   ) PARTITION BY RANGE (customer_id);
   ```

3. **Materialize Segments**: Store segment assignments in a separate table
   ```sql
   CREATE TABLE customer_segments AS
   SELECT * FROM (<all segment queries>);
   ```

4. **Use Statistics**: Refresh table statistics for optimizer
   ```sql
   ANALYZE TABLE customers;  -- MySQL
   VACUUM ANALYZE customers;  -- PostgreSQL
   DBMS_STATS.GATHER_TABLE_STATS(...);  -- Oracle
   ```

## Integration Examples

### dbt Integration
```yaml
# models/staging/customer_segments.sql
{% set segment_queries = export_all_segments_sql("snowflake", "raw.customers") %}

WITH segments AS (
  {% for segment_name, query in segment_queries.items() %}
    {{ query | replace('SELECT', 'SELECT') }}
    {% if not loop.last %} UNION ALL {% endif %}
  {% endfor %}
)

SELECT * FROM segments
```

### Airflow Integration
```python
from airflow import DAG
from airflow.operators.sql import SQLExecuteQueryOperator
from clusteraudiencekit import export_all_segments_sql

dag = DAG("segment_refresh")

queries = export_all_segments_sql("snowflake", "raw_customers")

for segment_name, sql in queries.items():
    SQLExecuteQueryOperator(
        task_id=f"segment_{segment_name}",
        sql=sql,
        dag=dag
    )
```

## Troubleshooting

### "Segment not found"
Ensure you're using the exact segment name. Use `get_segment_rfm_patterns()` with different names to debug.

### "No .so files found"
The Rust extension is not installed. Reinstall: `pip install --force-reinstall clusteraudiencekit`

### SQL Syntax Errors
- Verify column names match your table
- Check dialect-specific syntax rules
- Use `EXPLAIN` or `ANALYZE` to debug

### Performance Issues
- Add indices on RFM score columns
- Use LIMIT to test before full execution
- Check table statistics

## FAQ

**Q: Can I modify the segment definitions?**
A: Currently, segment definitions are fixed. Define custom segments using the WHERE clauses as templates.

**Q: Do I need Python running to apply segments?**
A: No. Export SQL once, then run directly in your warehouse without Python.

**Q: Which dialect should I use?**
A: Use your warehouse's native dialect for best performance (Snowflake, BigQuery, etc.).

**Q: Can I combine multiple segments in one query?**
A: Yes. Create a UNION of queries or use CASE statements:
```sql
SELECT 
  customer_id,
  CASE 
    WHEN (recency_score = 5 AND ...) THEN 'Champions'
    WHEN (recency_score = 3 AND ...) THEN 'AtRisk'
    ELSE 'Other'
  END AS segment
FROM customers;
```

**Q: How often should I re-run segment assignments?**
A: Depends on your use case. Daily, weekly, or monthly re-calculations are typical.

## Support

For issues or questions:
- Check the tests: `tests/test_sql_export.py`
- Review examples: `examples/sql_export_example.py`
- File an issue: GitHub issues
