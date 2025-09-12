---
title: "ML Model Management & Caching"
date: 2025-09-10
layout: page
categories: [memory, models]
---

> **🚧 Implementation Status**
>
> ML model management is a core component of the embedded memory system
> from ADR-30. This documentation serves as the technical specification
> for model download, caching, and validation infrastructure currently
> under development.
>
> **Target**: Zero-configuration model management with integrity validation
> **Status**: Hugging Face integration and caching infrastructure in progress

## Overview

Caxton's embedded memory system requires machine learning models for
semantic search capabilities. The model management system provides automatic
download, validation, caching, and fallback mechanisms to ensure reliable
operation while minimizing storage requirements and network dependencies.

## Architecture Components

### Model Registry

The system maintains a registry of supported embedding models with their specifications:

```rust
pub struct ModelSpec {
    pub model_id: String,           // Hugging Face model ID
    pub model_type: ModelType,      // SentenceTransformer, ONNX, SafeTensors
    pub architecture: String,       // e.g., "bert", "distilbert", "roberta"
    pub dimensions: usize,          // Vector dimensions (384, 512, 768, etc.)
    pub max_sequence_length: usize, // Maximum input token length
    pub size_mb: u64,              // Approximate model size in MB
    pub checksum_sha256: String,    // Expected SHA-256 hash
    pub required_files: Vec<String>, // Model files needed for operation
    pub compatibility: Compatibility, // Hardware/OS requirements
}

pub enum ModelType {
    SentenceTransformer,  // Sentence-BERT compatible models
    Onnx,                 // ONNX Runtime models
    SafeTensors,          // SafeTensors format models
    TensorFlowLite,       // TensorFlow Lite models
}
```

### Supported Models

#### Primary Models (Recommended)

**all-MiniLM-L6-v2** (Default):

- **Size**: 23MB download, ~90MB memory usage
- **Dimensions**: 384
- **Performance**: 1000+ embeddings/second (CPU)
- **Languages**: English optimized, multilingual capable
- **Use Case**: General-purpose embeddings for most applications

**all-mpnet-base-v2** (High Quality):

- **Size**: 438MB download, ~1.2GB memory usage
- **Dimensions**: 768
- **Performance**: 200+ embeddings/second (CPU)
- **Languages**: English optimized
- **Use Case**: Higher quality embeddings for critical applications

**multilingual-e5-small** (Multilingual):

- **Size**: 118MB download, ~400MB memory usage
- **Dimensions**: 384
- **Performance**: 800+ embeddings/second (CPU)
- **Languages**: 100+ languages supported
- **Use Case**: Multilingual applications and content

#### Specialized Models

**msmarco-distilbert-base-v4** (Retrieval):

- **Size**: 268MB download
- **Dimensions**: 768
- **Optimization**: Passage retrieval and search
- **Use Case**: Document search and information retrieval

**paraphrase-multilingual-mpnet-base-v2** (Similarity):

- **Size**: 278MB download
- **Dimensions**: 768
- **Optimization**: Semantic similarity detection
- **Use Case**: Duplicate detection and content matching

**code-search-net** (Code):

- **Size**: 492MB download
- **Dimensions**: 768
- **Optimization**: Source code embeddings
- **Use Case**: Code analysis and programming assistance agents

### Model Download System

#### Hugging Face Integration

**Repository Access**:

```rust
pub struct HuggingFaceClient {
    base_url: String,        // Default: https://huggingface.co
    api_token: Option<String>, // Optional for private models
    timeout: Duration,       // Download timeout
    retry_attempts: usize,   // Retry failed downloads
}

pub struct ModelDownload {
    pub model_id: String,
    pub revision: Option<String>,  // Git revision/branch (default: main)
    pub files: Vec<String>,        // Specific files to download
    pub cache_dir: PathBuf,        // Local cache directory
    pub force_download: bool,      // Skip cache check
}
```

**Download Process**:

1. **Registry Lookup**: Validate model ID and get specifications
2. **Cache Check**: Check if model exists locally and is valid
3. **Repository Query**: Get file list and metadata from Hugging Face
4. **File Download**: Download required model files in parallel
5. **Integrity Validation**: Verify checksums and file completeness
6. **Installation**: Move validated files to cache directory
7. **Registry Update**: Update local registry with download metadata

#### Required Model Files

**SentenceTransformer Models**:

```text
├── config.json              # Model configuration
├── pytorch_model.bin        # Model weights (PyTorch)
├── tokenizer.json           # Fast tokenizer
├── tokenizer_config.json    # Tokenizer configuration
├── vocab.txt                # Vocabulary (BERT-style)
└── README.md               # Model documentation
```

**ONNX Models**:

```text
├── model.onnx              # ONNX model file
├── config.json             # Model configuration
├── tokenizer.json          # Tokenizer
└── tokenizer_config.json   # Tokenizer config
```

**SafeTensors Models**:

```text
├── model.safetensors       # Model weights (SafeTensors)
├── config.json             # Model configuration
├── tokenizer.json          # Tokenizer
└── tokenizer_config.json   # Tokenizer config
```

### Caching System

#### Cache Directory Structure

```text
~/.cache/caxton/models/
├── registry.json           # Local model registry
├── downloads/              # Temporary download directory
├── all-MiniLM-L6-v2/      # Model cache directory
│   ├── config.json
│   ├── pytorch_model.bin
│   ├── tokenizer.json
│   ├── .caxton_metadata    # Caxton-specific metadata
│   └── .caxton_checksum    # File integrity hashes
└── multilingual-e5-small/
    ├── model.onnx
    ├── config.json
    └── .caxton_metadata
```

#### Cache Management

**Cache Metadata**:

```json
{
  "model_id": "sentence-transformers/all-MiniLM-L6-v2",
  "revision": "main",
  "download_date": "2025-09-10T14:30:00Z",
  "last_validated": "2025-09-10T14:30:00Z",
  "download_source": "huggingface.co",
  "total_size_bytes": 90218496,
  "file_count": 6,
  "validation_status": "verified",
  "access_count": 145,
  "last_access": "2025-09-10T15:45:23Z"
}
```

**Cache Operations**:

- **Size Monitoring**: Track total cache size and usage patterns
- **Access Tracking**: Record model usage for cache eviction decisions
- **Automatic Cleanup**: Remove unused models after configurable period
- **Cache Validation**: Periodic integrity checks of cached models
- **Storage Limits**: Configurable maximum cache size

#### Cache Configuration

```yaml
# caxton.yaml
memory:
  model_cache:
    directory: "~/.cache/caxton/models"
    max_size_gb: 5.0
    max_models: 10
    cleanup_policy: "lru"  # lru, fifo, manual
    validation_interval: "24h"
    download_timeout: "30m"
    parallel_downloads: 3
```

### Model Validation

#### Integrity Verification

**SHA-256 Checksums**:

```rust
pub struct FileIntegrity {
    pub filename: String,
    pub expected_sha256: String,
    pub actual_sha256: Option<String>,
    pub size_bytes: u64,
    pub last_checked: SystemTime,
    pub status: ValidationStatus,
}

pub enum ValidationStatus {
    Valid,           // File passes all checks
    Invalid,         // Checksum mismatch
    Missing,         // File not found
    Corrupted,       // File exists but unreadable
    Unknown,         // Not yet validated
}
```

**Validation Process**:

1. **File Existence**: Verify all required files are present
2. **Size Validation**: Check file sizes match expected values
3. **Checksum Verification**: Calculate and compare SHA-256 hashes
4. **Format Validation**: Validate file format (JSON syntax, tensor format)
5. **Compatibility Check**: Verify model is compatible with current Candle version
6. **Functional Test**: Load model and generate test embedding

#### Model Loading Validation

**Runtime Validation**:

```rust
pub struct ModelValidation {
    pub can_load: bool,
    pub embedding_dimensions: Option<usize>,
    pub test_embedding: Option<Vec<f32>>,
    pub load_time_ms: u64,
    pub memory_usage_mb: f64,
    pub error_message: Option<String>,
}
```

**Validation Tests**:

- **Load Test**: Attempt to load model into Candle runtime
- **Embedding Test**: Generate embedding for test sentence "Hello world"
- **Dimension Check**: Verify embedding dimensions match specification
- **Performance Test**: Measure loading time and memory usage
- **Error Handling**: Capture and report any loading errors

### Download Strategies

#### Network-Aware Downloads

**Progressive Download**:

- Download essential files first (config, tokenizer)
- Download model weights after successful validation
- Resume interrupted downloads using HTTP Range requests
- Parallel downloads for multiple files

**Bandwidth Optimization**:

```yaml
download:
  strategy: "adaptive"     # adaptive, conservative, aggressive
  max_concurrent: 3        # Maximum parallel downloads
  chunk_size_kb: 1024     # Download chunk size
  retry_delay_ms: 5000    # Delay between retries
  compression: true       # Use gzip compression when available
```

**Network Resilience**:

- Exponential backoff for failed downloads
- Alternative mirror support (Hugging Face mirrors)
- Partial download resumption
- Network connectivity testing

#### Offline Operation

**Bundled Models**:

- Core models bundled with Caxton distribution
- Minimal model set for offline operation
- Automatic fallback to bundled models

**Manual Installation**:

```bash
# Manual model installation from local files
caxton model install \
  --model-id all-MiniLM-L6-v2 \
  --source ./downloaded-model/ \
  --validate-checksums

# Export model for offline distribution
caxton model export \
  --model-id all-MiniLM-L6-v2 \
  --output model-package.tar.gz \
  --include-metadata
```

### Fallback Mechanisms

#### Model Unavailability

**Fallback Chain**:

1. **Preferred Model**: User-specified or system default
2. **Compatible Alternative**: Same dimensions, different architecture
3. **Bundled Fallback**: Minimal model included with distribution
4. **Degraded Mode**: Operate without embeddings (keyword-only search)

**Fallback Configuration**:

```yaml
memory:
  embedding_model: "all-MiniLM-L6-v2"
  fallback_models:
    - "all-MiniLM-L12-v2"      # Same architecture, larger
    - "distiluse-base-multilingual-cased"  # Different arch, similar dims
    - "bundled-mini"            # Minimal bundled model
  fallback_strategy: "auto"    # auto, manual, strict
  allow_dimension_mismatch: false
```

#### Download Failures

**Failure Recovery**:

- **Temporary Failures**: Retry with exponential backoff
- **Network Issues**: Fall back to cached version if available
- **Server Errors**: Try alternative download sources
- **Corruption**: Re-download corrupted files
- **Disk Space**: Clean cache and retry with minimal model

**Error Handling**:

```rust
pub enum DownloadError {
    NetworkTimeout,
    NetworkError(String),
    ServerError(u16),          // HTTP status code
    ChecksumMismatch {
        expected: String,
        actual: String,
    },
    InsufficientDiskSpace {
        required: u64,
        available: u64,
    },
    PermissionDenied(String),
    InvalidModelFormat(String),
}
```

### Model Performance Optimization

#### Loading Optimization

**Lazy Loading**:

- Load models only when first needed
- Cache loaded models in memory
- Unload unused models to free memory
- Pre-warm critical models at startup

**Memory Management**:

```rust
pub struct ModelMemoryManager {
    pub max_loaded_models: usize,    // Maximum models in memory
    pub memory_limit_gb: f64,        // Total memory limit for models
    pub eviction_policy: EvictionPolicy, // LRU, FIFO, Manual
    pub preload_models: Vec<String>, // Models to load at startup
}

pub enum EvictionPolicy {
    LeastRecentlyUsed,    // Evict least recently used model
    FirstInFirstOut,      // Evict oldest loaded model
    Manual,               // Manual model management only
    MemoryPressure,       // Evict based on system memory pressure
}
```

#### Embedding Batch Processing

**Batch Configuration**:

```yaml
embedding:
  batch_size: 32           # Sentences per batch
  max_batch_wait_ms: 100   # Wait time to accumulate batch
  queue_size: 1000         # Maximum queued embedding requests
  parallel_batches: 2      # Concurrent batch processing
```

**Performance Optimization**:

- Group similar-length sequences for efficient padding
- Use optimal batch sizes for the target model
- Parallel processing for multiple batches
- Memory-efficient batch management

### Model Update Management

#### Version Management

**Model Versioning**:

```rust
pub struct ModelVersion {
    pub model_id: String,
    pub current_revision: String,
    pub available_revisions: Vec<String>,
    pub update_available: bool,
    pub last_update_check: SystemTime,
    pub auto_update_enabled: bool,
}
```

**Update Strategies**:

- **Manual Updates**: User-initiated model updates only
- **Automatic Updates**: Regular checks for model updates
- **Security Updates**: Immediate updates for security issues
- **Staged Updates**: Gradual rollout of model updates

#### Model Migration

**Backward Compatibility**:

- Maintain embedding compatibility across model updates
- Provide migration tools for dimension changes
- Support multiple model versions during transition
- Clear migration paths and documentation

**Migration Process**:

1. **Pre-Migration Validation**: Test new model compatibility
2. **Gradual Rollout**: Update agents incrementally
3. **Performance Monitoring**: Compare embedding quality
4. **Rollback Plan**: Quick rollback to previous model if needed
5. **Cleanup**: Remove old model after successful migration

### Configuration Examples

#### Development Configuration

```yaml
# caxton-dev.yaml
memory:
  backend: "embedded"
  embedding_model: "all-MiniLM-L6-v2"
  model_cache:
    directory: "~/.cache/caxton/models"
    max_size_gb: 2.0
    auto_download: true
    validation_on_startup: true
  download:
    timeout_minutes: 10
    retry_attempts: 3
    source: "huggingface"
```

#### Production Configuration

```yaml
# caxton-prod.yaml
memory:
  backend: "embedded"
  embedding_model: "all-mpnet-base-v2"  # Higher quality for production
  fallback_models:
    - "all-MiniLM-L6-v2"
    - "bundled-mini"
  model_cache:
    directory: "/opt/caxton/models"
    max_size_gb: 10.0
    cleanup_policy: "lru"
    validation_interval: "6h"
  download:
    timeout_minutes: 30
    retry_attempts: 5
    parallel_downloads: 2
    compression: true
  monitoring:
    track_download_metrics: true
    alert_on_failures: true
    performance_logging: true
```

#### Offline Configuration

```yaml
# caxton-offline.yaml
memory:
  backend: "embedded"
  embedding_model: "bundled-mini"
  model_cache:
    directory: "/data/caxton/models"
    auto_download: false      # Disable downloads
    bundled_models_only: true
  fallback_strategy: "strict"  # Only use pre-installed models
  download:
    enabled: false
```

## Error Handling and Recovery

### Download Error Recovery

**Network Issues**:

```rust
async fn handle_download_failure(error: DownloadError) -> Result<(), ModelError> {
    match error {
        DownloadError::NetworkTimeout => {
            // Retry with longer timeout
            retry_with_backoff(Duration::from_secs(60)).await
        }
        DownloadError::ChecksumMismatch { .. } => {
            // Re-download corrupted file
            clear_partial_download().await?;
            retry_download().await
        }
        DownloadError::InsufficientDiskSpace { required, available } => {
            // Attempt cache cleanup
            cleanup_cache(required - available).await?;
            retry_download().await
        }
        _ => Err(ModelError::DownloadFailed(error))
    }
}
```

**Recovery Strategies**:

- **Automatic Retries**: Exponential backoff for transient failures
- **Cache Cleanup**: Free space by removing unused models
- **Mirror Fallback**: Try alternative download sources
- **Degraded Operation**: Continue with available models

### Validation Error Handling

**Integrity Failures**:

- **Checksum Mismatch**: Re-download affected files
- **Missing Files**: Download missing model components
- **Format Errors**: Try alternative model formats (ONNX vs PyTorch)
- **Compatibility Issues**: Fall back to compatible model version

**Runtime Failures**:

- **Loading Errors**: Clear cache and re-download model
- **Memory Errors**: Fall back to smaller model
- **Performance Degradation**: Switch to optimized model variant

### Monitoring and Observability

#### Model Management Metrics

```rust
// Prometheus metrics for model management
caxton_model_download_duration_seconds{model_id, status}
caxton_model_cache_size_bytes
caxton_model_validation_failures_total{model_id, failure_type}
caxton_model_load_time_seconds{model_id}
caxton_model_memory_usage_bytes{model_id}
```

#### Health Checks

**Model System Health**:

- **Cache Integrity**: Verify cached models are valid
- **Download Connectivity**: Test Hugging Face connectivity
- **Model Loading**: Verify critical models can load
- **Embedding Generation**: Test embedding functionality

**Alerting Conditions**:

- Model download failures exceeding threshold
- Cache corruption detected
- Model loading failures
- Memory usage exceeding limits
- Disk space warnings

## Related Documentation

- [Memory System Overview](overview.md) - Core memory system architecture
- [Embedded Backend](embedded-backend.md) - SQLite + Candle implementation
- [ADR-0030: Embedded Memory System](../adrs/0030-embedded-memory-system.md) -
  Architectural decisions
- [Performance Specifications](../api/performance-specifications.md) -
  Performance SLOs

## Next Steps

1. **Model Selection**: Choose appropriate embedding model for your use case
2. **Cache Configuration**: Configure model cache size and policies
3. **Download Testing**: Validate model download and caching functionality
4. **Performance Tuning**: Optimize model loading and embedding generation
5. **Monitoring Setup**: Implement model management monitoring and alerting
