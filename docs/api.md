# API Specification

The backend exposes REST and WebSocket endpoints for interacting with positions.

## REST Endpoints

### POST `/positions/open`

**Request Body** (`OpenPositionRequest`):

```json
{
  "symbol": "BTC-PERP",
  "side": "long",
  "size": "1.5",
  "leverage": 20,
  "entry_price": "42000",
  "collateral": "3150"
}
```

**Response** (`PositionRecord`): position snapshot mirrored from cache.

### PUT `/positions/:id/modify`

Partially modifies a position (stubbed in reference implementation).

### DELETE `/positions/:id/close`

Closes a position with the provided settlement information.

### GET `/positions/:id`

Fetches a cached position record.

### GET `/users/:id/positions`

Returns all cached positions for a user.

## WebSocket `/ws`

* Streams serialized `PositionEvent` payloads.
* Events include `opened`, `modified`, `closed`, and `liquidation_alert`.
* Designed for UI updates and liquidation monitoring.

## Authentication

Authentication is not implemented in this reference. Production deployments should integrate wallet-based authentication and request signing.

## Rate Limits

Rate limiting is not implemented. Deploy behind an API gateway to enforce quotas.
