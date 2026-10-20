# Package Manifest Specification (`package.toml`)

`jpm` uses a [TOML](https://toml.io/) file (typically named `package.toml`) to define package metadata, dependencies, build instructions, environment variables, and lifecycle hooks for Linux From Scratch (LFS) systems.

---

## Table of Contents
- [Overview](#overview)
- [Manifest Sections](#manifest-sections)
  - [`[info]`](#info-required)
  - [`[dependency]`](#dependency-optional)
  - [`[env]`](#env-optional)
  - [`[build]`](#build-required)
  - [`[hook]`](#hook-optional)
- [Validation Rules](#validation-rules)
  - [Package Names](#package-names)
  - [Hash Formats](#hash-formats)
  - [Environment Variable Names](#environment-variable-names)
  - [Patch Paths](#patch-paths)
- [Complete Example](#complete-example)
- [Minimal Example](#minimal-example)

---

## Overview

A manifest consists of two required sections (`[info]` and `[build]`) and three optional sections (`[dependency]`, `[env]`, and `[hook]`).

| Section | Required | Description |
| :--- | :--- | :--- |
| [`[info]`](#info-required) | **Yes** | Core package metadata (name, version, source URL, hash, license). |
| [`[build]`](#build-required) | **Yes** | Build system definition and build/install command overrides. |
| [`[dependency]`](#dependency-optional) | No | Build-time, runtime, optional, and conflicting package dependencies. |
| [`[env]`](#env-optional) | No | Environment variables set during package compilation and installation. |
| [`[hook]`](#hook-optional) | No | Patches and phase script hooks executed across the package lifecycle. |

---

## Manifest Sections

### `[info]` (Required)

Contains fundamental information about the package and upstream source archive.

| Field | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `name` | String | **Yes** | Package name (must follow [Package Name rules](#package-names)). |
| `pkgver` | String | **Yes** | Upstream package version (e.g. `"1.4.7"`). Must not be empty or whitespace. |
| `pkgrel` | Integer | **Yes** | Package release/revision number for this version. Must be an integer `≥ 1`. |
| `desc` | String | **Yes** | Short summary or description of the software. |
| `license` | String | **Yes** | Software license identifier (e.g. `none`, `"GPL-3.0-or-later"`, `"MIT"`, `"Apache-2.0"`). |
| `src` | String | **Yes** | URL or path to download the source archive. |
| `hash` | String | **Yes** | Integrity checksum (see [Hash Formats](#hash-formats)) or `"skip"`. |
| `url` | String | No | Upstream homepage or repository URL. |
| `src_dir` | String | No | Extracted directory name inside archive if it differs from the default. |

---

### `[dependency]` (Optional)

Defines package relationships. All lists default to empty `[]` if omitted.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `build` | Array of Strings | `[]` | Tools and libraries required strictly at build time (e.g. compilers, CMake). |
| `runtime` | Array of Strings | `[]` | Packages required for the software to run on the target system (e.g. shared libraries). |
| `optional` | Array of Strings | `[]` | Optional packages enabling additional features. |
| `conflicts` | Array of Strings | `[]` | Packages incompatible with this package that must not be installed simultaneously. |

Every entry in these lists must be a valid [package name](#package-names).

---

### `[env]` (Optional)

Defines key-value pairs of environment variables exported during the build and install process.

```toml
[env]
CC = "clang"
CFLAGS = "-O2 -pipe"
MAKEFLAGS = "-j4"
```

- **Keys**: Must follow standard [Environment Variable Name rules](#environment-variable-names).
- **Values**: Must be strings.

---

### `[build]` (Required)

Specifies the build system and commands used to build and install the package.

| Field | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `system` | String | **Yes** | Build system identifier: `"autotools"`, `"cmake"`, `"meson"`, `"make"`, or `"custom"`. |
| `configure` | String | No | Custom configure command. Forbidden when `system = "make"`. |
| `compile` | String | No | Custom compilation command override. |
| `install` | String | Semi | Custom install command override. **Required** when `system = "custom"`. |
| `out_of_src` | Boolean | No | Whether to build outside the source directory. Must **not** be `false` for `"meson"`. |

#### Build System Rules

- **`autotools`**: Standard `./configure`, `make`, `make install` workflow.
- **`cmake`**: CMake project workflow.
- **`meson`**: Meson / Ninja workflow. Always requires out-of-source builds; setting `out_of_src = false` is an error.
- **`make`**: Pure Makefile project. `configure` must **not** be specified.
- **`custom`**: Arbitrary scripts or bespoke systems. Requires `install` to be specified.

> **Note**: Optional string fields (`configure`, `compile`, `install`) cannot be empty strings if specified.

---

### `[hook]` (Optional)

Specifies patches and phase lifecycle scripts.

| Field | Type | Description |
| :--- | :--- | :--- |
| `patches` | Array of Strings | Relative patch file paths applied before configuration (see [Patch Paths](#patch-paths)). |
| `pre_configure` | String | Shell script executed immediately before the configure step. |
| `pre_install` | String | Shell script executed immediately before the install step. |
| `post_install` | String | Shell script executed immediately after the install step (e.g. running `ldconfig`). |
| `pre_remove` | String | Shell script executed before package removal. |
| `post_remove` | String | Shell script executed after package removal. |

> **Note**: Multiline scripts can be written cleanly using TOML multi-line strings (`'''` or `"""`). If defined, hook script strings must not be empty.

---

## Validation Rules

### Package Names
All package names (in `info.name`, `dependency.build`, `dependency.runtime`, `dependency.optional`, and `dependency.conflicts`) must satisfy:
- Length: 1 to 64 characters.
- Allowed characters: ASCII alphanumeric (`a-z`, `A-Z`, `0-9`), hyphen (`-`), and underscore (`_`).
- First character: Must **not** begin with `-` or `_` (must start with an alphanumeric character).
- No leading, trailing, or internal whitespace.

### Hash Formats
Checksums in `info.hash` can be:
- `"skip"` (case-insensitive): Skips integrity verification.
- `<algo>:<digest>`: Cryptographic hash in hexadecimal format.

Supported algorithms and expected digest lengths:
- **`md5`**: 32 hex characters
- **`sha1`**: 40 hex characters
- **`sha256`**: 64 hex characters

Example: `sha256:933de2e4d1b2211a638be463eb6e8616891bfba73aef5d38060bd8319baeefc6`

### Environment Variable Names
Environment keys in `[env]` must satisfy:
- Must not be empty or contain leading/trailing whitespace.
- Must start with an ASCII letter (`a-z`, `A-Z`) or an underscore (`_`).
- Subsequent characters must be ASCII alphanumeric or underscores (`[a-zA-Z0-9_]`).

### Patch Paths
Paths listed in `hook.patches` are strictly checked for system security:
- Must be a **relative path** (cannot start with `/`).
- Must **not contain path traversal** (no `..` components).
- Must **not start with `~`** (home directory expansion).

---

## Complete Example

```toml
[info]
name = "btop"
pkgver = "1.4.7"
pkgrel = 1
desc = "A TUI monitor of resources"
license = "Apache-2.0"
url = "https://github.com/aristocratos/btop"
src = "https://github.com/aristocratos/btop/archive/v1.4.7/btop-1.4.7.tar.gz"
hash = "sha256:933de2e4d1b2211a638be463eb6e8616891bfba73aef5d38060bd8319baeefc6"
src_dir = "btop-1.4.7"

[dependency]
build = ["cmake", "gcc"]
runtime = ["glibc"]
optional = ["rocm"]
conflicts = ["btop-git"]

[env]
CXXFLAGS = "-O3 -flto"

[build]
system = "autotools"
configure = "--prefix=/usr"
compile = "make -j$(nproc)"
install = "make DESTDIR=$DESTDIR PREFIX=/usr install"
out_of_src = false

[hook]
patches = ["patches/0001-fix-install-path.patch"]
pre_configure = '''
sed -i 's|doc/btop|&-1.4.7|g' Makefile
'''
post_install = '''
echo "Installed btop successfully."
'''
```

---

## Minimal Example

```toml
[info]
name = "nano"
pkgver = "8.1"
pkgrel = 1
desc = "Small and friendly text editor"
license = "GPL-3.0-or-later"
src = "https://www.nano-editor.org/dist/v8/nano-8.1.tar.xz"
hash = "sha256:4eb24a732df308728d84ceba9bf57f33b1e7555eb3fce79f9f9f9d77a835b0e8"

[build]
system = "make"
```
