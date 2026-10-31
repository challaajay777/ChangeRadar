# Contributing to ChangeRadar

Thank you for your interest in contributing to ChangeRadar! As an open-source, local-first file intelligence and recovery platform, we welcome contributions from engineers, designers, technical writers, and testers of all experience levels.

---

## 1. How to Contribute

There are many ways to contribute:
- **Reporting Bugs**: Open an issue describing the problem, reproduction steps, OS, and version.
- **Suggesting Features**: Submit feature requests aligned with our local-first, zero-telemetry philosophy.
- **Improving Documentation**: Fix typos, add examples, or expand guides.
- **Writing Code**: Add new diff comparators, enhance the desktop UI, improve watcher reliability, or optimize CAS compression.

---

## 2. Development Workflow

1. **Fork and Clone**:
   ```bash
   git clone https://github.com/changeradar/changeradar.git
   cd ChangeRadar
   ```
2. **Create a Feature Branch**:
   ```bash
   git checkout -b feature/my-new-feature
   ```
3. **Make and Test Changes**:
   - Run tests: `cargo test --workspace`
   - Test UI: `cd apps/desktop && npm run build`
   - Check formatting: `cargo fmt --all -- --check`
   - Check linter: `cargo clippy --workspace --all-targets`
4. **Commit Your Changes**:
   Use clear, conventional commit messages:
   - `feat: add support for ODT file comparison`
   - `fix: resolve race condition in file watcher event debounce`
   - `docs: update API documentation for snapshot endpoints`
5. **Open a Pull Request**:
   - Provide a clear description of the problem solved.
   - Link any related issue numbers.
   - Confirm all CI checks pass.

---

## 3. Pull Request Guidelines

- **No Telemetry**: PRs introducing network requests or telemetry will be closed immediately.
- **Test Coverage**: Any new logic or bug fix must include corresponding unit or integration tests.
- **No Unsafe Code**: Avoid `unsafe` blocks in Rust crates unless strictly required for low-level OS interop and accompanied by comprehensive safety comments.
- **UI Consistency**: Follow Tailwind CSS conventions and ensure accessible keyboard navigation across all views.
