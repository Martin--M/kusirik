# Xtream Codes API Protocol Reference

This document outlines the API protocol endpoints, parameters, responses, and serialization logic utilized by the Kusirik backend to communicate with Xtream Codes-compatible servers.

---

## Base Client Configuration

All backend client requests are orchestrated by [XtreamClient](../src-tauri/src/api/client.rs). Every request appends authentication credentials directly in the query parameters:

```text
GET {server_url}/player_api.php?username={username}&password={password}&action={action}
```

---

## API Actions & Payload Mappings

### 1. Categories Query

To retrieve list categories (Live, VOD, or Series), the client queries the following actions. All category items map to the shared [CategoryApi](../src-tauri/src/api/common.rs) struct.

| Action | HTTP Request | Payload Struct | Description |
| :--- | :--- | :--- | :--- |
| `get_live_categories` | `GET /player_api.php?...&action=get_live_categories` | `Vec<CategoryApi>` | Gets all live channel categories. |
| `get_vod_categories` | `GET /player_api.php?...&action=get_vod_categories` | `Vec<CategoryApi>` | Gets all VOD movie categories. |
| `get_series_categories` | `GET /player_api.php?...&action=get_series_categories` | `Vec<CategoryApi>` | Gets all TV series categories. |

#### Category Struct Model
```json
{
  "category_id": "10",
  "category_name": "Action Movies"
}
```

### 2. Streams & Listings

Retrieves indices of active streams. These lists are virtualized in the frontend and cached inside SQLite for performance.

* **Live Streams**: `action=get_live_streams`
  - Returns: `Vec<LiveStreamApi>`
  - Mapped inside: `src-tauri/src/api/live.rs`
* **VOD Streams**: `action=get_vod_streams`
  - Returns: `Vec<VodStreamApi>`
  - Mapped inside: `src-tauri/src/api/vod.rs`
* **Series (Show List)**: `action=get_series`
  - Returns: `Vec<SeriesApi>`

### 3. Detailed Metadata (On-Demand)

Unlike category lists, detailed metadata is lazy-loaded on-demand to conserve user network resources and local memory.

* **VOD Info**: `action=get_vod_info`
  - Required Parameter: `&vod_id={stream_id}`
  - Returns: Dynamic JSON payload (cast, synopsis, rating, backdrop urls).
  - Backend Command: `get_vod_info` checks local `vod_info` table first, falling back to network queries on a cache miss.
* **Series Info**: `action=get_series_info`
  - Required Parameter: `&series_id={series_id}`
  - Returns: Dynamic JSON payload (seasons, episodes, release dates).

---

## Data Deserialization & Type Coercion

Xtream API servers are notoriously inconsistent. The same field (e.g., ID or archive availability) might return as an integer from one server and a string from another. 

To prevent serialization crashes, the backend implements custom deserializers in `src-tauri/src/api/client.rs`:

1. **`deserialize_option_string`**:
   - Safely parses JSON values that could be a `String`, an `Integer`, or `Null` into `Option<String>`.
2. **`deserialize_option_i32`**:
   - Coerces stringified numbers (e.g., `"123"`) or actual integers (`123`) into `Option<i32>`.
   - Used for fields like `tv_archive` or rating averages that vary in representation.
