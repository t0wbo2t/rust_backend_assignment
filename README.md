# Task Management API with 2FA and Caching

This is a production-quality Rust backend implementing a Task Management API built using Axum, SQLx, SQLite, and Argon2 password hashing. It includes JWT authentication, email-based two-factor authentication (2FA), role-based access control (RBAC), and per-user in-memory caching.

## Features

- **JWT Authentication & 2FA**:
  - Secure login with password verification hashed using Argon2.
  - A 2FA challenge is generated (6-digit verification code hashed with Argon2, expiring in 5 minutes).
  - Dev-only endpoint to retrieve the latest code (`GET /dev/email-logs/latest`).
  - Signed JWT token is issued only after verifying the 2FA code.
- **Role-Based Access Control (RBAC)**:
  - **Admin**: Can create tasks, assign tasks to users, and view their own assigned tasks.
  - **Staff**: Can log in and view only their own assigned tasks (forbidden from task creation or assignment).
- **Per-User Caching**:
  - Caches the `GET /tasks/view-my-tasks` endpoint per user ID.
  - Returns `cache.hit: false` on database fetch (first request) and `cache.hit: true` on subsequent requests.
  - Automatically invalidates the cache for the assignee and any previous assignees when tasks are assigned.

---

## Technical Stack

- **Language**: Rust (2024 Edition)
- **Framework**: Axum (v0.8)
- **Runtime**: Tokio
- **Database**: SQLite with SQLx (migrations run automatically on startup)
- **Hashing & JWT**: Argon2 for passwords/codes, `jsonwebtoken` (with `rust_crypto`) for tokens
- **Testing**: Tower-based service integration tests

---

## Setup & Running

### 1. Prerequisites
Ensure you have the Rust toolchain installed.

### 2. Environment Configuration
Create a `.env` file or use the defaults provided in `.env.example`:
```bash
cp .env.example .env
```

### 3. Run the Application
Start the server (by default listening on `127.0.0.1:3000`):
```bash
cargo run
```
Migrations will be automatically executed on startup, creating the `tasks.db` file.

### 4. Run the Integration Tests
To execute the automated end-to-end validation test suite:
```bash
cargo test
```

---

## API Endpoints

### 1. Seed Initial Users
- **Endpoint**: `POST /seed/users`
- **Description**: Seeds two initial users in the database:
  - Admin: `admin@example.com` (password: `admin123`)
  - James Bond: `jamesbond@example.com` (password: `james123`, role: staff)

### 2. Authentication Flow
- **Step 1: Credentials Login**
  - **Endpoint**: `POST /auth/login`
  - **Input Payload**:
    ```json
    {
      "email": "jamesbond@example.com",
      "password": "james123"
    }
    ```
  - **Output Response**:
    ```json
    {
      "login_challenge_id": "8f96e271-8b2b-426b-8874-9844e1837c76"
    }
    ```
- **Step 2: Retrieve Code (Dev-Only)**
  - **Endpoint**: `GET /dev/email-logs/latest`
  - **Output Response**:
    ```json
    {
      "recipient": "jamesbond@example.com",
      "code": "834920"
    }
    ```
- **Step 3: Verify 2FA & Get JWT**
  - **Endpoint**: `POST /auth/verify-2fa`
  - **Input Payload**:
    ```json
    {
      "login_challenge_id": "8f96e271-8b2b-426b-8874-9844e1837c76",
      "code": "834920"
    }
    ```
  - **Output Response**:
    ```json
    {
      "token": "eyJhbGciOi..."
    }
    ```

### 3. Task Management (Admin Only)
- **Create Task**
  - **Endpoint**: `POST /tasks` (Requires Admin JWT Bearer Token)
  - **Input Payload**:
    ```json
    {
      "title": "Task Title",
      "description": "Task Description",
      "status": "todo",
      "priority": "high",
      "assigned_to_id": null
    }
    ```
- **Assign Task(s)**
  - **Endpoint**: `POST /tasks/assign` (Requires Admin JWT Bearer Token)
  - **Input Payload**:
    ```json
    {
      "task_ids": ["8f96e271-8b2b-426b-8874-9844e1837c76"],
      "user_id": "87ca4b2b-7e61-4de2-ba92-747d9539abef"
    }
    ```

### 4. View My Tasks
- **Endpoint**: `GET /tasks/view-my-tasks` (Requires JWT Bearer Token)
- **Output Response Shape**:
  ```json
  {
    "user": {
      "email": "jamesbond@example.com",
      "role": "staff"
    },
    "tasks": [
      {
        "id": "1bc7754f-124b-4b13-a4c3-e2346761fca3",
        "title": "Task 1",
        "status": "todo",
        "priority": "high",
        "assigned_to": "jamesbond@example.com"
      }
    ],
    "summary": {
      "total_assigned_tasks": 1
    },
    "cache": {
      "hit": true
    }
  }
  ```

---

## Development Seeding Helper
For manual testing convenience, the following dev endpoint seeds users, resets tasks, creates exactly 5 tasks, assigns 3 to James Bond, invalidates caches, and returns valid JWT tokens for both users in one step:
- **Endpoint**: `POST /dev/seed-full`
- **Output Response**:
  ```json
  {
    "admin_token": "ey...",
    "james_bond_token": "ey..."
  }
  ```
