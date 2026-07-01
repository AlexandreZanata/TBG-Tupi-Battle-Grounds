# API Contract — _(Project Name)_

> Sketch before implementation. Version all public APIs from day one.

---

## Base URL

```
https://api.example.com/v1
```

## Authentication

| Method | Header / mechanism |
|--------|-------------------|
| _(e.g. Bearer JWT)_ | `Authorization: Bearer <token>` |
| Tenant | _(e.g. `X-Tenant-Id`)_ |

## Error format (all endpoints)

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable safe message",
    "correlationId": "uuid"
  }
}
```

## Pagination (list endpoints)

| Param | Default | Max |
|-------|---------|-----|
| `page` | 1 | — |
| `pageSize` | 20 | 50 |

---

## Endpoints

### _(ResourceName)_

#### `GET /v1/resources`

- **Auth:** _(roles)_
- **Response 200:** _(shape or link to OpenAPI)_

#### `POST /v1/resources`

- **Auth:** _(roles)_
- **Request body:** _(allow-listed fields only — no mass assignment)_
- **Idempotency:** `Idempotency-Key` header required? Yes / No
- **Response 201:** _(shape)_

---

## _(Add endpoints below)_

<!--
Agent: define OpenAPI/schema before handler code.
See agent-rules/10-api-design/contract-first.md
-->
