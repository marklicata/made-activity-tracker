# Activity Tracker for Amplifier - Project Overview

**Status**: ✅ Planning Complete, Ready for Implementation  
**Estimated Effort**: 50-70 hours (60% reduction from original plan)  
**Architecture**: Built on Paul Payne's issue-manager module

---

## 🎯 What We're Building

An intelligent activity tracking system that:
- **Automatically detects** duplicate work when developers start new tasks
- **Files new ideas** discovered during development
- **Coordinates across repos** for multi-repo projects
- **Uses LLM + embeddings** for accurate duplicate detection
- **Minimal overhead** - <2s impact on session start

### Key Innovation
Building on Paul's existing `issue-manager` module instead of external tools means we only need to add the intelligence layer - storage and CRUD are already done!

---

## 📚 Documentation

### Core Documents (Start Here)

1. **[SPECIFICATION_V2.md](SPECIFICATION_V2.md)** (24KB)
   - Complete technical specification
   - Architecture design
   - Data models and flows
   - Integration with issue-manager
   - **READ THIS FIRST**

2. **[TASKS.md](TASKS.md)** (18KB)
   - 28 detailed tasks across 4 phases
   - Hour estimates per task
   - Dependencies and priorities
   - Acceptance criteria
   - **YOUR IMPLEMENTATION ROADMAP**

3. **[TESTING_STRATEGY.md](TESTING_STRATEGY.md)** (22KB)
   - Unit, integration, performance, E2E tests
   - Coverage goals (>80%)
   - Test frameworks and tools
   - Example test code
   - **QUALITY ASSURANCE GUIDE**

4. **[PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md)** (19KB)
   - Repository structure
   - Module breakdown
   - Data flows
   - Configuration examples
   - **ARCHITECTURE REFERENCE**

### Supporting Documents

5. **[SPECIFICATION.md](SPECIFICATION.md)** (46KB)
   - Original design with Beads substrate
   - Archived for reference
   - Shows evolution of thinking

---

## 🚀 Quick Start

### Phase 1: MVP (Week 1-2, ~25 hours)

**What You'll Build**:
- Hook module that captures session start/end
- Simple LLM-based duplicate detection
- Integration with issue-manager
- Basic notification system

**How to Start**:
```bash
# 1. Set up project
cd C:\ANext\activity-tracker
mkdir amplifier-module-hooks-activity-tracker
cd amplifier-module-hooks-activity-tracker

# 2. Initialize Python project
python -m venv venv
venv\Scripts\activate
pip install amplifier-core openai pyyaml pytest

# 3. Create module structure (see PROJECT_STRUCTURE.md)
# 4. Start with Task 1.1 in TASKS.md
```

**First Tasks**:
- [ ] Task 1.1: Module Scaffold (2 hours)
- [ ] Task 1.2: Context Capture (3 hours)
- [ ] Task 1.3: Session Start Hook (4 hours)

---

## 📋 Implementation Phases

### Phase 1: MVP - Basic Tracking
**Duration**: 1-2 weeks  
**Effort**: ~25 hours  
**Tasks**: 10 tasks  
**Goal**: Prove concept with core functionality

**Deliverables**:
- ✅ Session start/end hooks
- ✅ Context capture (prompt, git, files)
- ✅ Simple LLM duplicate detection
- ✅ Session tracking in issue-manager
- ✅ Basic idea filing

---

### Phase 2: Enhanced Analysis
**Duration**: 1 week  
**Effort**: ~20 hours  
**Tasks**: 7 tasks  
**Goal**: Add embeddings for speed and accuracy

**Deliverables**:
- ✅ Embedding generation (OpenAI)
- ✅ Embedding cache (SQLite)
- ✅ Two-phase matching (embeddings → LLM)
- ✅ Performance optimization
- ✅ <5s analysis for 100 issues

---

### Phase 3: Multi-Repo Support
**Duration**: 1 week  
**Effort**: ~15 hours  
**Tasks**: 5 tasks  
**Goal**: Project groups and cross-repo coordination

**Deliverables**:
- ✅ ProjectGroupManager
- ✅ Multi-repo issue querying
- ✅ Smart repo selection for new issues
- ✅ Enhanced tool-issue operations
- ✅ Cross-repo duplicate detection

---

### Phase 4: Polish & Production
**Duration**: 3-5 days  
**Effort**: ~10 hours  
**Tasks**: 6 tasks  
**Goal**: Production-ready release

**Deliverables**:
- ✅ Comprehensive error handling
- ✅ Full test suite (>80% coverage)
- ✅ Complete documentation
- ✅ Performance validation
- ✅ v1.0.0 release

---

## 🏗️ Architecture Overview

```
User Session
    ↓
┌─────────────────────────────────┐
│  hooks-activity-tracker (NEW)   │ ← What we're building
│  • Session lifecycle hooks      │
│  • LLM-powered analysis         │
│  • Multi-repo coordination      │
└────────────┬────────────────────┘
             ↓
┌─────────────────────────────────┐
│  issue-manager (EXISTING) ✅    │ ← Paul's module (already done!)
│  • Storage & CRUD               │
│  • Dependencies & cycles        │
│  • Ready work detection         │
│  • Event tracking               │
└─────────────────────────────────┘
```

**Key Insight**: We're building a thin intelligence layer on top of solid existing infrastructure.

---

## 🧪 Testing Strategy

### Coverage Goals
- **Overall**: >80%
- **Critical paths**: >85%
- **Data integrity**: >95%

### Test Levels
1. **Unit Tests** (60%) - Individual functions
2. **Integration Tests** (30%) - Module interactions
3. **E2E Tests** (10%) - Full workflows

### Key Test Areas
- Context capture (git, files, prompts)
- LLM analysis and parsing
- Embedding generation and caching
- Multi-repo coordination
- Error handling and recovery

See [TESTING_STRATEGY.md](TESTING_STRATEGY.md) for detailed test plans.

---

## 📊 Success Metrics

### Phase 1 (MVP)
- [ ] Detects obvious duplicates >80% of time
- [ ] Files ideas automatically with >90% accuracy
- [ ] Zero unhandled exceptions
- [ ] Setup time <10 minutes

### Phase 2 (Enhanced)
- [ ] Analysis <5s for 100 issues
- [ ] False positive rate <10%
- [ ] Cache hit rate >70%
- [ ] User satisfaction 7/10+

### Phase 3 (Multi-Repo)
- [ ] Works with 3-5 repo groups
- [ ] Cross-repo duplicate detection >80%
- [ ] No performance degradation

### Phase 4 (Production)
- [ ] Test coverage >80%
- [ ] Zero critical bugs in 1 week
- [ ] Documentation complete
- [ ] Ready for team rollout

---

## 🔧 Technology Stack

### Core Dependencies
- **amplifier-core** - Amplifier kernel
- **issue-manager** - Storage and CRUD (Paul's module)
- **openai** - LLM and embeddings
- **pyyaml** - Configuration
- **numpy** - Vector operations
- **sqlite3** - Embedding cache

### Development Tools
- **pytest** - Testing framework
- **pytest-asyncio** - Async tests
- **pytest-cov** - Coverage reporting
- **black** - Code formatting
- **ruff** - Linting
- **mypy** - Type checking

---

## 📈 Project Timeline

### Week 1-2: Phase 1 MVP
- Set up project structure
- Implement core hooks
- Basic LLM analysis
- Integration with issue-manager
- **Milestone**: Working duplicate detection

### Week 3: Phase 2 Enhanced
- Add embeddings
- Build cache layer
- Performance optimization
- **Milestone**: <5s analysis time

### Week 4: Phase 3 Multi-Repo
- Project groups
- Multi-repo querying
- Enhanced tool operations
- **Milestone**: Cross-repo coordination

### Week 5: Phase 4 Polish
- Error handling
- Full test suite
- Documentation
- Release prep
- **Milestone**: v1.0.0 release

**Total**: 4-5 weeks for complete implementation

---

## 🎯 Key Decisions Made

### ✅ Use issue-manager (Not Beads)
- Native Amplifier module
- Teammate collaboration
- 60% effort reduction
- Pure Python

### ✅ Two-Phase Analysis
- Embeddings pre-filter (fast)
- LLM reasoning (accurate)
- Best of both worlds

### ✅ Silent Notifications
- Only high-confidence (>0.85)
- Less intrusive
- Better UX

### ✅ Git-Based Sync
- Simple, no infrastructure
- Works with existing workflow
- Can add real-time later if needed

---

## 🤝 Collaboration

### With Paul Payne
- Using his issue-manager module
- Can contribute enhancements
- Coordinate on features

### With Team
- Share profiles and config
- Collaborate via git
- Multi-repo project groups

---

## 📝 Next Steps

### Immediate Actions
1. **Review specifications** - Make sure design meets needs
2. **Set up repository** - Create module structure
3. **Start Task 1.1** - Module scaffold (2 hours)

### First Week Goals
- Complete Module 1 (hooks-activity-tracker scaffold)
- Complete Module 2 (ActivityAnalyzer basic)
- First integration test passing
- Can detect duplicates with simple LLM

### Questions to Resolve
- [ ] LLM provider configuration details
- [ ] OpenAI API key management
- [ ] Repository location decisions
- [ ] Team coordination approach

---

## 📂 File Manifest

**Planning Documents** (All in `C:\ANext\activity-tracker\`):
- ✅ README.md (this file) - Project overview
- ✅ SPECIFICATION_V2.md - Technical specification (use this)
- ✅ SPECIFICATION.md - Original spec with Beads (archive)
- ✅ TASKS.md - Implementation task list
- ✅ TESTING_STRATEGY.md - Testing approach
- ✅ PROJECT_STRUCTURE.md - Architecture reference

**Git Repositories**:
- ✅ amplifier/ - Cloned Amplifier repo (reference)
- ✅ payne-amplifier/ - Paul's modules (reference)

---

## 🎓 Learning Resources

### Amplifier Architecture
- `amplifier/docs/REPOSITORY_RULES.md` - Module boundaries
- `amplifier/docs/MODULES.md` - Available modules
- `amplifier/docs/MODULE_DEVELOPMENT.md` - How to build modules

### issue-manager Reference
- `payne-amplifier/max_payne_collection/modules/issue-manager/` - Source code
- `payne-amplifier/max_payne_collection/modules/tool-issue/` - Tool wrapper

### Implementation Philosophy
- @foundation:context/IMPLEMENTATION_PHILOSOPHY.md - Ruthless simplicity
- @foundation:context/MODULAR_DESIGN_PHILOSOPHY.md - Bricks and studs

---

## 🚦 Status Board

### Planning
- ✅ Requirements gathered
- ✅ Architecture designed
- ✅ Tasks broken down
- ✅ Testing strategy defined
- ✅ Documentation created

### Implementation
- ⬜ Phase 1: MVP (not started)
- ⬜ Phase 2: Enhanced (not started)
- ⬜ Phase 3: Multi-Repo (not started)
- ⬜ Phase 4: Polish (not started)

### Ready to Begin! 🚀

---

## 💡 Pro Tips

### Development Workflow
1. **Read SPECIFICATION_V2.md first** - Understand the design
2. **Follow TASKS.md sequentially** - Dependencies matter
3. **Write tests alongside code** - Don't defer testing
4. **Use issue-manager for tracking** - Dogfood your own tool!

### Common Pitfalls to Avoid
- ❌ Don't skip error handling
- ❌ Don't test implementation details
- ❌ Don't over-engineer Phase 1
- ❌ Don't ignore performance early
- ✅ Keep it simple (ruthless simplicity!)

### Getting Unstuck
1. Check SPECIFICATION_V2.md for design details
2. Review PROJECT_STRUCTURE.md for examples
3. Look at issue-manager source code
4. Ask Paul about issue-manager specifics
5. Review Amplifier module examples

---

## 📞 Support

### Documentation
- All specs and guides in this directory
- Amplifier docs in `amplifier/docs/`
- issue-manager in `payne-amplifier/`

### Code Examples
- Look at existing Amplifier modules
- Reference Paul's issue-manager
- Check tool-issue for tool patterns

---

**Last Updated**: 2025-11-20  
**Next Milestone**: Phase 1 MVP Complete  
**Estimated Completion**: 4-5 weeks from start

**Let's build this! 🚀**
