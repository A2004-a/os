# NovaOS Apex — Application Model & Package Specification

## Nova Package Format (`.nvp` / `.npk`)

Applications are distributed as self-contained, signed binary packages:

```text
+-------------------------------------------------------------+
| Package Header (64 bytes)                                   |
| - Magic: 0x4E504B47 ('NPKG')                                |
| - Version: 1                                                |
| - Requested Capabilities Bitmask                            |
| - Manifest Offset & Size                                    |
| - Payload Offset & Size                                     |
| - Adler-32 / SHA-256 Integrity Checksum                     |
+-------------------------------------------------------------+
| Application Manifest (UTF-8 Key-Value / TOML)               |
| - app_id: "org.novaos.appname"                              |
| - name: "Application Display Name"                          |
| - version: "1.0.0"                                          |
| - author: "Author / Organization"                           |
| - exec_entry: "/bin/appname"                                |
| - required_caps: [GRAPHICS, STORAGE]                        |
+-------------------------------------------------------------+
| Executable Payload                                          |
| - ELF64 statically-linked executable binary                 |
| - Resource assets (icons, fonts, localized strings)         |
+-------------------------------------------------------------+
```

## Package Manager Lifecycle
1. **Verification:** Inspects header magic, checks package bounds, recomputes Adler-32 checksum against payload.
2. **Capability Check:** Verifies requested capabilities against the system ceiling. If an unprivileged user installs an app requesting `CAP_ADMIN`, installation is blocked.
3. **Registration:** Registers application into `/pkg/installed.db`.
4. **Sandboxed Execution:** Spawns process with its isolated Ring 3 address space and restricted capability token.
