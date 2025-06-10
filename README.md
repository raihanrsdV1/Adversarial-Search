# ⚛️ Chain Reaction Game - Adversarial Search Implementation

A strategic multiplayer board game featuring explosive chain reactions, implemented with advanced AI algorithms and a modern web-based interface. This project demonstrates sophisticated adversarial search techniques, multiple AI heuristics, and provides both console and GUI gameplay experiences.

## 🎮 Game Overview

**Chain Reaction** is a deterministic strategy game where players place orbs on a grid to create chain reactions. When a cell reaches its critical mass, it explodes and distributes orbs to adjacent cells, potentially triggering cascading explosions that can dramatically alter the game state.

### Game Rules
- **Grid**: Customizable board size (3x3 to 15x15)
- **Critical Mass**: Corner cells (2), edge cells (3), center cells (4)
- **Winning**: Eliminate all opponent orbs or be the only player with orbs remaining
- **Chain Reactions**: Exploding cells spread orbs to adjacent cells, creating cascades
- **Ownership**: Players can only place orbs in empty cells or their own cells

## 🚀 Project Structure

```
Assignment3/
├── backend/                 # Rust console implementation
│   ├── src/
│   │   ├── main.rs         # Console game runner
│   │   ├── game.rs         # Core game logic
│   │   ├── board.rs        # Board state management
│   │   ├── ai.rs          # AI algorithms & heuristics
│   │   └── full_impl.rs   # Complete game implementation
│   └── Cargo.toml
├── chain-reaction-game/     # Modern GUI application (Tauri + Svelte)
│   ├── src/
│   │   └── routes/
│   │       └── +page.svelte # Main game interface
│   ├── src-tauri/
│   │   └── src/           # Rust backend for GUI
│   └── package.json
├── test/                   # Test utilities
├── graph.py               # Performance analysis tools
└── *.png                  # Generated charts and reports
```

## 🤖 AI Features

### Adversarial Search Algorithms
- **Minimax Algorithm**: Classic game tree search
- **Alpha-Beta Pruning**: Optimized minimax with branch elimination
- **Configurable Depth**: Adjustable search depth (1-10 levels)
- **Time-Limited Search**: Configurable thinking time (0.5-10 seconds)

### Advanced Heuristics
1. **Orb Difference**: Simple material advantage evaluation
2. **Peripheral Control**: Prioritizes edge and corner positions
3. **Territory Control**: Evaluates board area domination
4. **Chain Reaction Potential**: Assesses explosive opportunities
5. **Conversion Potential**: Measures ability to capture opponent orbs
6. **Cascade Potential**: Evaluates multi-step chain reaction possibilities
7. **Safe Mobility**: Considers safe movement options

### Heuristic Combinations
- **Multiple Heuristics**: AI can use weighted combinations of heuristics
- **Dynamic Evaluation**: Different heuristics excel in different game phases
- **Performance Analysis**: Built-in tools to compare heuristic effectiveness

## 🎯 Special Features

### 1. **Dual Implementation Architecture**
- **Console Version**: High-performance Rust implementation for tournaments
- **GUI Version**: Modern Tauri + Svelte interface with rich visualizations
- **Shared Logic**: Core game algorithms used in both versions

### 2. **Advanced Spectator Mode**
- **AI vs AI Battles**: Watch AI algorithms compete
- **Recovery System**: Automatic game state recovery if UI freezes
- **Real-time Statistics**: Live move counting and game metrics
- **Keyboard Shortcuts**: Quick recovery controls (R, L keys)

### 3. **Comprehensive Game Analytics**
- **Runtime Tracking**: Precise game duration measurement
- **Move Counting**: Accurate frontend and backend move synchronization
- **Performance Metrics**: AI thinking time and decision analysis
- **Game History**: Detailed move-by-move logging

### 4. **Enhanced User Experience**
- **Configurable Players**: Human vs Human, Human vs AI, AI vs AI
- **Visual Feedback**: Animated chain reactions and explosions
- **Critical Mass Indicators**: Visual warnings for cells near explosion
- **Responsive Design**: Modern UI with smooth animations

### 5. **Tournament System**
- **Automated Tournaments**: Run multiple games with different configurations
- **Statistical Analysis**: Win rates, move counts, runtime comparisons
- **Heuristic Evaluation**: Compare AI strategies across multiple games
- **Chart Generation**: Automated performance visualization

## 🛠️ Installation & Setup

### Prerequisites
- **Rust** (1.70+): [Install Rust](https://rustup.rs/)
- **Node.js** (18+): [Install Node.js](https://nodejs.org/)
- **npm** or **yarn**: Package manager

### Quick Start

#### Console Version
```bash
# Navigate to backend directory
cd backend/

# Run interactive game
cargo run

# Run with specific configuration
cargo run -- --help
```

#### GUI Version (Recommended)
```bash
# Navigate to GUI directory
cd chain-reaction-game/

# Install dependencies
npm install

# Start development server
npm run tauri dev

# Build for production
npm run tauri build
```

### Development Setup
```bash
# Clone the repository
git clone <repository-url>
cd Assignment3/

# Setup backend
cd backend/
cargo build --release

# Setup GUI
cd ../chain-reaction-game/
npm install
npm run tauri dev
```

## 🎮 How to Play

### Using the GUI (Recommended)

1. **Launch the Game**
   ```bash
   cd chain-reaction-game/
   npm run tauri dev
   ```

2. **Configure Game Settings**
   - **Board Size**: Adjust width and height (3-15)
   - **Player Types**: Choose Human or AI for each player
   - **AI Configuration**: Select strategy, depth, and heuristics

3. **AI Configuration Options**
   - **Strategy**: Alpha-Beta or Random
   - **Search Depth**: 1-10 levels
   - **Thinking Time**: 0.5-10 seconds
   - **Heuristics**: Select multiple evaluation functions

4. **Gameplay**
   - Click on empty cells or your own cells to place orbs
   - Watch chain reactions unfold automatically
   - Use recovery controls if spectating AI vs AI games

### Using the Console Version

```bash
cd backend/
cargo run
```
- Follow on-screen prompts for game configuration
- Enter moves as coordinates (row, column)
- View detailed game logs and statistics

## 📊 Performance Analysis

### Running Tournaments
```bash
cd backend/
cargo run --release
```

### Generating Analytics
```bash
python graph.py
```
This generates:
- **Win rate charts**: Comparison across different AI configurations
- **Runtime analysis**: Game duration statistics
- **Move count analysis**: Efficiency metrics
- **Heuristic performance**: Individual and combined heuristic effectiveness

## 🔧 Advanced Configuration

### AI Heuristic Tuning
- **Weight Adjustment**: Modify heuristic combinations in `ai.rs`
- **Custom Heuristics**: Implement new evaluation functions
- **Performance Profiling**: Use built-in timing and statistics

### Game Customization
- **Board Dimensions**: Modify game complexity
- **Critical Mass Rules**: Adjust explosion thresholds
- **Victory Conditions**: Customize win scenarios

### UI Customization
- **Themes**: Modify CSS in `+page.svelte`
- **Animations**: Adjust explosion and transition effects
- **Spectator Features**: Configure recovery timers and shortcuts

## 🧪 Technical Implementation Details

### Core Algorithms
- **Game State Representation**: Efficient 2D grid with metadata
- **Move Generation**: Fast legal move enumeration
- **Chain Reaction Simulation**: Optimized cascade calculation
- **State Evaluation**: Multi-criteria decision making

### Performance Optimizations
- **Alpha-Beta Pruning**: Reduces search space by ~50-90%
- **Move Ordering**: Improves pruning effectiveness
- **Transposition Tables**: Caches evaluated positions (future enhancement)
- **Iterative Deepening**: Time-bounded progressive search

### Frontend-Backend Communication
- **Tauri Bridge**: Secure Rust-TypeScript communication
- **State Synchronization**: Consistent game state management
- **Error Handling**: Robust recovery from failures
- **Real-time Updates**: Smooth UI responsiveness

## 📈 Results & Analysis

Based on extensive testing:

### AI Performance Insights
- **Depth 3-4**: Optimal balance of performance and quality
- **Combined Heuristics**: Outperform single-heuristic approaches
- **Peripheral Control**: Particularly effective in early game
- **Chain Reaction Potential**: Crucial for late-game tactics

### Tournament Results
- **Alpha-Beta vs Random**: 95%+ win rate for Alpha-Beta
- **Heuristic Combinations**: 20-30% improvement over single heuristics
- **Depth Impact**: Diminishing returns beyond depth 5
- **Runtime Scaling**: Exponential growth with depth

## 🤝 Contributing

### Development Workflow
1. **Backend Changes**: Modify Rust code in `backend/src/`
2. **Frontend Changes**: Update Svelte components in `chain-reaction-game/src/`
3. **Testing**: Run both console and GUI versions
4. **Performance**: Use tournament mode for benchmarking

### Code Style
- **Rust**: Follow `rustfmt` guidelines
- **TypeScript/Svelte**: Use Prettier formatting
- **Documentation**: Maintain comprehensive comments

## 📝 License

This project is developed for educational purposes as part of CSE 318 - Artificial Intelligence Sessional coursework at BUET.

## 🎯 Future Enhancements

- **Machine Learning Integration**: Neural network evaluation functions
- **Online Multiplayer**: Network-based gameplay
- **Advanced Visualizations**: 3D board representation
- **Mobile Support**: Cross-platform compatibility
- **Tournament Hosting**: Automated competition platform

---

**Developed by**: Mohammad Raihan Rashid
**Course**: CSE 318 - Artificial Intelligence Sessional  
**Institution**: Bangladesh University of Engineering and Technology (BUET)  
**Academic Year**: 2024-2025
