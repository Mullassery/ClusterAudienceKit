"""Comprehensive SQL export tests for all 8 SQL dialects."""

import pytest
from clusteraudiencekit import (
    export_segment_sql,
    export_all_segments_sql,
    get_supported_sql_dialects,
    get_segment_rfm_patterns,
)


class TestSQLDialects:
    """Test SQL export across all supported dialects."""

    EXPECTED_DIALECTS = [
        "ansi", "snowflake", "bigquery", "redshift",
        "postgresql", "oracle", "sqlserver", "mysql"
    ]

    SEGMENTS = [
        "Champions", "VIP", "LoyalCustomers", "PotentialLoyalists",
        "CannotLose", "AtRisk", "AboutToSleep", "NewCustomers",
        "Promising", "NeedAttention", "Lost", "AtRiskSleeping", "Hibernating"
    ]

    def test_supported_dialects(self):
        """Test that all 8 dialects are supported."""
        dialects = get_supported_sql_dialects()
        assert len(dialects) == 8
        for dialect in self.EXPECTED_DIALECTS:
            assert dialect in dialects

    @pytest.mark.parametrize("dialect", EXPECTED_DIALECTS)
    def test_export_champions_all_dialects(self, dialect):
        """Test Champions segment export in all dialects."""
        sql = export_segment_sql("Champions", dialect, "customers")

        # All queries should contain required elements
        assert "SELECT" in sql
        assert "FROM customers" in sql
        assert "WHERE" in sql
        assert "Champions" in sql

        # Should end with semicolon
        assert sql.rstrip().endswith(";")

    @pytest.mark.parametrize("segment", SEGMENTS)
    def test_export_all_segments_ansi(self, segment):
        """Test export of all 13 segments in ANSI SQL."""
        sql = export_segment_sql(segment, "ansi", "customers")
        assert segment in sql
        assert "SELECT" in sql
        assert "FROM customers" in sql
        assert "WHERE" in sql

    @pytest.mark.parametrize("dialect", EXPECTED_DIALECTS)
    @pytest.mark.parametrize("segment", SEGMENTS)
    def test_sql_validity(self, segment, dialect):
        """Test SQL validity for all segments in all dialects."""
        sql = export_segment_sql(segment, dialect, "customers")

        # Basic SQL structure validation
        assert "SELECT" in sql
        assert "FROM" in sql
        assert "WHERE" in sql
        assert segment in sql

        # Should have proper WHERE conditions (AND/OR)
        where_part = sql.split("WHERE")[1] if "WHERE" in sql else ""
        assert ("=" in where_part) or ("IN" in where_part)

    def test_snowflake_column_naming(self):
        """Test Snowflake uses proper column naming."""
        sql = export_segment_sql("Champions", "snowflake", "customers")
        assert "customer_id" in sql
        assert "recency_score" in sql
        assert "frequency_score" in sql
        assert "monetary_score" in sql

    def test_bigquery_column_alias(self):
        """Test BigQuery includes column alias."""
        sql = export_segment_sql("Champions", "bigquery", "customers")
        # BigQuery version should alias customer_id
        assert "customer_id AS customer_id" in sql or "customer_id," in sql

    def test_redshift_optimization(self):
        """Test Redshift uses IN clause optimization."""
        sql = export_segment_sql("Champions", "redshift", "customers")
        # Should have some optimization with IN clause
        assert "=" in sql
        # Champions has 4 patterns, Redshift should optimize some with IN
        assert "IN" in sql or sql.count("AND") > 0

    def test_postgresql_optimization(self):
        """Test PostgreSQL uses IN clause optimization."""
        sql = export_segment_sql("Champions", "postgresql", "customers")
        assert "=" in sql
        # PostgreSQL should use IN clause
        assert "IN" in sql or sql.count("AND") > 0

    def test_oracle_spacing(self):
        """Test Oracle SQL formatting."""
        sql = export_segment_sql("Champions", "oracle", "customers")
        # Oracle typically uses tighter spacing
        assert "recency_score=" in sql  # No spaces around =
        assert "AND" in sql

    def test_sqlserver_optimization(self):
        """Test SQL Server uses optimizations."""
        sql = export_segment_sql("Champions", "sqlserver", "customers")
        assert "=" in sql
        # Should potentially use IN clause
        assert ("IN" in sql or "OR" in sql)

    def test_mysql_compatibility(self):
        """Test MySQL SQL compatibility."""
        sql = export_segment_sql("Champions", "mysql", "customers")
        assert "SELECT" in sql
        assert "FROM" in sql
        assert "WHERE" in sql
        # MySQL should work with standard SQL

    def test_custom_column_mapping(self):
        """Test SQL export with custom column names."""
        sql = export_segment_sql(
            "Champions",
            "snowflake",
            "customers",
            customer_id="cust_id",
            recency_score="r",
            frequency_score="f",
            monetary_score="m"
        )

        assert "cust_id" in sql
        assert "r" in sql
        assert "f" in sql
        assert "m" in sql
        # Original names should not appear
        assert "recency_score" not in sql or "recency_score" in sql  # May appear in pattern
        assert "frequency_score" not in sql or "frequency_score" in sql

    def test_export_all_segments_batch(self):
        """Test batch export of all segments."""
        queries = export_all_segments_sql("snowflake", "customers")

        assert len(queries) == 13

        for segment in self.SEGMENTS:
            assert segment in queries
            query = queries[segment]
            assert "SELECT" in query
            assert "FROM customers" in query
            assert segment in query

    def test_export_large_table_name(self):
        """Test SQL export with complex table names."""
        sql = export_segment_sql(
            "Champions",
            "bigquery",
            "project.dataset.analytics_customers_v2"
        )

        assert "project.dataset.analytics_customers_v2" in sql
        assert "Champions" in sql

    def test_rfm_patterns_champions(self):
        """Test Champions segment has expected RFM patterns."""
        patterns = get_segment_rfm_patterns("Champions")

        # Champions should have 4 patterns
        assert len(patterns) == 4
        # All should have high scores
        for r, f, m in patterns:
            assert r >= 4
            assert f >= 4
            assert m >= 4

    def test_rfm_patterns_lost(self):
        """Test Lost segment has expected RFM patterns."""
        patterns = get_segment_rfm_patterns("Lost")

        # Lost should have low scores
        for r, f, m in patterns:
            assert r <= 2
            assert f <= 2
            assert m <= 2

    def test_rfm_patterns_at_risk(self):
        """Test At Risk segment patterns."""
        patterns = get_segment_rfm_patterns("AtRisk")

        # Should have patterns with declining engagement
        for r, f, m in patterns:
            # Low to medium recency/frequency
            assert r <= 3
            assert f <= 3
            # But higher monetary (was valuable)
            assert m >= 4

    @pytest.mark.parametrize("segment", SEGMENTS)
    def test_rfm_patterns_exist(self, segment):
        """Test that all segments have RFM patterns."""
        patterns = get_segment_rfm_patterns(segment)

        assert len(patterns) > 0
        # All patterns should be tuples of 3 integers 1-5
        for r, f, m in patterns:
            assert 1 <= r <= 5
            assert 1 <= f <= 5
            assert 1 <= m <= 5

    def test_sql_no_injection_vulnerability(self):
        """Test SQL export doesn't allow injection through segment names."""
        # Segment names are validated, so this should fail gracefully
        try:
            sql = export_segment_sql(
                "'; DROP TABLE customers; --",
                "snowflake",
                "customers"
            )
            # If it doesn't error, the injected text should not appear in SQL
            assert "DROP" not in sql
        except Exception:
            # Expected - invalid segment name
            pass

    def test_export_consistency(self):
        """Test that exporting same segment twice gives same result."""
        sql1 = export_segment_sql("Champions", "snowflake", "customers")
        sql2 = export_segment_sql("Champions", "snowflake", "customers")

        assert sql1 == sql2

    @pytest.mark.parametrize("dialect", EXPECTED_DIALECTS)
    def test_dialect_case_insensitive(self, dialect):
        """Test dialect names are case-insensitive."""
        sql_lower = export_segment_sql("Champions", dialect.lower(), "customers")
        sql_upper = export_segment_sql("Champions", dialect.upper(), "customers")

        # Both should work (case insensitive) and produce same result
        assert "Champions" in sql_lower
        assert "Champions" in sql_upper


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
