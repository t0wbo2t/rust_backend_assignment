# Rust Backend Coding Assignment - Implementation Specification

## Objective

Implement a production-style Rust backend for a Task Management API supporting:

* JWT Authentication
* Email-based Two-Factor Authentication (2FA)
* Role-Based Access Control (RBAC)
* Task Assignment
* Per-user Caching
* Database persistence

The implementation should support a complete end-to-end validation workflow.

---

# Technical Stack

## Required

* Rust 2021+
* Async runtime: Tokio
* Web framework

  * Axum (preferred)
  * or Actix Web
* Database

  * PostgreSQL (preferred)
  * SQLite (acceptable)
* ORM / Database

  * SQLx
  * SeaORM
  * Diesel
* Serialization

  * Serde
* JWT authentication
* Password hashing

  * Argon2 (preferred)
  * bcrypt
* Database migrations

## Optional

* Redis cache
* Docker Compose
* OpenAPI / Swagger
* tracing
* GitHub Actions
* rustfmt
* clippy

---

# Required Entities

## User

Fields

* id
* full_name
* email
* hashed_password
* role
* created_at
* updated_at

Role values

* admin
* staff

---

## Task

Fields

* id
* title
* description
* status
* priority
* created_by_id
* assigned_to_id (nullable)
* created_at
* updated_at

Status

* todo
* in_progress
* done

Priority

* low
* medium
* high

---

## LoginChallenge (2FA)

Fields

* id
* user_id
* hashed_code
* expires_at
* used
* created_at

Requirements

* code stored hashed (not plaintext)
* expires after 5 minutes
* single use

---

## EmailLog

Development only.

Fields

* id
* recipient
* code (or rendered email)
* created_at

Can alternatively log to console.

---

# Authentication Flow

## Step 1

POST /auth/login

Input

```json
{
  "email": "...",
  "password": "..."
}
```

Behavior

* validate credentials
* generate one-time verification code
* create LoginChallenge
* send email (or log)
* return

```json
{
  "login_challenge_id": "..."
}
```

Must NOT return JWT.

---

## Step 2

Retrieve latest email

Development endpoint

GET /dev/email-logs/latest

Returns latest verification code.

---

## Step 3

POST /auth/verify-2fa

Input

```json
{
  "login_challenge_id": "...",
  "code": "123456"
}
```

Validation

* challenge exists
* code matches
* unused
* not expired

Success

Return JWT.

Failure

Reject

* invalid code
* expired code
* reused code

---

# Authorization

Use JWT Bearer Authentication.

Roles

## Admin

Allowed

* create task
* assign task
* view own assigned tasks

## Staff

Allowed

* login
* view own assigned tasks

Forbidden

* create task
* assign task

Return

```
403 Forbidden
```

---

# Required API

## POST /seed/users

Create exactly two users

Admin

```
admin@example.com
role = admin
```

James Bond

```
jamesbond@example.com
role = staff
```

Passwords may be fixed.

---

## POST /auth/login

Starts login.

Returns challenge id.

---

## GET /dev/email-logs/latest

Returns latest verification code.

Development only.

---

## POST /auth/verify-2fa

Verifies challenge.

Returns JWT.

---

## POST /tasks

Admin only.

Create task.

---

## POST /tasks/assign

Admin only.

Assign task(s) to user.

Must invalidate affected cache.

---

## GET /tasks/view-my-tasks

Authenticated.

Returns tasks assigned to current user.

Includes cache metadata.

Example

```json
{
  "user": {
    "email": "...",
    "role": "staff"
  },
  "tasks": [],
  "summary": {
    "total_assigned_tasks": 3
  },
  "cache": {
    "hit": false
  }
}
```

---

# Business Rules

## Users

Exactly two seeded users

* Admin
* James Bond

---

## Tasks

Validation workflow requires

* create exactly 5 tasks
* assign exactly 3 to James Bond

---

## Permissions

Admin

* create tasks
* assign tasks

James Bond

Cannot

* create tasks

Can

* view only assigned tasks

Never see unassigned or others' tasks.

---

# Caching

Cache only

```
GET /tasks/view-my-tasks
```

Cache key

```
user_id
```

Behavior

First request

* database query
* cache result
* response

```json
"cache": {
    "hit": false
}
```

Second identical request

* return cached response

```json
"cache": {
    "hit": true
}
```

Cache invalidation

Whenever

* task assigned
* task updated
* assignment changed

Invalidate affected user's cache.

Redis preferred.

In-memory cache acceptable if documented.

---

# Validation Workflow

The implementation must successfully execute this sequence.

## 1

Seed users.

## 2

Admin login.

Receive

```
login_challenge_id
```

## 3

Retrieve verification code.

## 4

Verify 2FA.

Receive Admin JWT.

## 5

Create exactly

```
5
```

tasks.

## 6

Assign exactly

```
3
```

tasks to James Bond.

## 7

James Bond login.

## 8

Retrieve James verification code.

## 9

Verify 2FA.

Receive James JWT.

## 10

Attempt

```
POST /tasks
```

Expect

```
403 Forbidden
```

## 11

Call

```
GET /tasks/view-my-tasks
```

Expect

* exactly 3 tasks
* cache.hit = false

## 12

Call same endpoint again.

Expect

* same 3 tasks
* cache.hit = true

---

# Required Response Shape

```json
{
  "user": {
    "email": "jamesbond@example.com",
    "role": "staff"
  },
  "tasks": [
    {
      "id": "...",
      "title": "...",
      "status": "todo",
      "priority": "high",
      "assigned_to": "jamesbond@example.com"
    }
  ],
  "summary": {
    "total_assigned_tasks": 3
  },
  "cache": {
    "hit": false
  }
}
```

---

# Security Requirements

Passwords

* hashed with Argon2 or bcrypt

Verification codes

* hashed
* expire after 5 minutes
* single use

JWT

* issued only after successful 2FA

Never

* store plaintext passwords
* store plaintext verification codes

---

# Testing Requirements

Must test

* seed users
* login creates challenge
* login does not return JWT
* correct 2FA succeeds
* incorrect code rejected
* expired code rejected
* reused code rejected
* admin creates tasks
* admin assigns tasks
* staff cannot create tasks
* staff views exactly 3 assigned tasks
* cache miss on first request
* cache hit on second request
* cache invalidation after assignment/update

Use

```
cargo test
```

Include integration tests where practical.

---

# Deliverables

Repository must include

* source code
* README.md
* AI_USAGE.md
* .env.example
* migrations
* tests

README should document

* setup
* migrations
* running
* seeding
* validation workflow
* test execution
* final `/tasks/view-my-tasks` response

AI_USAGE.md should explain

* AI tools used
* manually written or modified code

---

# Suggested Project Structure

```
src/
    auth/
    cache/
    db/
    handlers/
    middleware/
    models/
    repositories/
    routes/
    services/
    state/
    tasks/
    users/
    utils/

migrations/
tests/
docs/

Cargo.toml
README.md
AI_USAGE.md
.env.example
```

---

# Acceptance Checklist

* Seed endpoint creates Admin and James Bond.
* Login returns only `login_challenge_id`.
* Email verification code is retrievable locally.
* JWT issued only after successful 2FA.
* Admin creates exactly 5 tasks.
* Admin assigns exactly 3 tasks to James Bond.
* James Bond receives `403 Forbidden` when creating tasks.
* `GET /tasks/view-my-tasks` returns exactly 3 assigned tasks.
* First request returns `cache.hit = false`.
* Second request returns `cache.hit = true`.
* Cache is invalidated after assignment or task updates.
* Passwords and 2FA codes are stored hashed.
* Core workflow is covered by automated tests.
