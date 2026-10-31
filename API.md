# ChangeRadar REST API Specification

ChangeRadar Daemon (`changeradard`) exposes a local, authenticated HTTP REST API allowing scripts, third-party integrations, extensions, and local automation tools to interact directly with the ChangeRadar engine.

---

## 1. Connection & Authentication

### Base URL
By default, the daemon binds to the loopback interface:
```
http://127.0.0.1:41820
```

> [!IMPORTANT]
> The ChangeRadar REST API is strictly bound to `127.0.0.1` by default for security. It should never be exposed to public networks without an encrypted reverse proxy and mutual TLS.

### Authentication
Every request must include an `Authorization` header containing the Bearer token configured in `%LOCALAPPDATA%/ChangeRadar/config.json`:

```http
Authorization: Bearer <api_token>
```

If the token is invalid or missing, the API responds with:
```json
{
  "error": "Unauthorized",
  "message": "Missing or invalid Bearer authentication token"
}
```
HTTP Status: `401 Unauthorized`

---

## 2. Endpoints Summary

| Method | Endpoint | Description |
|:---|:---|:---|
| `GET` | `/api/v1/health` | Service health status and uptime |
| `GET` | `/api/v1/folders` | List all monitored folders |
| `POST` | `/api/v1/folders` | Add a new folder to monitor |
| `DELETE` | `/api/v1/folders/:id` | Remove a monitored folder |
| `GET` | `/api/v1/files` | List tracked files (supports `folder_id` filter) |
| `GET` | `/api/v1/files/:id/versions` | List historical versions of a file |
| `GET` | `/api/v1/events` | Query recent filesystem change events |
| `GET` | `/api/v1/snapshots` | List point-in-time directory snapshots |
| `POST` | `/api/v1/snapshots` | Create a new snapshot for a folder |
| `POST` | `/api/v1/compare` | Compare two file versions across diff engines |
| `POST` | `/api/v1/recovery/restore` | Execute safe 8-step file recovery |
| `GET` | `/api/v1/intelligence/score` | Retrieve Change Radar Score and velocity analytics |
| `GET` | `/api/v1/storage` | Retrieve CAS metrics, quota, and storage breakdown |
| `POST` | `/api/v1/storage/cleanup` | Execute retention policy and garbage collection |

---

## 3. Endpoint Reference

### 3.1 Health Check
**`GET /api/v1/health`**

Returns the operational state of the ChangeRadar daemon.

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 14205,
  "engine_state": "running"
}
```

---

### 3.2 List Monitored Folders
**`GET /api/v1/folders`**

Returns all directories currently monitored by ChangeRadar.

**Response:**
```json
[
  {
    "id": "f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
    "path": "C:/Users/ajays/Documents/Research",
    "name": "Research",
    "enabled": true,
    "recursive": true,
    "created_at": 1729840100
  }
]
```

---

### 3.3 Add Monitored Folder
**`POST /api/v1/folders`**

Registers a new folder for real-time filesystem monitoring.

**Request Body:**
```json
{
  "path": "C:/Users/ajays/Documents/Research",
  "name": "Research",
  "recursive": true
}
```

**Response:**
```json
{
  "id": "f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
  "path": "C:/Users/ajays/Documents/Research",
  "name": "Research",
  "enabled": true,
  "recursive": true,
  "created_at": 1729840100
}
```
HTTP Status: `201 Created`

---

### 3.4 Remove Monitored Folder
**`DELETE /api/v1/folders/:id`**

Stops monitoring a folder. Existing historical versions in CAS remain intact unless cleaned up.

**Response:**
```json
{
  "success": true,
  "message": "Folder removed from monitoring"
}
```

---

### 3.5 List Tracked Files
**`GET /api/v1/files?folder_id=<optional_folder_id>`**

Retrieves tracked file records.

**Response:**
```json
[
  {
    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "folder_id": "f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
    "path": "C:/Users/ajays/Documents/Research/paper.md",
    "name": "paper.md",
    "extension": "md",
    "size_bytes": 14290,
    "current_hash": "c092fb138e6be01cd6d9a8a7de5d88691f14ec048384f509e5b94541ecf0f089",
    "created_at": 1729840120,
    "updated_at": 1729845600
  }
]
```

---

### 3.6 List Historical File Versions
**`GET /api/v1/files/:id/versions`**

Retrieves the complete immutable version history for a specific file.

**Response:**
```json
[
  {
    "id": "v-9981-a20",
    "file_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "version_number": 3,
    "content_hash": "c092fb138e6be01cd6d9a8a7de5d88691f14ec048384f509e5b94541ecf0f089",
    "size_bytes": 14290,
    "compressed_size_bytes": 4810,
    "change_kind": "MODIFIED",
    "pinned": true,
    "created_at": 1729845600
  },
  {
    "id": "v-9980-b11",
    "file_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "version_number": 2,
    "content_hash": "1f82bc923e44ca01ab6d88a7de5d88691f14ec048384f509e5b94541ecf01234",
    "size_bytes": 12800,
    "compressed_size_bytes": 4200,
    "change_kind": "MODIFIED",
    "pinned": false,
    "created_at": 1729843200
  }
]
```

---

### 3.7 List Filesystem Events
**`GET /api/v1/events?limit=50&offset=0`**

Retrieves recorded filesystem change events.

**Response:**
```json
[
  {
    "id": "evt-00129",
    "file_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "folder_id": "f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
    "path": "C:/Users/ajays/Documents/Research/paper.md",
    "kind": "MODIFIED",
    "timestamp": 1729845600,
    "details": "Content altered (1,490 bytes added)"
  }
]
```

---

### 3.8 List Snapshots
**`GET /api/v1/snapshots?folder_id=<optional_folder_id>`**

Retrieves captured point-in-time directory snapshots.

**Response:**
```json
[
  {
    "id": "snap-552",
    "folder_id": "f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
    "name": "Draft Complete Milestone",
    "description": "Snapshot before chapter 4 revisions",
    "file_count": 42,
    "total_bytes": 1054920,
    "created_at": 1729844000
  }
]
```

---

### 3.9 Create Directory Snapshot
**`POST /api/v1/snapshots`**

Captures a complete point-in-time snapshot manifest of a folder.

**Request Body:**
```json
{
  "folder_id": "f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
  "name": "Before large refactor",
  "description": "Pre-refactor checkpoint"
}
```

**Response:**
```json
{
  "id": "snap-553",
  "folder_id": "f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
  "name": "Before large refactor",
  "description": "Pre-refactor checkpoint",
  "file_count": 42,
  "total_bytes": 1054920,
  "created_at": 1729850100
}
```

---

### 3.10 Compare Two File Versions
**`POST /api/v1/compare`**

Performs multi-format difference analysis between two versions.

**Request Body:**
```json
{
  "version_id_a": "v-9980-b11",
  "version_id_b": "v-9981-a20",
  "mode": "unified"
}
```

**Response:**
```json
{
  "file_type": "text",
  "additions": 14,
  "deletions": 2,
  "modifications": 3,
  "identical": false,
  "diff_text": "--- v2\n+++ v3\n@@ -10,4 +10,16 @@\n- Preliminary conclusions.\n+ Refined conclusions based on secondary analysis.\n+ Additional references.\n",
  "similarity_score": 0.88
}
```

---

### 3.11 Safe File Recovery
**`POST /api/v1/recovery/restore`**

Executes the safe 8-step recovery pipeline to restore a file version.

**Request Body:**
```json
{
  "version_id": "v-9980-b11",
  "target_path": "C:/Users/ajays/Documents/Research/paper.md",
  "strategy": "original",
  "user_note": "Reverting accidental deletion of Section 3"
}
```

**Response:**
```json
{
  "success": true,
  "restored_path": "C:/Users/ajays/Documents/Research/paper.md",
  "verified_hash": "1f82bc923e44ca01ab6d88a7de5d88691f14ec048384f509e5b94541ecf01234",
  "safety_backup_hash": "c092fb138e6be01cd6d9a8a7de5d88691f14ec048384f509e5b94541ecf0f089",
  "restored_at": 1729851000
}
```

---

### 3.12 Change Radar Score & Intelligence
**`GET /api/v1/intelligence/score`**

Calculates real-time activity metrics and transparent Change Radar Score (0–100).

**Response:**
```json
{
  "score": 38,
  "level": "moderate",
  "score_components": {
    "velocity_score": 18,
    "volume_score": 10,
    "churn_score": 10
  },
  "metrics": {
    "events_last_24h": 142,
    "files_modified_24h": 28,
    "deletions_last_24h": 3,
    "bytes_changed_24h": 4194304
  },
  "anomalies": []
}
```

---

### 3.13 Storage Breakdown & Metrics
**`GET /api/v1/storage`**

Returns storage consumption, object counts, deduplication efficiency, and quota status.

**Response:**
```json
{
  "total_objects": 1840,
  "total_raw_bytes": 4294967296,
  "total_stored_bytes": 1428571428,
  "compression_ratio": 3.01,
  "deduplication_savings_bytes": 858993459,
  "quota_limit_bytes": 53687091200,
  "quota_used_percentage": 2.66
}
```

---

### 3.14 Storage Cleanup & Garbage Collection
**`POST /api/v1/storage/cleanup`**

Executes configured retention rules (max age, max versions per file) and prunes unreferenced CAS objects.

**Request Body:**
```json
{
  "dry_run": false
}
```

**Response:**
```json
{
  "versions_pruned": 18,
  "objects_deleted": 12,
  "bytes_freed": 104857600
}
```

---

## 4. Error Responses

ChangeRadar returns standardized JSON error payloads:

```json
{
  "error": "NotFound",
  "message": "File version with ID 'v-9999' was not found in the database"
}
```

Standard HTTP status codes:
- `200 OK`: Request succeeded.
- `201 Created`: Resource successfully created.
- `400 Bad Request`: Malformed JSON or invalid parameter.
- `401 Unauthorized`: Missing or invalid Bearer token.
- `404 Not Found`: Requested resource not found.
- `409 Conflict`: Conflict during recovery or resource creation.
- `500 Internal Server Error`: Unhandled engine error.
