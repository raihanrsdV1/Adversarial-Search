<svelte:head>
  <script src="https://cdn.jsdelivr.net/npm/canvas-confetti@1.9.3/dist/confetti.browser.min.js"></script>
</svelte:head>

<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, tick } from "svelte";

  // --- Type Definitions (matching Rust DTOs) ---
  interface AIConfigData {
    strategy: string;
    depth: number;
    heuristics: string[];
    time_limit_ms: number;
  }

  interface PlayerConfigData {
    player_type: string;
    name: string;
    ai_config: AIConfigData | null;
  }

  interface GameConfigData {
    width: number;
    height: number;
    red_player: PlayerConfigData;
    blue_player: PlayerConfigData;
  }

  interface CellData {
    player: string | null;
    orbs: number;
    critical_mass: number;
  }

  interface GameStateData {
    board: CellData[][];
    current_player: string;
    game_status: string;
    red_orbs: number;
    blue_orbs: number;
    total_moves: number;
    winner: string | null;
  }

  // --- Reactive Svelte State ---
  let view: 'setup' | 'playing' | 'finished' = 'setup';
  let gameState: GameStateData | null = null;
  let isGameActive = false;
  let errorMessage = "";
  let gameHistory: string[] = [];
  let lastStateChange = Date.now(); // Track when game state last changed
  let recoveryTimer: number | null = null; // Timer for automatic recovery
  let gameStartTime: number = 0; // Track game start time for runtime calculation
  let frontendMoveCount: number = 0; // Frontend-controlled move counter

  // --- Default Configuration ---
  let config: GameConfigData = {
    width: 6,
    height: 9,
    red_player: { player_type: "Human", name: "Player 1", ai_config: null },
    blue_player: {
      player_type: "AI",
      name: "Computer",
      ai_config: {
        strategy: "AlphaBeta",
        depth: 3, 
        heuristics: ["OrbDifference", "PeripheralControl"],
        time_limit_ms: 2000, 
      }
    }
  };

  const availableHeuristics = [
    { value: "OrbDifference", label: "Orb Difference" },
    { value: "PeripheralControl", label: "Peripheral Control" },
    { value: "TerritoryControl", label: "Territory Control" },
    { value: "ChainReactionPotential", label: "Chain Reaction Potential" },
    { value: "ConversionPotential", label: "Conversion Potential" },
    { value: "CascadePotential", label: "Cascade Potential" },
    { value: "SafeMobility", label: "Safe Mobility" }
  ];
  
  // --- Game Logic Functions ---
  
  // Recovery mechanism - get fresh state from backend
  async function recoverGameState() {
    try {
      const spectatorMsg = isAIvsAI() ? "🎭 [Spectator] " : "";
      addToHistory(`${spectatorMsg}🔄 Recovering game state from backend...`);
      const freshState: GameStateData = await invoke("get_current_state");
      gameState = freshState;
      lastStateChange = Date.now();
      addToHistory(`${spectatorMsg}✅ Game state recovered successfully!`);
      
      // If we're in AI vs AI mode and game is still ongoing, continue the flow
      if (isAIvsAI() && gameState.game_status === 'ongoing') {
        addToHistory("🎭 Resuming AI vs AI battle...");
        continueGame();
      }
    } catch (e) {
      errorMessage = `Recovery failed: ${e}`;
      addToHistory("❌ Failed to recover game state");
    }
  }

  // Manual recovery from log file
  async function recoverFromLogFile() {
    try {
      const spectatorMsg = isAIvsAI() ? "🎭 [Spectator] " : "";
      addToHistory(`${spectatorMsg}📄 Attempting recovery from log file...`);
      const freshState: GameStateData = await invoke("recover_from_log");
      gameState = freshState;
      lastStateChange = Date.now();
      addToHistory(`${spectatorMsg}✅ Recovered from log file successfully!`);
      
      // If we're in AI vs AI mode and game is still ongoing, continue the flow
      if (isAIvsAI() && gameState.game_status === 'ongoing') {
        addToHistory("🎭 Resuming AI vs AI battle from saved state...");
        continueGame();
      }
    } catch (e) {
      errorMessage = `Log recovery failed: ${e}`;
      addToHistory("❌ Failed to recover from log file");
    }
  }

  // Start recovery timer
  function startRecoveryTimer() {
    if (recoveryTimer) clearInterval(recoveryTimer);
    recoveryTimer = setInterval(() => {
      if (isGameActive && view === 'playing' && gameState) {
        const timeSinceChange = Date.now() - lastStateChange;
        // Shorter timeout for AI vs AI games since spectators want smooth experience
        const timeout = isAIvsAI() ? 15000 : 20000; // 15s for AI vs AI, 20s for human games
        if (timeSinceChange > timeout) {
          const spectatorMsg = isAIvsAI() ? "🎭 [Auto-Recovery] " : "";
          addToHistory(`${spectatorMsg}⚠️ Game appears frozen, attempting automatic recovery...`);
          recoverGameState();
        }
      }
    }, 5000); // Check every 5 seconds
  }

  // Stop recovery timer
  function stopRecoveryTimer() {
    if (recoveryTimer) {
      clearInterval(recoveryTimer);
      recoveryTimer = null;
    }
  }
  
  async function animateBoard(history: GameStateData[]) {
    for (const frame of history) {
      gameState = frame;
      lastStateChange = Date.now(); // Update last change time
      await new Promise(resolve => setTimeout(resolve, 150));
    }
    // After animation, continue game flow
    continueGame();
  }
  
  async function startGame() {
    isGameActive = true;
    errorMessage = "";
    lastStateChange = Date.now();
    gameStartTime = Date.now(); // Record game start time
    frontendMoveCount = 0; // Reset frontend move counter
    
    try {
      gameHistory = [];
      const initialState: GameStateData = await invoke("start_game", { config });
      gameState = initialState;
      view = 'playing';
      addToHistory(`Game started! ${getPlayerName(initialState.current_player)}'s turn`);
      
      // Start recovery monitoring
      startRecoveryTimer();
      
      // Start the game flow
      continueGame();
    } catch (e) {
      errorMessage = `Failed to start game: ${e}`;
      isGameActive = false;
      stopRecoveryTimer();
    }
  }

  async function handleHumanMove(row: number, col: number) {
    // Only humans can call this, so no need for complex checks
    if (!isGameActive || view !== 'playing' || !gameState) return;
    
    addToHistory(`${getPlayerName(gameState.current_player)} played at (${row}, ${col})`);
    try {
      const history: GameStateData[] = await invoke("make_move", { row, col });
      frontendMoveCount++; // Increment move counter after successful move
      await animateBoard(history);
    } catch (e) {
      errorMessage = `Invalid Move: ${e}`;
      setTimeout(() => errorMessage = "", 3000);
    }
  }

  async function playAITurn() {
    if (!isGameActive || !gameState || view !== 'playing') return;
    
    addToHistory(`${getPlayerName(gameState.current_player)} is thinking...`);
    await tick();

    try {
      const aiMove = await invoke("get_ai_move_command") as [number, number];
      const [row, col] = aiMove;
      addToHistory(`AI played at (${row}, ${col})`);
      const history: GameStateData[] = await invoke("make_move", { row, col });
      frontendMoveCount++; // Increment move counter after successful AI move
      await animateBoard(history);
    } catch (e) {
      errorMessage = `AI Error: ${e}`;
      isGameActive = false;
    }
  }

  // Simple game flow - no race conditions
  function continueGame() {
    if (!gameState || !isGameActive) return;
    
    if (gameState.game_status === 'finished') {
      isGameActive = false;
      stopRecoveryTimer();
      view = 'finished';
      addToHistory(`🎉 ${getPlayerName(gameState.winner)} wins the game!`);
      addToHistory(`📊 Game completed in ${frontendMoveCount} moves over ${formatGameRuntime()}`);
      addToHistory(`🔴 Red: ${gameState.red_orbs} orbs | 🔵 Blue: ${gameState.blue_orbs} orbs`);
      
      // Add player configuration summary
      const redType = config.red_player.player_type === 'AI' ? 'AI' : 'Human';
      const blueType = config.blue_player.player_type === 'AI' ? 'AI' : 'Human';
      addToHistory(`👥 Players: ${config.red_player.name} (${redType}) vs ${config.blue_player.name} (${blueType})`);
      
      // Add AI heuristics if applicable
      if (config.red_player.player_type === 'AI' && config.red_player.ai_config) {
        const redHeuristics = config.red_player.ai_config.heuristics.map(h => getHeuristicLabel(h)).join(', ');
        addToHistory(`🔴 Red AI: ${config.red_player.ai_config.strategy}, Depth ${config.red_player.ai_config.depth}, Heuristics: ${redHeuristics}`);
      }
      if (config.blue_player.player_type === 'AI' && config.blue_player.ai_config) {
        const blueHeuristics = config.blue_player.ai_config.heuristics.map(h => getHeuristicLabel(h)).join(', ');
        addToHistory(`🔵 Blue AI: ${config.blue_player.ai_config.strategy}, Depth ${config.blue_player.ai_config.depth}, Heuristics: ${blueHeuristics}`);
      }
      
      setTimeout(launchConfetti, 100);
      return;
    }
    
    // Check if current player is AI and play immediately
    if (isCurrentPlayerAI(gameState)) {
      // Small delay for UI updates, then play AI turn
      setTimeout(() => playAITurn(), 200);
    } else {
      // Human turn - just wait for click
      addToHistory(`${getPlayerName(gameState.current_player)}'s turn.`);
    }
  }
  
  function launchConfetti() {
    const confetti = (window as any).confetti;
    if (confetti) {
        confetti({
            particleCount: 150,
            spread: 90,
            origin: { y: 0.6 }
        });
    }
  }

  function resetGame() {
    view = 'setup';
    gameState = null;
    errorMessage = "";
    isGameActive = false;
    gameHistory = [];
    lastStateChange = Date.now();
    gameStartTime = 0; // Reset game start time
    frontendMoveCount = 0; // Reset frontend move counter
    stopRecoveryTimer();
  }
  
  // --- UI Helper Functions ---
  function formatGameRuntime(): string {
    if (gameStartTime === 0) return "0s";
    const runtimeMs = Date.now() - gameStartTime;
    const runtimeSeconds = Math.floor(runtimeMs / 1000);
    
    if (runtimeSeconds < 60) {
      return `${runtimeSeconds}s`;
    } else if (runtimeSeconds < 3600) {
      const minutes = Math.floor(runtimeSeconds / 60);
      const seconds = runtimeSeconds % 60;
      return `${minutes}m ${seconds}s`;
    } else {
      const hours = Math.floor(runtimeSeconds / 3600);
      const minutes = Math.floor((runtimeSeconds % 3600) / 60);
      const seconds = runtimeSeconds % 60;
      return `${hours}h ${minutes}m ${seconds}s`;
    }
  }

  function isCurrentPlayerAI(state: GameStateData | null): boolean {
    if (!state) return false;
    const playerConfig = state.current_player === 'Red' ? config.red_player : config.blue_player;
    return playerConfig.player_type === "AI";
  }
  
  function isAIvsAI(): boolean {
    return config.red_player.player_type === "AI" && config.blue_player.player_type === "AI";
  }
  
  function getPlayerName(color: string | null | undefined): string {
    if (!color) return "";
    return color === 'Red' ? config.red_player.name : config.blue_player.name;
  }

  function getHeuristicLabel(heuristicValue: string): string {
    const heuristic = availableHeuristics.find(h => h.value === heuristicValue);
    return heuristic ? heuristic.label : heuristicValue;
  }

  function canMakeMove(cell: CellData): boolean {
    // Simple logic: game active, human's turn, valid cell
    return isGameActive && 
           view === 'playing' && 
           gameState !== null && 
           !isCurrentPlayerAI(gameState) &&
           (cell.player === null || cell.player === gameState.current_player);
  }
  
  function getCellClass(cell: CellData): string {
    if (!cell) return "cell";
    let classes = "cell";
    if (cell.player === "Red") classes += " red";
    else if (cell.player === "Blue") classes += " blue";
    else classes += " empty";
    
    // Only show clickable if human can actually make a move
    if (canMakeMove(cell)) {
      classes += " clickable";
    }
    
    if (cell.orbs >= cell.critical_mass && cell.player) {
        classes += " critical";
    }
    return classes;
  }
  
  function addToHistory(message: string) {
    gameHistory = [`${new Date().toLocaleTimeString()}: ${message}`, ...gameHistory];
  }

  function handlePlayerTypeChange(playerKey: 'red_player' | 'blue_player') {
      if (config[playerKey].player_type === 'AI' && !config[playerKey].ai_config) {
          config[playerKey].ai_config = { 
              strategy: 'AlphaBeta', 
              depth: 3, 
              heuristics: ['OrbDifference', 'PeripheralControl'],
              time_limit_ms: 2000,
          };
      }
      config = {...config};
  }

  function toggleHeuristic(playerKey: 'red_player' | 'blue_player', heuristic: string) {
      const targetPlayer = config[playerKey];
      if (targetPlayer.ai_config) {
          const heuristics = targetPlayer.ai_config.heuristics;
          if (heuristics.includes(heuristic)) {
              targetPlayer.ai_config.heuristics = heuristics.filter(h => h !== heuristic);
          } else {
              targetPlayer.ai_config.heuristics.push(heuristic);
          }
          config = {...config};
      }
  }

  // --- Keyboard shortcuts for spectating ---
  function handleKeydown(event: KeyboardEvent) {
    // Only enable shortcuts during AI vs AI spectating
    if (!isAIvsAI() || view !== 'playing') return;
    
    if (event.key === 'r' || event.key === 'R') {
      event.preventDefault();
      recoverGameState();
      addToHistory("🎮 Quick recovery triggered via keyboard (R)");
    } else if (event.key === 'l' || event.key === 'L') {
      event.preventDefault();
      recoverFromLogFile();
      addToHistory("🎮 Log recovery triggered via keyboard (L)");
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    return () => {
      window.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

<main class="container">
  <header>
    <h1>⚛️ Chain Reaction Game</h1>
    <p>A strategic board game with explosive chain reactions!</p>
  </header>

  {#if view === 'setup'}
    <div class="setup-panel">
      <h2 class="setup-title">Game Configuration</h2>
      
      <div class="board-config">
        <h3>Board Dimensions</h3>
        <div class="form-row">
            <label class="depth-slider">Width: <span>{config.width}</span>
                <input type="range" bind:value={config.width} min="3" max="15" />
            </label>
            <label class="depth-slider">Height: <span>{config.height}</span>
                <input type="range" bind:value={config.height} min="3" max="15" />
            </label>
        </div>
      </div>

      <div class="config-grid">
        <div class="config-section red">
          <h3>Player 1 (Red)</h3>
          <div class="form-row">
            <label>Name: <input type="text" bind:value={config.red_player.name} /></label>
            <label>Type: 
              <select bind:value={config.red_player.player_type} on:change={() => handlePlayerTypeChange('red_player')}>
                <option value="Human">Human</option>
                <option value="AI">AI</option>
              </select>
            </label>
          </div>
          {#if config.red_player.player_type === 'AI' && config.red_player.ai_config}
            <div class="ai-config">
              <label>Strategy: 
                <select bind:value={config.red_player.ai_config.strategy}>
                  <option value="AlphaBeta">Alpha-Beta</option>
                  <option value="Random">Random</option>
                </select>
              </label>
              <label class="depth-slider">Max Depth: <span>{config.red_player.ai_config.depth}</span>
                <input type="range" bind:value={config.red_player.ai_config.depth} min="1" max="10" />
              </label>
              <label class="depth-slider">Thinking Time: <span>{config.red_player.ai_config.time_limit_ms / 1000}s</span>
                <input type="range" bind:value={config.red_player.ai_config.time_limit_ms} min="500" max="10000" step="500" />
              </label>
              <h4>Heuristics</h4>
              <div class="heuristics-grid">
                {#each availableHeuristics as heuristic}
                  <label class="heuristic-checkbox">
                    <input type="checkbox" checked={config.red_player.ai_config.heuristics.includes(heuristic.value)} on:change={() => toggleHeuristic('red_player', heuristic.value)} />
                    {heuristic.label}
                  </label>
                {/each}
              </div>
            </div>
          {/if}
        </div>

        <div class="config-section blue">
          <h3>Player 2 (Blue)</h3>
          <div class="form-row">
            <label>Name: <input type="text" bind:value={config.blue_player.name} /></label>
            <label>Type: 
              <select bind:value={config.blue_player.player_type} on:change={() => handlePlayerTypeChange('blue_player')}>
                <option value="Human">Human</option>
                <option value="AI">AI</option>
              </select>
            </label>
          </div>
          {#if config.blue_player.player_type === 'AI' && config.blue_player.ai_config}
            <div class="ai-config">
              <label>Strategy: 
                <select bind:value={config.blue_player.ai_config.strategy}>
                  <option value="AlphaBeta">Alpha-Beta</option>
                  <option value="Random">Random</option>
                </select>
              </label>
              <label class="depth-slider">Max Depth: <span>{config.blue_player.ai_config.depth}</span>
                <input type="range" bind:value={config.blue_player.ai_config.depth} min="1" max="10" />
              </label>
              <label class="depth-slider">Thinking Time: <span>{config.blue_player.ai_config.time_limit_ms / 1000}s</span>
                <input type="range" bind:value={config.blue_player.ai_config.time_limit_ms} min="500" max="10000" step="500" />
              </label>
              <h4>Heuristics</h4>
              <div class="heuristics-grid">
                {#each availableHeuristics as heuristic}
                  <label class="heuristic-checkbox">
                    <input type="checkbox" checked={config.blue_player.ai_config.heuristics.includes(heuristic.value)} on:change={() => toggleHeuristic('blue_player', heuristic.value)} />
                    {heuristic.label}
                  </label>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      </div>
      
      <button class="start-button" on:click={startGame} disabled={isGameActive}>
        {#if isGameActive}Loading...{:else}🚀 Start Game{/if}
      </button>
    </div>
  {:else if view === 'playing' && gameState}
    <div class="game-panel">
      <div class="game-info">
        <div class="status-bar">
          <div class="current-turn">
            <span class="turn-indicator {gameState.current_player.toLowerCase()}">
              {getPlayerName(gameState.current_player)}'s Turn
            </span>
            {#if isGameActive && isCurrentPlayerAI(gameState)}
              <span class="thinking">🤔 Thinking...</span>
            {/if}
            {#if recoveryTimer && isAIvsAI()}
              <span class="recovery-monitor">🛡️ Auto-recovery active</span>
            {/if}
          </div>
          <div class="score-board">
            <div class="score red">🔴 {gameState.red_orbs}</div>
            <div class="score blue">🔵 {gameState.blue_orbs}</div>
          </div>
          <div class="move-counter">Move: {frontendMoveCount}</div>
        </div>
        {#if errorMessage}
          <div class="error">{errorMessage}</div>
        {/if}
      </div>
      <div class="game-board" style="--grid-width: {config.width}; --grid-height: {config.height};">
        {#each gameState.board as row, r}
          <div class="board-row">
            {#each row as cell, c}
              <button 
                class={getCellClass(cell)}
                disabled={!canMakeMove(cell)}
                on:click={() => handleHumanMove(r, c)}
              >
                {#if cell.player}
                  <div class="orb-container" style="--orb-count: {cell.orbs}">
                    {#each Array(cell.orbs) as _}
                      <div class="orb"></div>
                    {/each}
                  </div>
                {/if}
                <div class="critical-mass">{cell.critical_mass}</div>
              </button>
            {/each}
          </div>
        {/each}
      </div>
      <div class="game-controls">
        {#if isAIvsAI()}
          <div class="spectator-info">
            <span class="spectator-badge">👁️ Spectating AI vs AI</span>
          </div>
          <div class="spectator-controls">
            <h4>🛡️ Recovery Controls</h4>
            <div class="recovery-buttons">
              <button class="control-button recovery primary" on:click={recoverGameState} 
                      title="Get fresh state from backend - Use if game seems stuck">
                🔧 Quick Recover
              </button>
              <button class="control-button recovery" on:click={recoverFromLogFile}
                      title="Restore from logged board state - Use if corruption occurred">
                📄 Log Recover
              </button>
            </div>
            <div class="spectator-shortcuts">
              <small>⌨️ Shortcuts: <kbd>R</kbd> Quick Recover • <kbd>L</kbd> Log Recover</small>
            </div>
          </div>
        {:else}
          <button class="control-button recovery" on:click={recoverGameState} 
                  title="Get fresh state from backend">🔧 Recover State</button>
          <button class="control-button recovery" on:click={recoverFromLogFile}
                  title="Restore from logged board state">📄 Recover from Log</button>
        {/if}
        <button class="control-button" on:click={resetGame}>🔄 New Game</button>
      </div>
    </div>
    <div class="history-panel">
      <h3>Game History</h3>
      <div class="history-log">
        {#each gameHistory as entry (entry)}
          <div class="history-entry">{entry}</div>
        {/each}
      </div>
    </div>
  {:else if view === 'finished' && gameState}
     <div class="game-over-panel">
        <h2 class="winner-text">🎉 {getPlayerName(gameState.winner)} Wins! 🎉</h2>
        <div class="game-stats">
          <div class="stat-item">
            <span class="stat-label">🎯 Total Moves:</span>
            <span class="stat-value">{frontendMoveCount}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">⏱️ Game Runtime:</span>
            <span class="stat-value">{formatGameRuntime()}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">🔴 Red Orbs:</span>
            <span class="stat-value">{gameState.red_orbs}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">🔵 Blue Orbs:</span>
            <span class="stat-value">{gameState.blue_orbs}</span>
          </div>
        </div>
        
        <div class="player-configs">
          <div class="player-config red">
            <h4>🔴 {config.red_player.name}</h4>
            <div class="config-details">
              <div class="player-type">
                {config.red_player.player_type === 'AI' ? '🤖 AI Player' : '👤 Human Player'}
              </div>
              {#if config.red_player.player_type === 'AI' && config.red_player.ai_config}
                <div class="ai-details">
                  <div class="strategy">Strategy: {config.red_player.ai_config.strategy}</div>
                  <div class="depth">Depth: {config.red_player.ai_config.depth}</div>
                  <div class="thinking-time">Time: {config.red_player.ai_config.time_limit_ms / 1000}s</div>
                  <div class="heuristics">
                    <span class="heuristics-label">Heuristics:</span>
                    <div class="heuristics-list">
                      {#each config.red_player.ai_config.heuristics as heuristic}
                        <span class="heuristic-tag">{getHeuristicLabel(heuristic)}</span>
                      {/each}
                    </div>
                  </div>
                </div>
              {/if}
            </div>
          </div>
          
          <div class="player-config blue">
            <h4>🔵 {config.blue_player.name}</h4>
            <div class="config-details">
              <div class="player-type">
                {config.blue_player.player_type === 'AI' ? '🤖 AI Player' : '👤 Human Player'}
              </div>
              {#if config.blue_player.player_type === 'AI' && config.blue_player.ai_config}
                <div class="ai-details">
                  <div class="strategy">Strategy: {config.blue_player.ai_config.strategy}</div>
                  <div class="depth">Depth: {config.blue_player.ai_config.depth}</div>
                  <div class="thinking-time">Time: {config.blue_player.ai_config.time_limit_ms / 1000}s</div>
                  <div class="heuristics">
                    <span class="heuristics-label">Heuristics:</span>
                    <div class="heuristics-list">
                      {#each config.blue_player.ai_config.heuristics as heuristic}
                        <span class="heuristic-tag">{getHeuristicLabel(heuristic)}</span>
                      {/each}
                    </div>
                  </div>
                </div>
              {/if}
            </div>
          </div>
        </div>
        
        <button class="control-button" on:click={resetGame}>🔄 Play Again</button>
     </div>
  {/if}
</main>

<style>
  :root { 
    --red-player: #e74c3c; 
    --blue-player: #3498db; 
    --cell-size: 50px;
  }
  .container { max-width: 1200px; margin: 0 auto; padding: 20px; font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; }
  header { text-align: center; margin-bottom: 30px; }
  header h1 { color: #2c3e50; margin-bottom: 10px; font-size: 2.5em; }
  header p { color: #7f8c8d; font-size: 1.1em; }
  .setup-panel, .game-panel, .game-over-panel { background: white; padding: 30px; border-radius: 12px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); max-width: 800px; margin: 0 auto; }
  .setup-title { text-align: center; margin-bottom: 25px; color: #2c3e50; font-size: 1.5em; }
  
  /* Styles for the new board configuration section */
  .board-config {
    margin-bottom: 30px;
    padding: 20px;
    background-color: #f8f9fa;
    border-radius: 8px;
    border: 1px solid #e1e8ed;
  }
  .board-config h3 {
    margin: 0 0 15px 0;
    text-align: center;
    color: #2c3e50;
  }
  .board-config .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
  }

  .config-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 25px; margin-bottom: 30px; }
  .config-section { padding: 20px; background: #f8f9fa; border-radius: 8px; border-left: 4px solid; }
  .config-section.red { border-color: var(--red-player); }
  .config-section.blue { border-color: var(--blue-player); }
  .config-section h3 { margin: 0 0 15px 0; color: #2c3e50; font-size: 1.2em; }
  .form-row, .ai-config { display: flex; flex-direction: column; gap: 15px; }
  .form-row label, .ai-config label { display: flex; flex-direction: column; gap: 5px; font-weight: 500; color: #34495e; }
  .form-row input, .form-row select { padding: 8px 12px; border: 2px solid #e1e8ed; border-radius: 6px; font-size: 14px; transition: border-color 0.2s; }
  .form-row input:focus, .form-row select:focus { border-color: #3498db; outline: none; }
  .ai-config { margin-top: 20px; padding-top: 20px; border-top: 1px solid #e1e8ed; }
  .ai-config h4 { margin: 0 0 5px 0; color: #34495e; font-size: 1em; }
  .heuristics-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .heuristic-checkbox { display: flex; align-items: center; gap: 8px; padding: 8px; background: white; border-radius: 6px; cursor: pointer; transition: background-color 0.2s; font-size: 13px; border: 1px solid #e1e8ed; }
  .heuristic-checkbox:hover { background: #ecf0f1; }
  .heuristic-checkbox input { accent-color: #3498db; }
  .depth-slider span { font-weight: bold; color: #3498db; }
  .start-button { display: block; width: 100%; padding: 15px; background: linear-gradient(135deg, #3498db, #2980b9); color: white; border: none; border-radius: 8px; font-size: 1.2em; font-weight: bold; cursor: pointer; transition: all 0.2s; }
  .start-button:hover { transform: translateY(-2px); box-shadow: 0 6px 12px rgba(52, 152, 219, 0.3); }
  .start-button:disabled { background: #95a5a6; cursor: not-allowed; }
  .game-panel { display: flex; flex-direction: column; gap: 20px; max-width: 1000px; }
  .game-info { padding: 20px; border-radius: 12px; }
  .status-bar { display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 15px; }
  .current-turn { display: flex; align-items: center; gap: 10px; }
  .turn-indicator { padding: 8px 16px; border-radius: 20px; font-weight: bold; color: white; }
  .turn-indicator.red { background: linear-gradient(135deg, var(--red-player), #c0392b); }
  .turn-indicator.blue { background: linear-gradient(135deg, var(--blue-player), #2980b9); }
  .thinking { font-style: italic; color: #7f8c8d; }
  .recovery-monitor { font-style: italic; color: #27ae60; font-size: 0.9em; }
  .score-board { display: flex; gap: 20px; }
  .score { padding: 8px 12px; border-radius: 8px; font-weight: bold; background: #f8f9fa; border: 2px solid #e1e8ed; }
  .score.red { border-color: var(--red-player); color: var(--red-player); }
  .score.blue { border-color: var(--blue-player); color: var(--blue-player); }
  .move-counter { padding: 8px 12px; background: #34495e; color: white; border-radius: 8px; font-weight: bold; }
  .error { background: #e74c3c; color: white; padding: 10px; border-radius: 6px; margin-top: 15px; text-align: center; }
  .game-board { display: flex; flex-direction: column; gap: 2px; background: #34495e; padding: 10px; border-radius: 12px; align-items: center; }
  .board-row { display: flex; gap: 2px; }
  .cell { width: var(--cell-size); height: var(--cell-size); border: none; border-radius: 6px; background: #ecf0f1; position: relative; display: flex; align-items: center; justify-content: center; cursor: not-allowed; transition: all 0.2s; }
  .cell.empty { background: #bdc3c7; }
  .cell.red { background: linear-gradient(135deg, var(--red-player), #c0392b); }
  .cell.blue { background: linear-gradient(135deg, var(--blue-player), #2980b9); }
  .cell.clickable { cursor: pointer; }
  .cell.clickable:hover { transform: scale(1.05); box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2); z-index: 10; }
  .cell.exploding { animation: explode 0.3s ease-out; }
  .cell.critical { animation: pulse 1.5s infinite; }
  @keyframes pulse { 0%, 100% { box-shadow: 0 0 0 0 rgba(241, 196, 15, 0.7); } 50% { box-shadow: 0 0 12px 6px rgba(241, 196, 15, 0); } }
  @keyframes explode { 0% { transform: scale(1); } 50% { transform: scale(1.2); } 100% { transform: scale(1); } }
  .orb-container { display: grid; grid-template-columns: repeat(2, 1fr); gap: 2px; width: 24px; height: 24px; }
  .orb { width: 8px; height: 8px; border-radius: 50%; background: white; box-shadow: inset 0 -1px 2px rgba(0,0,0,0.4); }
  .critical-mass { position: absolute; bottom: 2px; right: 4px; font-size: 10px; color: rgba(255, 255, 255, 0.7); font-weight: bold; }
  .game-controls, .game-over-panel { display: flex; justify-content: center; gap: 15px; margin-top: 20px; text-align: center; flex-direction: row; align-items: center; flex-wrap: wrap;}
  .spectator-info { width: 100%; margin-bottom: 10px; }
  .spectator-badge { background: #9b59b6; color: white; padding: 8px 16px; border-radius: 20px; font-weight: bold; font-size: 0.9em; }
  .winner-text { font-size: 2.5rem; font-weight: bold; color: #2ecc71; margin-bottom: 1rem; }
  .game-stats { display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin: 20px 0; padding: 20px; background: #f8f9fa; border-radius: 8px; border: 2px solid #e1e8ed; }
  .stat-item { display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; background: white; border-radius: 6px; }
  .stat-label { font-weight: 500; color: #34495e; }
  .stat-value { font-weight: bold; color: #2c3e50; font-size: 1.1em; }
  .player-configs { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin: 20px 0; }
  .player-config { padding: 15px; border-radius: 8px; border: 2px solid; }
  .player-config.red { border-color: var(--red-player); background: rgba(231, 76, 60, 0.05); }
  .player-config.blue { border-color: var(--blue-player); background: rgba(52, 152, 219, 0.05); }
  .player-config h4 { margin: 0 0 10px 0; font-size: 1.1em; color: #2c3e50; }
  .config-details { font-size: 0.9em; }
  .player-type { font-weight: bold; margin-bottom: 8px; color: #34495e; }
  .ai-details { margin-top: 8px; }
  .ai-details > div { margin-bottom: 4px; color: #34495e; }
  .strategy, .depth, .thinking-time { font-size: 0.85em; }
  .heuristics { margin-top: 8px; }
  .heuristics-label { font-weight: 500; color: #2c3e50; margin-bottom: 4px; display: block; }
  .heuristics-list { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 4px; }
  .heuristic-tag { background: #ecf0f1; color: #2c3e50; padding: 2px 6px; border-radius: 4px; font-size: 0.75em; border: 1px solid #bdc3c7; }
  .control-button { padding: 12px 24px; background: #95a5a6; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; transition: background 0.2s; }
  .control-button:hover { background: #7f8c8d; }
  .control-button.recovery { background: #f39c12; font-size: 0.9em; }
  .control-button.recovery:hover { background: #e67e22; }
  .control-button.recovery.primary { background: #e74c3c; font-size: 1em; padding: 14px 28px; }
  .control-button.recovery.primary:hover { background: #c0392b; transform: translateY(-1px); }
  .spectator-controls { width: 100%; margin: 10px 0; padding: 15px; background: rgba(155, 89, 182, 0.1); border-radius: 8px; border: 2px dashed #9b59b6; }
  .spectator-controls h4 { margin: 0 0 10px 0; color: #9b59b6; font-size: 0.9em; text-align: center; }
  .recovery-buttons { display: flex; gap: 10px; justify-content: center; flex-wrap: wrap; }
  .spectator-shortcuts { text-align: center; margin-top: 8px; }
  .spectator-shortcuts small { color: #7f8c8d; font-size: 0.8em; }
  .spectator-shortcuts kbd { background: #ecf0f1; border: 1px solid #bdc3c7; border-radius: 3px; padding: 2px 6px; font-size: 0.75em; color: #2c3e50; }
  .history-panel { background: white; padding: 20px; border-radius: 12px; margin-top: 20px; }
  .history-panel h3 { margin: 0 0 15px 0; color: #2c3e50; }
  .history-log { max-height: 150px; overflow-y: auto; border: 1px solid #e1e8ed; border-radius: 6px; padding: 10px; background: #f8f9fa; }
  .history-entry { padding: 4px 0; border-bottom: 1px solid #e1e8ed; font-size: 0.9em; color: #2c3e50; }
  .history-entry:last-child { border-bottom: none; }
</style>
