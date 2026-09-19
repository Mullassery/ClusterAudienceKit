# ClusterAudienceKit Workflow Integration Guide

ClusterAudienceKit integrates seamlessly with workflow orchestration tools through CLI commands and REST APIs for audience creation, refresh, and member retrieval.

## Quick Start

### Option 1: CLI (Bash/Shell)

```bash
# Create an audience
clusteraudiencekit create-audience churn_risk "High Churn Risk Customers"

# Refresh membership
clusteraudiencekit refresh-audience churn_risk 5000

# Get members
clusteraudiencekit get-members churn_risk 100 0
```

### Option 2: REST API

```bash
# Start server
python -m clusteraudiencekit.server

# Create audience (HTTP POST)
curl -X POST http://localhost:8002/audiences \
  -H "Content-Type: application/json" \
  -d '{
    "audience_id": "churn_risk",
    "config": {
      "name": "High Churn Risk Customers"
    }
  }'

# Refresh audience
curl -X POST http://localhost:8002/audiences/churn_risk/refresh \
  -H "Content-Type: application/json" \
  -d '{"limit": 5000}'

# Get members
curl -X GET "http://localhost:8002/audiences/churn_risk/members?limit=100&offset=0"
```

---

## Workflow Tool Integration

### n8n (No-Code Workflow)

**Setup:**
1. Add "HTTP Request" node
2. Configure as shown below
3. Connect to other nodes

**Create Audience Node:**
```
Method: POST
URL: http://localhost:8002/audiences
Headers:
  Content-Type: application/json
Body:
{
  "audience_id": "{{ $node.trigger.json.audience_id }}",
  "config": {
    "name": "{{ $node.trigger.json.audience_name }}"
  }
}
```

**Refresh Audience Node:**
```
Method: POST
URL: http://localhost:8002/audiences/{{ $node.trigger.json.audience_id }}/refresh
Headers:
  Content-Type: application/json
Body:
{
  "limit": {{ $node.trigger.json.member_limit }}
}
```

**Get Members Node:**
```
Method: GET
URL: http://localhost:8002/audiences/{{ $node.trigger.json.audience_id }}/members?limit=100&offset=0
```

**Get Metrics Node:**
```
Method: GET
URL: http://localhost:8002/metrics?audience_id={{ $node.trigger.json.audience_id }}
```

---

### Power Automate (Microsoft Cloud)

**Create Audience:**
```
Method: POST
URI: https://your-server/api/audiences
Headers:
  Content-Type: application/json
Body:
{
  "audience_id": "@{triggerBody()?['audience_id']}",
  "config": {
    "name": "@{triggerBody()?['audience_name']}"
  }
}
```

**Refresh Audience:**
```
Method: POST
URI: https://your-server/api/audiences/@{triggerBody()?['audience_id']}/refresh
Headers:
  Content-Type: application/json
Body:
{
  "limit": @{triggerBody()?['member_limit']}
}
```

**Get Members (with Pagination):**
```
Method: GET
URI: https://your-server/api/audiences/@{triggerBody()?['audience_id']}/members?limit=100&offset=0
Headers:
  Content-Type: application/json
```

---

### Temporal (Durable Workflows)

**TypeScript Workflow:**
```typescript
import * as wf from "@temporalio/workflow";
import axios from "axios";

export async function refreshAudienceAndGetMembers(
  audienceId: string,
  memberLimit: number
) {
  // Create/refresh audience
  const refreshRes = await axios.post(
    "http://localhost:8002/audiences/" + audienceId + "/refresh",
    { limit: memberLimit }
  );

  const membersCalculated = refreshRes.data.members_calculated;

  // Paginate through members
  const allMembers = [];
  const pageSize = 100;
  for (let offset = 0; offset < membersCalculated; offset += pageSize) {
    const membersRes = await axios.get(
      `http://localhost:8002/audiences/${audienceId}/members?limit=${pageSize}&offset=${offset}`
    );
    allMembers.push(...membersRes.data.members);
  }

  return {
    audience_id: audienceId,
    total_members: allMembers.length,
    members: allMembers,
  };
}
```

---

### Apache Airflow (Python DAGs)

**DAG Example:**
```python
from airflow import DAG
from airflow.operators.python import PythonOperator
from datetime import datetime
import requests

def create_audience(audience_id, audience_name, **context):
    response = requests.post(
        "http://localhost:8002/audiences",
        json={
            "audience_id": audience_id,
            "config": {"name": audience_name},
        },
    )
    return response.json()

def refresh_audience(audience_id, member_limit, **context):
    response = requests.post(
        f"http://localhost:8002/audiences/{audience_id}/refresh",
        json={"limit": member_limit},
    )
    return response.json()

def get_audience_members(audience_id, **context):
    all_members = []
    page_size = 100
    offset = 0
    
    while True:
        response = requests.get(
            f"http://localhost:8002/audiences/{audience_id}/members",
            params={"limit": page_size, "offset": offset},
        )
        data = response.json()
        all_members.extend(data["members"])
        
        if len(data["members"]) < page_size:
            break
        offset += page_size
    
    return {"audience_id": audience_id, "members": all_members}

with DAG(
    "audience_refresh_pipeline",
    start_date=datetime(2024, 1, 1),
    schedule_interval="daily",
) as dag:
    create = PythonOperator(
        task_id="create_audience",
        python_callable=create_audience,
        op_kwargs={
            "audience_id": "churn_risk",
            "audience_name": "High Churn Risk",
        },
    )

    refresh = PythonOperator(
        task_id="refresh_audience",
        python_callable=refresh_audience,
        op_kwargs={"audience_id": "churn_risk", "member_limit": 5000},
    )

    get_members = PythonOperator(
        task_id="get_members",
        python_callable=get_audience_members,
        op_kwargs={"audience_id": "churn_risk"},
    )

    create >> refresh >> get_members
```

---

### UiPath (RPA)

**Sequence:**
```
1. Log Message: "Starting audience refresh"

2. HTTP Request
   URL: http://localhost:8002/audiences/churn_risk/refresh
   Method: POST
   Headers:
     Content-Type: application/json
   Body:
     {"limit": 5000}
   Output: response

3. Deserialize JSON
   JSON String: response
   Output: refresh_result

4. HTTP Request (Get Members)
   URL: http://localhost:8002/audiences/churn_risk/members?limit=100&offset=0
   Method: GET
   Output: members_response

5. Deserialize JSON
   JSON String: members_response
   Output: members_data

6. For Each: members_data.members
   Log Message: "[item]"
```

---

### Bash/Shell Scripts

**Full Audience Pipeline:**
```bash
#!/bin/bash

AUDIENCE_ID="churn_risk"
AUDIENCE_NAME="High Churn Risk Customers"
MEMBER_LIMIT=5000
API_URL="http://localhost:8002"

# Create audience
echo "Creating audience..."
clusteraudiencekit create-audience $AUDIENCE_ID "$AUDIENCE_NAME"

# Refresh membership
echo "Refreshing audience..."
REFRESH_RESULT=$(clusteraudiencekit refresh-audience $AUDIENCE_ID $MEMBER_LIMIT)
MEMBER_COUNT=$(echo $REFRESH_RESULT | jq -r '.members_calculated')
echo "Members calculated: $MEMBER_COUNT"

# Get members (paginated)
echo "Fetching members..."
ALL_MEMBERS=()
PAGE_SIZE=100
for ((OFFSET=0; OFFSET<MEMBER_COUNT; OFFSET+=PAGE_SIZE)); do
    MEMBERS=$(clusteraudiencekit get-members $AUDIENCE_ID $PAGE_SIZE $OFFSET | jq -r '.members[]')
    ALL_MEMBERS+=($MEMBERS)
done

echo "Total members retrieved: ${#ALL_MEMBERS[@]}"

# Output for downstream systems
printf '%s\n' "${ALL_MEMBERS[@]}" > audience_members.txt

# Get metrics
clusteraudiencekit metrics $AUDIENCE_ID | jq '.total_members'
```

**Batch Member Export:**
```bash
#!/bin/bash

AUDIENCE_ID=$1
OUTPUT_FILE=${2:-"audience_members.csv"}

# Get total members
METRICS=$(curl -s http://localhost:8002/metrics?audience_id=$AUDIENCE_ID)
TOTAL=$(echo $METRICS | jq -r '.total_members')

echo "Audience,Member ID,Customer ID" > $OUTPUT_FILE

# Paginate and export
PAGE_SIZE=1000
for ((OFFSET=0; OFFSET<TOTAL; OFFSET+=PAGE_SIZE)); do
    curl -s "http://localhost:8002/audiences/$AUDIENCE_ID/members?limit=$PAGE_SIZE&offset=$OFFSET" | \
    jq -r '.members[] | "$AUDIENCE_ID,\(.),\(.)"' >> $OUTPUT_FILE
done

echo "Exported $TOTAL members to $OUTPUT_FILE"
```

---

## API Endpoints Reference

### Audiences

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/audiences` | Create a new audience |
| GET | `/audiences` | List all audiences |
| POST | `/audiences/<id>/refresh` | Refresh audience membership |
| GET | `/audiences/<id>/members` | Get audience members (paginated) |

### Segments

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/segments` | Create a new segment |
| GET | `/segments` | List all segments |

### Monitoring

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| GET | `/metrics` | Get all metrics |
| GET | `/metrics?audience_id=X` | Get audience metrics |

---

## Docker Deployment

```dockerfile
FROM python:3.11-slim

WORKDIR /app
RUN pip install clusteraudiencekit flask

COPY . .

EXPOSE 8002

CMD ["python", "-m", "clusteraudiencekit.server"]
```

**docker-compose.yml:**
```yaml
version: '3.8'
services:
  clusteraudiencekit:
    build: .
    ports:
      - "8002:8002"
    environment:
      - FLASK_ENV=production
    restart: unless-stopped
```

---

## Integration Patterns

### Data Quality Gates
Check **StatGuardian** before audience activation:
```bash
# Validate audience freshness
if statguardian validate audience churn_risk; then
  clusteraudiencekit refresh-audience churn_risk 5000
else
  echo "Audience failed validation"
fi
```

### Intelligent Retrieval
Use **PyStreamMCP** to optimize member context retrieval:
```bash
# Get audience members
MEMBERS=$(clusteraudiencekit get-members churn_risk 100)

# Get optimized context for each member
for MEMBER in $(echo $MEMBERS | jq -r '.members[]'); do
  CONTEXT=$(pystreammcp query "context for $MEMBER" retrieve)
  # Use context in journey or activation
done
```

### Journey Activation
Combine with **PyCustomerJourney**:
```bash
# Refresh audience
clusteraudiencekit refresh-audience churn_risk 5000

# Get members
MEMBERS=$(clusteraudiencekit get-members churn_risk 100)

# Launch journey for each
echo $MEMBERS | jq '.members[]' | \
  xargs -I {} pycustomerjourney launch-journey churn_prevention {}
```

### Data Activation
Combine with **PyReverseETL**:
```bash
# Create audience
clusteraudiencekit create-audience high_value "High Value Customers"

# Refresh
clusteraudiencekit refresh-audience high_value 10000

# Activate to CRM via PyReverseETL
pyreverseetl activate --audience high_value --destination salesforce
```

---

## Performance Tips

1. **Pagination:** Always use limit/offset for large audiences
2. **Caching:** Cache membership lists locally when possible
3. **Batch Operations:** Use batch endpoints where available
4. **Rate Limiting:** Respect API rate limits in workflows
5. **Monitoring:** Track metrics for performance optimization

---

## Support

- Issues: https://github.com/Mullassery/ClusterAudienceKit/issues
- Discussions: https://github.com/Mullassery/ClusterAudienceKit/discussions
