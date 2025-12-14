# Kistaverk Documentation

Welcome to the kistaverk documentation! This file serves as the main entry point to all project documentation.

## 📚 Documentation Structure

```
docs/
├── README.md                  # 👈 You are here
├── architecture/              # High-level design and architecture
├── features/                  # Feature-specific documentation
├── development/               # Development guides and setup
└── reference/                 # API and technical reference
```

## 🏗️ Architecture

Understand the high-level design and architecture of kistaverk:

- **[System Architecture Overview](architecture/overview.md)** - Overall system architecture
- **[CAS Design](architecture/cas-design.md)** - Computer Algebra System architecture
- **[MIR JIT Integration](architecture/mir-integration.md)** - MIR Just-In-Time compilation integration

## ⚙️ Features

Learn about kistaverk's main features:

### Math Tool
- **[Math Tool Overview](features/math-tool/overview.md)** - Basic and advanced mathematical operations
- **[Precision Implementation](features/math-tool/precision.md)** - Arbitrary precision arithmetic
- **[Symbolic Math](features/math-tool/symbolic.md)** - Symbolic computation capabilities

### MIR Scripting
- **[MIR Scripting Overview](features/mir-scripting/overview.md)** - MIR language integration
- **[Examples](features/mir-scripting/examples.md)** - Practical MIR programming examples
- **[Integration](features/mir-scripting/integration.md)** - How MIR integrates with other features

## 👨‍💻 Development

Guides for developers and contributors:

### Android Development
- **[Android Build Guide](development/android/build-guide.md)** - Setting up Android builds
- **[Precision Setup](development/android/precision-setup.md)** - Configuring precision math on Android
- **[Troubleshooting](development/android/troubleshooting.md)** - Common issues and solutions

### General Development
- **[Testing Strategies](development/testing.md)** - How to test kistaverk components
- **[Contribution Guide](development/contributing.md)** - How to contribute to the project

## 📖 Reference

Technical reference documentation:

- **[MIR API Reference](reference/mir-api.md)** - MIR scripting API
- **[CAS API Reference](reference/cas-api.md)** - Computer Algebra System API

## 🚀 Getting Started

New to kistaverk? Start here:

1. **Read the [System Architecture Overview](architecture/overview.md)** to understand how kistaverk works
2. **Explore the [Math Tool](features/math-tool/overview.md)** to see the core mathematical capabilities
3. **Try the [MIR Scripting Examples](features/mir-scripting/examples.md)** to see metaprogramming in action
4. **Set up your development environment** using the [Android Build Guide](development/android/build-guide.md)

## 🤝 Contributing

Want to contribute? Check out:

- **[Contribution Guide](development/contributing.md)** - How to contribute code
- **[Testing Strategies](development/testing.md)** - How to ensure your changes work correctly
- **[Architecture Documents](architecture/)** - Understand the design before making changes

## 📝 Documentation Conventions

- **📘 Overview documents** provide high-level explanations
- **🔧 Implementation documents** contain technical details
- **💡 Example documents** show practical usage
- **⚠️ Troubleshooting documents** help solve common problems

## 🔍 Searching Documentation

Use `grep` or your IDE's search functionality to find specific topics:

```bash
# Search for "precision" in all documentation
grep -r "precision" docs/

# Find all references to MIR scripting
grep -r "MIR" docs/
```

## 📈 Documentation Status

- ✅ Main structure created
- ⏳ Content being migrated and organized
- 🔄 Cross-references being added
- 🗑️ Redundant content being cleaned up

**Last updated:** 2025-12-14