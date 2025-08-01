# 🧠 CLAUDE.md - AI Development Guidelines for Warp Terminal

<div align="center">

[![AI Guidelines](https://img.shields.io/badge/AI-Guidelines-blue?style=for-the-badge)](https://github.com/gitworkflows/warp-terminal)
[![Code Quality](https://img.shields.io/badge/Quality-First-green?style=for-the-badge)](https://github.com/gitworkflows/warp-terminal)
[![Testing](https://img.shields.io/badge/Testing-Required-orange?style=for-the-badge)](https://github.com/gitworkflows/warp-terminal)

</div>

---

## 🔄 **Project Awareness & Context**

### **Project Understanding**
When working on Warp Terminal, always maintain awareness of:

- **Current Status**: 45% complete, 162 tracked components, build SUCCESS with 72 warnings
- **Architecture**: Enhanced modular design with event-driven system, plugin architecture
- **Priority Focus**: Command Palette (80% → 100%), UI Design Review (6 files), Testing (5% → 80%)
- **Recent Achievements**: M1-M7 milestones completed, comprehensive source structure implemented

### **Context Requirements**
- **Read TODO.md first** for current priorities and project health
- **Check INITIAL.md** for project overview and immediate tasks
- **Review `.claude/settings.local.json`** for project configuration
- **Understand PRP workflow** using `.claude/commands/` templates

### **Codebase Awareness Rules**
1. **Always check existing patterns** before implementing new features
2. **Follow established module structure** in `src/` directory (15+ modules)
3. **Respect architectural decisions** (event-driven, plugin system, performance monitoring)
4. **Maintain consistency** with existing code style and conventions

---

## 🧱 **Code Structure & Modularity**

### **Architecture Principles**
- **Modular Design**: Each feature should be self-contained with clear interfaces
- **Event-Driven**: Use the EventProcessor system for async operations
- **Plugin System**: Follow security sandboxing and lifecycle management patterns
- **Performance First**: Leverage multi-level caching and resource management

### **Module Organization Standards**
```rust
// Standard module structure
src/
├── core/                 // Core architecture components
├── command_history/      // Command history and search
├── command_palette/      // Command palette system (80% complete)
├── websocket/           // WebSocket real-time communication
├── agent_mode_eval/     // AI agent performance evaluation
├── graphql/             // GraphQL API functionality
├── languages/           // 100+ programming language support
└── virtual_fs/          // Virtual file system with encryption
```

### **Code Quality Standards**
- **Zero compilation errors** (currently SUCCESS status)
- **Minimize warnings** (target: <10, current: 72)
- **Follow Rust conventions** (naming, error handling, async patterns)
- **Use proper error handling** with comprehensive Result types
- **Implement Debug traits** manually where needed (avoid derive conflicts)

### **Integration Patterns**
- **Settings Integration**: Use SettingsState with proper persistence
- **UI Integration**: Follow established Message/Command patterns
- **Async Operations**: Use proper async/await with error handling
- **Plugin Integration**: Follow security sandboxing requirements

---

## 🧪 **Testing & Reliability**

### **Testing Requirements (CRITICAL FOCUS)**
> **Current Gap**: Only 5% test coverage - needs immediate attention

#### **Testing Standards**
- **Unit Tests**: Cover all core functionality with comprehensive test cases
- **Integration Tests**: Test component interactions and data flow
- **Performance Tests**: Benchmark critical paths (startup time, memory usage)
- **Error Handling Tests**: Verify proper error propagation and recovery

#### **Testing Implementation Guide**
```rust
// Standard test structure
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_feature_functionality() {
        // Arrange
        let test_data = setup_test_data();
        
        // Act
        let result = feature_function(test_data).await;
        
        // Assert
        assert!(result.is_ok());
        verify_expected_behavior(result.unwrap());
    }
}
```

#### **Testing Priorities**
1. **Command History System** - Fuzzy search, analytics, filtering
2. **Command Palette** - Quick actions, workflow integration
3. **Pane Management** - Layout persistence, resizing
4. **Settings System** - Persistence, validation, UI integration
5. **Error Analysis** - Multi-language parsing accuracy

### **Reliability Standards**
- **Error Handling**: Every async operation must handle errors gracefully
- **Resource Management**: Implement proper cleanup and memory management
- **Performance Monitoring**: Use existing performance tracking systems
- **Logging**: Comprehensive logging for debugging and monitoring

---

## ✅ **Task Completion**

### **Development Workflow**
1. **Read Current Priorities** from TODO.md (Section: 🔥 Priority Matrix)
2. **Check Milestone Status** (Currently M7 active, M8-M13 upcoming)
3. **Follow PRP Process** using `.claude/commands/generate-prp.md`
4. **Implement with Testing** (mandatory for all new features)
5. **Clean Up Warnings** as part of completion

### **Definition of Done**
- [ ] Feature functionality implemented and tested
- [ ] Unit tests written with >80% coverage for new code
- [ ] Integration tests for component interactions
- [ ] No new compilation warnings introduced
- [ ] Documentation updated (inline comments, README updates)
- [ ] Performance impact assessed and optimized if needed
- [ ] Error handling implemented with proper propagation

### **Priority Completion Order**
1. **🚨 CRITICAL**: Clean up 72 compilation warnings
2. **⚡ HIGH**: Complete Command Palette Quick Actions (80% → 100%)
3. **📋 MEDIUM**: UI Design Review and component planning
4. **🔮 LOW**: Advanced features and optimizations

---

## 📎 **Style & Conventions**

### **Rust Style Guidelines**
- **Naming**: Follow Rust conventions (snake_case for functions, PascalCase for types)
- **Error Handling**: Use `Result<T, E>` consistently, avoid panics in production code
- **Async Patterns**: Proper async/await usage with error propagation
- **Memory Management**: Leverage Rust's ownership system, avoid unnecessary clones

### **Code Organization**
```rust
// File structure template
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

// Public types first
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    // fields
}

// Implementation blocks
impl ComponentConfig {
    pub fn new() -> Self {
        // implementation
    }
}

// Tests at end
#[cfg(test)]
mod tests {
    // test code
}
```

### **Documentation Standards**
- **Public APIs**: Comprehensive rustdoc comments with examples
- **Modules**: Clear module-level documentation explaining purpose
- **Complex Logic**: Inline comments explaining algorithms and decisions
- **Error Cases**: Document error conditions and recovery strategies

---

## 📚 **Documentation & Explainability**

### **Documentation Requirements**
- **API Documentation**: All public functions must have rustdoc comments
- **Architecture Documentation**: Update docs/ when adding new systems
- **User Documentation**: Update user-facing docs for new features
- **Change Documentation**: Update CHANGELOG.md for significant changes

### **Explainability Standards**
- **Code Comments**: Explain WHY, not just WHAT
- **Design Decisions**: Document architectural choices and trade-offs
- **Performance Considerations**: Explain optimization strategies
- **Error Recovery**: Document error handling and recovery mechanisms

### **Documentation Templates**
```rust
/// Processes command history with fuzzy search capabilities.
/// 
/// This function implements intelligent scoring for command suggestions
/// based on usage patterns, recency, and context similarity.
/// 
/// # Arguments
/// * `query` - The search query string
/// * `history` - Command history to search through
/// 
/// # Returns
/// * `Result<Vec<CommandMatch>, SearchError>` - Ranked search results
/// 
/// # Errors
/// * `SearchError::InvalidQuery` - Query string is malformed
/// * `SearchError::IndexError` - Search index is corrupted
/// 
/// # Example
/// ```rust
/// let results = search_command_history("git st", &history).await?;
/// ```
pub async fn search_command_history(
    query: &str, 
    history: &CommandHistory
) -> Result<Vec<CommandMatch>, SearchError> {
    // Implementation
}
```

---

## 🧠 **AI Behavior Rules**

### **Development Approach**
1. **Understand Before Acting**: Always analyze existing code patterns first
2. **Follow Project Priorities**: Respect the current focus areas from TODO.md
3. **Test-Driven Development**: Write tests alongside implementation
4. **Incremental Improvements**: Build on existing architecture, don't rebuild
5. **Performance Awareness**: Consider memory and CPU impact of changes

### **Code Generation Rules**
- **No Breaking Changes**: Maintain backward compatibility with existing APIs
- **Error Handling First**: Always implement proper error handling
- **Follow Existing Patterns**: Match established code style and architecture
- **Security Considerations**: Follow plugin sandboxing and security patterns
- **Performance Optimization**: Use existing caching and monitoring systems

### **Problem Solving Approach**
1. **Analyze**: Understand the problem within project context
2. **Research**: Check existing solutions and patterns in codebase
3. **Design**: Plan implementation following architectural principles
4. **Implement**: Write code with comprehensive error handling and tests
5. **Validate**: Ensure solution meets requirements and passes all tests
6. **Document**: Update documentation and add appropriate comments

### **Quality Assurance Rules**
- **Zero Tolerance**: No compilation errors in submitted code
- **Warning Reduction**: Actively work to reduce warning count (current: 72)
- **Test Coverage**: Maintain >80% test coverage for new code
- **Performance Monitoring**: Use existing performance tracking systems
- **Security First**: Follow established security patterns and sandboxing

### **Communication Guidelines**
- **Status Updates**: Provide clear progress updates on complex tasks
- **Problem Reporting**: Document issues with context and potential solutions
- **Code Reviews**: Explain design decisions and trade-offs
- **Documentation**: Keep README and docs/ updated with changes

---

## 🎯 **Current Focus Areas (August 2025)**

### **Immediate Priorities**
1. **🚨 Code Quality**: Reduce 72 warnings to <10
2. **⚡ Command Palette**: Complete Quick Actions system (80% → 100%)
3. **📋 UI Design**: Analyze 6 WebP design files for implementation
4. **🧪 Testing**: Increase coverage from 5% to 80% minimum

### **Success Metrics**
- **Build Health**: SUCCESS status maintained, warnings <10
- **Feature Completion**: Command Palette 100% functional
- **Test Coverage**: >80% for all new code, >50% overall
- **Performance**: Startup time <2s, memory usage <200MB idle
- **Code Quality**: Zero compilation errors, clean clippy output

---

<div align="center">

**🎯 AI Development Confidence Score: 8.5/10**

*🟢 Architecture Understanding: Excellent | 🟡 Testing Focus: Critical | 🟢 Code Quality: Excellent*

---

*Created: August 1, 2025*  
*Based on: TODO.md project analysis and development patterns*  
*Updated: As project evolves and priorities change*

**🧠 Remember**: Quality and testing are not optional - they are the foundation of reliable software.

</div>
