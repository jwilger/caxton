# Quickstart: Health Check Endpoint

## Overview

This guide demonstrates how to use and test the Caxton health check endpoint. The health endpoint provides basic server availability monitoring for the Caxton AI agent platform.

## Prerequisites

- Caxton server running and configured
- Access to command line tools (`curl`, `httpie`, or similar)
- Network access to Caxton server

## Configuration

### Default Configuration

By default, the health endpoint is available at:

- **URL**: `http://localhost:8080/health`
- **Methods**: GET, HEAD
- **Response**: JSON format

### Custom Configuration

Configure the health endpoint via TOML configuration:

```toml
# caxton.toml
[server]
host = "0.0.0.0"        # Server bind address
port = 8080             # Server port
health_path = "/health" # Health endpoint path
```

### Environment Variables

Override configuration via environment variables:

```bash
export CAXTON_SERVER_HOST="0.0.0.0"
export CAXTON_SERVER_PORT="8080"
export CAXTON_SERVER_HEALTH_PATH="/health"
```

## Usage Examples

### Basic Health Check (GET)

Check server health with full JSON response:

```bash
# Using curl
curl -X GET http://localhost:8080/health

# Using httpie
http GET localhost:8080/health

# Expected Response:
# HTTP/1.1 200 OK
# Content-Type: application/json
#
# {"status":"OK"}
```

### Health Check (HEAD)

Check server health with headers only (no response body):

```bash
# Using curl
curl -I http://localhost:8080/health

# Expected Response:
# HTTP/1.1 200 OK
# Content-Type: application/json
```

### Advanced Examples

#### With Custom Port

```bash
curl http://localhost:3000/health
```

#### With Custom Host

```bash
curl http://192.168.1.100:8080/health
```

#### With Custom Health Path

```bash
curl http://localhost:8080/api/health
```

## Testing Health Check

### Manual Testing Checklist

1. **Server Running**: Verify Caxton server is operational

   ```bash
   # Should return {"status":"OK"}
   curl http://localhost:8080/health
   ```

2. **GET Method**: Test GET request returns JSON

   ```bash
   curl -X GET http://localhost:8080/health \
     -H "Accept: application/json"
   ```

3. **HEAD Method**: Test HEAD request returns headers only

   ```bash
   curl -I http://localhost:8080/health
   ```

4. **Invalid Method**: Verify error handling

   ```bash
   # Should return 405 Method Not Allowed
   curl -X POST http://localhost:8080/health
   ```

5. **Response Time**: Verify sub-100ms response requirement
   ```bash
   curl -w "Response time: %{time_total}s\n" \
     http://localhost:8080/health
   ```

### Automated Testing Script

```bash
#!/bin/bash
# health-check-test.sh

BASE_URL="${CAXTON_URL:-http://localhost:8080}"
HEALTH_PATH="${HEALTH_PATH:-/health}"
URL="$BASE_URL$HEALTH_PATH"

echo "Testing Caxton Health Endpoint: $URL"

# Test 1: GET request
echo "Test 1: GET request"
response=$(curl -s -w "%{http_code}" "$URL")
if [[ "${response: -3}" == "200" ]]; then
    echo "✅ GET request successful"
else
    echo "❌ GET request failed: ${response: -3}"
fi

# Test 2: HEAD request
echo "Test 2: HEAD request"
status=$(curl -s -I -w "%{http_code}" "$URL" | tail -1)
if [[ "$status" == "200" ]]; then
    echo "✅ HEAD request successful"
else
    echo "❌ HEAD request failed: $status"
fi

# Test 3: Invalid method
echo "Test 3: Invalid method (POST)"
status=$(curl -s -X POST -w "%{http_code}" -o /dev/null "$URL")
if [[ "$status" == "405" ]]; then
    echo "✅ Invalid method properly rejected"
else
    echo "❌ Invalid method not properly handled: $status"
fi

# Test 4: Response time
echo "Test 4: Response time"
time=$(curl -s -w "%{time_total}" -o /dev/null "$URL")
if (( $(echo "$time < 0.1" | bc -l) )); then
    echo "✅ Response time OK: ${time}s"
else
    echo "⚠️ Response time slow: ${time}s"
fi

echo "Health check testing complete"
```

## Integration with Monitoring

### Prometheus Monitoring

Monitor health endpoint with Prometheus:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: "caxton-health"
    metrics_path: "/health"
    static_configs:
      - targets: ["localhost:8080"]
    scrape_interval: 30s
```

### Kubernetes Health Checks

Configure Kubernetes health probes:

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: caxton
spec:
  template:
    spec:
      containers:
        - name: caxton
          image: caxton:latest
          ports:
            - containerPort: 8080
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 30
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
```

### Docker Compose Health Check

Configure Docker Compose health monitoring:

```yaml
# docker-compose.yml
version: "3.8"
services:
  caxton:
    image: caxton:latest
    ports:
      - "8080:8080"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s
```

## Troubleshooting

### Common Issues

#### Connection Refused

```bash
curl: (7) Failed to connect to localhost port 8080: Connection refused
```

**Solution**: Verify Caxton server is running and listening on correct port.

#### 404 Not Found

```bash
HTTP/1.1 404 Not Found
```

**Solution**: Check health endpoint path configuration in TOML file.

#### Invalid JSON Response

```bash
{"error":"Internal Server Error","message":"...","code":500}
```

**Solution**: Check Caxton server logs for detailed error information.

#### Slow Response Time

Response time > 100ms
**Solution**: Check server resource usage and potential performance bottlenecks.

### Debug Commands

```bash
# Check if server is listening
netstat -tlnp | grep :8080

# Test with verbose output
curl -v http://localhost:8080/health

# Check server logs
journalctl -u caxton -f

# Monitor response times
while true; do
  curl -w "Time: %{time_total}s\n" -s -o /dev/null http://localhost:8080/health
  sleep 1
done
```

## Performance Benchmarking

### Load Testing

Test health endpoint under load:

```bash
# Using Apache Bench
ab -n 1000 -c 10 http://localhost:8080/health

# Using wrk
wrk -t4 -c100 -d30s http://localhost:8080/health

# Performance expectations:
# - Response time: < 100ms (p95)
# - Throughput: > 1000 req/s
# - CPU usage: < 5% during load test
```

### Continuous Monitoring

Set up continuous health monitoring:

```bash
# Simple monitoring script
while true; do
  timestamp=$(date '+%Y-%m-%d %H:%M:%S')
  status=$(curl -s -w "%{http_code},%{time_total}" -o /dev/null http://localhost:8080/health)
  echo "$timestamp,$status" >> health-monitor.log
  sleep 5
done
```

## Next Steps

After validating the health check endpoint:

1. **Monitor in Production**: Set up monitoring dashboards
2. **Alert Configuration**: Configure alerts for health check failures
3. **Load Testing**: Validate performance under expected load
4. **Integration**: Integrate with existing monitoring infrastructure
5. **Documentation**: Update operational documentation with health check procedures

For more advanced monitoring and agent management features, see the full Caxton platform documentation.
