---
created: 2025-12-15T09:58:30.000Z
updated: 2025-12-15T10:20:05.901Z
type: memory
---
# Architectural Decisions

## 2024-12-15: Frontend Asset Embedding

### Context
The project goal is a single deployable binary. Initial implementation used `ServeDir::new("frontend/dist")` which looks for files at runtime relative to the current working directory.

### Problem
When the binary is moved or run from a different location (e.g., `./target/release/steering-center`), it can't find `frontend/dist/` and serves a blank white page. The build script masked this by always running from project root.

### Decision
Use `rust-embed` to embed frontend assets into the binary at compile time.

### Consequences
- **Build order matters**: Frontend must be built BEFORE `cargo build` since assets are embedded at compile time
- **Binary size increases**: All frontend assets are bundled into the executable
- **True portability**: Binary can be copied anywhere and run without external dependencies (except SQLite db file)
- **Development workflow changes**: Need to rebuild Rust after frontend changes to see updates in release binary

### Alternatives Considered
1. **Require specific working directory** - Rejected: fragile, bad UX
2. **Config file for asset path** - Rejected: adds deployment complexity, defeats "single binary" goal
3. **Embed assets with `include_dir!`** - Viable but `rust-embed` has better ergonomics and mime type handling

### Status
Implementing `rust-embed` solution.

## 2025-12-15T10:20:05.902Z

## 2024-12-15: v0.1.0 Release Fixes

### Task Cancellation Architecture

**Problem**: Original design stored child process in registry, then immediately removed it to get stdout/stderr handles. This meant cancellation couldn't find the process.

**Solution**: Changed `TaskRegistry` from `HashMap<String, Child>` to `HashMap<String, Arc<Mutex<Option<Child>>>>`. Now:
- Take stdout/stderr handles BEFORE storing in registry
- Child stays in registry during execution
- Cancellation can find and kill the process
- Clean up registry after task completes

**Pattern**: When you need both streaming access AND cancellation, separate the I/O handles from the process handle early.

### Server Binding Security

**Decision**: Default to `127.0.0.1` (localhost only), not `0.0.0.0`.

**Rationale**: The design doc explicitly states "localhost only, Cloudflare handles external traffic". Binding to all interfaces by default is a security risk for a tool that executes shell scripts.

**Override**: `STEERING_HOST=0.0.0.0` for users who need external access.

### Quick Actions UX

**Decision**: Quick Actions navigate to Scripts page with script pre-selected rather than executing inline on Dashboard.

**Rationale**: 
- Keeps terminal output in one place (Scripts page)
- User sees what's about to run before execution
- Simpler implementation, consistent UX
- Dashboard stays clean (overview, not execution)

### History Page

**Added**: `/history` route showing last 100 executions with:
- Expandable output view
- Status badges (Running/Success/Failed)
- Re-run button per task
- Task ID and timestamps

**Note**: Output is stored in SQLite. For long-running scripts with lots of output, this could grow the DB. Consider adding output truncation or retention policy in future versions.
