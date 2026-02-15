# AuraForge Security Documentation

## Content Security Policy (CSP)

### Current Configuration

```
default-src 'self';
base-uri 'none';
form-action 'none';
frame-ancestors 'none';
object-src 'none';
frame-src 'none';
media-src 'self' data:;
img-src 'self' asset: data:;
style-src 'self' 'unsafe-inline';
script-src 'self';
font-src 'self' data:;
connect-src 'self' https://api.tavily.com http://localhost:* http://127.0.0.1:*
```

### Directive Rationale

| Directive | Value | Rationale |
|-----------|-------|-----------|
| `default-src` | `'self'` | Restrictive default: only load resources from same origin |
| `base-uri` | `'none'` | Prevents base tag injection attacks |
| `form-action` | `'none'` | Prevents form submission to external sites (all forms are internal) |
| `frame-ancestors` | `'none'` | Prevents clickjacking by disallowing embedding in iframes |
| `object-src` | `'none'` | Blocks all plugins (Flash, Java, etc.) |
| `frame-src` | `'none'` | No iframes allowed (prevents iframe injection) |
| `media-src` | `'self' data:` | Audio/video from same origin + data URIs |
| `img-src` | `'self' asset: data:` | Images from same origin, Tauri assets, and data URIs |
| `style-src` | `'self' 'unsafe-inline'` | **See "Style Security" section below** |
| `script-src` | `'self'` | ✅ **No inline scripts allowed** (strongest protection) |
| `font-src` | `'self' data:` | Fonts from same origin + data URIs for embedded fonts |
| `connect-src` | `'self' https://api.tavily.com http://localhost:* http://127.0.0.1:*` | Whitelist: same origin + Tavily search API + localhost for Ollama LLM |

### Style Security (`style-src 'unsafe-inline'`)

#### Why `unsafe-inline` is Required

The `unsafe-inline` directive for styles is necessary due to:

1. **React Inline Styles (27 instances)**
   - Dynamic styles using `style={{}}` for component-specific styling
   - Used for conditional styling, animations, and responsive behavior
   - Example: `<div style={{ opacity: isVisible ? 1 : 0 }}>`

2. **Tailwind CSS**
   - Utility-first CSS framework generates classes dynamically
   - Some Tailwind features require inline styles for CSS variables

3. **React Markdown Rendering**
   - `react-markdown` component uses inline styles for formatting
   - Code syntax highlighting (`react-syntax-highlighter`) uses inline styles
   - Removing `unsafe-inline` would break all markdown rendering

#### Mitigation Strategy

While `unsafe-inline` is present, risk is mitigated by:

- ✅ **No `script-src 'unsafe-inline'`** — Scripts are fully protected
- ✅ Restrictive `default-src` prevents unexpected resource loading
- ✅ `connect-src` whitelist limits API connections to known endpoints
- ✅ All external content is sanitized (markdown goes through DOMPurify in react-markdown)
- ✅ No user-generated content is rendered as HTML without sanitization

#### Future Improvements (Optional)

To remove `unsafe-inline` for styles would require:
1. Migrate from Tailwind to CSS Modules (large refactor)
2. Replace all React inline styles with CSS classes
3. Use a markdown renderer that doesn't require inline styles
4. Use CSS-in-JS with hash-based CSP nonces

**Cost-Benefit:** Current CSP provides strong protection; removing `unsafe-inline` for styles offers marginal security gain at high development cost.

---

## Network Security

### Allowed External Connections

| Service | Endpoint | Purpose | Required |
|---------|----------|---------|----------|
| Tavily Search API | `https://api.tavily.com` | Web search provider | Optional (can disable in settings) |
| Ollama (local) | `http://localhost:*`, `http://127.0.0.1:*` | LLM inference | **Required** for core functionality |

### Data Privacy

- ✅ **Local-first architecture**: All LLM inference runs locally via Ollama
- ✅ **No cloud APIs required**: Application works fully offline (except optional web search)
- ✅ **No telemetry**: No analytics, crash reporting, or usage tracking
- ✅ **No API keys stored in code**: Only user-configured keys stored in local config

### Threat Model

**Threats Mitigated:**
- ✅ Cross-Site Scripting (XSS) via `script-src 'self'` and sanitized markdown
- ✅ Clickjacking via `frame-ancestors 'none'`
- ✅ Base tag injection via `base-uri 'none'`
- ✅ Plugin exploits via `object-src 'none'`
- ✅ Unauthorized external connections via `connect-src` whitelist

**Residual Risks:**
- ⚠️ CSS injection attacks (mitigated by no user-generated CSS)
- ⚠️ Malicious markdown (mitigated by DOMPurify sanitization in react-markdown)

---

## Input Validation

### User Input Constraints

| Input Type | Maximum Size | Validation |
|------------|--------------|------------|
| Message content | 100 KB | Length check + encoding validation |
| Session name | 200 characters | Alphanumeric + spaces + hyphens |
| Export folder name | 60 characters | Alphanumeric + hyphens (sanitized) |
| Config file | 10 KB | YAML schema validation |
| Codebase import | 1 MB total | Byte budget, .gitignore filtering, symlink protection |

### Filename Sanitization

All user-provided filenames are sanitized:
```
Input: "My Project 2024!@#$%"
Output: "my-project-2024"
```

- Lowercase conversion
- Alphanumeric + hyphens only
- Leading/trailing hyphens removed

---

## Database Security

### SQLite Configuration

- ✅ **WAL mode enabled**: Atomic transactions for multi-step operations
- ✅ **Foreign key constraints**: Cascading deletes prevent orphaned records
- ✅ **Schema migrations**: Version-controlled schema with backward compatibility
- ✅ **Parameterized queries**: All SQL uses prepared statements (no SQL injection)

### Data Persistence

- **Config file permissions (Unix)**: `0600` (owner read/write only)
- **Database location**: User's app data directory (platform-specific)
- **No encryption**: Data is local-only; OS-level encryption (FileVault/BitLocker) recommended

---

## Rust Backend Security

### Memory Safety

- ✅ Rust's ownership model prevents memory corruption
- ✅ No unsafe code blocks in core application logic
- ✅ All dependencies audited via `cargo audit` (automated in CI)

### Error Handling

- ✅ All errors wrapped in `AppError` enum with structured codes
- ✅ No sensitive data in error messages (API keys, file paths sanitized)
- ✅ Errors logged with `tracing` crate (configurable log levels)

---

## Dependency Security

### Frontend Dependencies (npm)

- Current audit status: **5 moderate vulnerabilities** (as of Phase 1 verification)
- Mitigation: All vulnerabilities are in dev dependencies or non-exploitable contexts
- Update strategy: Monthly `npm audit fix` (non-breaking)

### Backend Dependencies (Cargo)

- Rust ecosystem has strong security culture
- All dependencies pinned in `Cargo.lock`
- Automated `cargo audit` runs in CI/CD

---

## Platform-Specific Considerations

### macOS

- ✅ App notarization for Gatekeeper compliance
- ✅ Hardened runtime enabled
- ✅ Code signing with Developer ID certificate

### Linux

- ✅ `.deb` and `.AppImage` packaging
- ✅ Sandboxed Tauri runtime (no system-wide access)

### Windows (Deferred)

- ⚠️ Config file atomic writes not yet optimized for NTFS
- ⚠️ Windows CI/CD pipeline not configured

---

## Release Security Checklist

Before each release:

1. ✅ Run `npm audit` and review vulnerabilities
2. ✅ Run `cargo audit` and update dependencies if needed
3. ✅ Verify all tests pass (frontend + backend)
4. ✅ Review CSP configuration for unintended changes
5. ✅ Verify no hardcoded credentials or API keys in code
6. ✅ Test on all supported platforms (macOS, Linux)
7. ✅ Generate SBOM (Software Bill of Materials) for compliance

---

## Reporting Security Issues

**Do NOT open public GitHub issues for security vulnerabilities.**

Instead, email: [security contact TBD]

Expected response time: 48 hours

---

## References

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Content Security Policy Reference](https://content-security-policy.com/)
- [Tauri Security Best Practices](https://tauri.app/v1/guides/building/app-publishing/#security-best-practices)
- [Rust Security Advisories](https://rustsec.org/)
