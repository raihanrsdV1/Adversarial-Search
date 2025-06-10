import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import numpy as np
from matplotlib.colors import ListedColormap

def create_tournament_heatmaps():
    """
    Generates and saves heatmaps for the Heuristic vs. Heuristic tournament results
    for both Red and Blue player perspectives.
    """
    # This matrix is reconstructed from the final tournament standings and head-to-head results.
    # 1 indicates the Row Player (Red) wins, 0 indicates the Column Player (Blue) wins.
    # P=0, O=1, T=2, C=3, R=4
    matrix = np.array([
        [np.nan, 1,      1,      1,      1],      # P vs (O, T, C, R)
        [0,      np.nan, 1,      1,      1],      # O vs (P, T, C, R)
        [0,      0,      np.nan, 0,      1],      # T vs (P, O, C, R)
        [0,      1,      1,      np.nan, 1],      # C vs (P, O, T, R)
        [1,      1,      1,      0,      np.nan]       # R vs (P, O, T, C)
    ])

    heuristics_labels = [
        'Peripheral\nControl', 
        'Orb\nDifference', 
        'Territory\nControl', 
        'Cascade\nPotential', 
        'Chain Reaction+\nConversion Pot.'
    ]
    
    df_red = pd.DataFrame(matrix, index=heuristics_labels, columns=heuristics_labels)

    # --- Create Heatmap for Red Player ---
    plt.figure(figsize=(10, 8))
    cmap_red = ListedColormap(["#ff9999", "#90ee90"])
    
    ax1 = sns.heatmap(df_red, cmap=cmap_red, cbar=False, linewidths=2.5, linecolor='white', annot=False)
    
    plt.title('Heuristic vs. Heuristic: Red Player Win/Loss Outcome', fontsize=16, pad=20)
    plt.xlabel('Blue Player Heuristic', fontsize=12, labelpad=15)
    plt.ylabel('Red Player Heuristic', fontsize=12, labelpad=15)
    plt.xticks(rotation=0)
    plt.yticks(rotation=0)
    
    for i in range(len(heuristics_labels)):
        for j in range(len(heuristics_labels)):
            if i == j: continue
            text = "Win" if df_red.iloc[i, j] == 1 else "Loss"
            ax1.text(j + 0.5, i + 0.5, text, ha='center', va='center', color='black', fontsize=12)

    plt.tight_layout()
    plt.savefig('tournament_grid_red_wins.png', dpi=300)
    plt.show()

    # --- Create Heatmap for Blue Player ---
    df_blue = df_red.transpose().map(lambda x: 1 - x if pd.notna(x) else np.nan)
    
    plt.figure(figsize=(10, 8))
    cmap_blue = ListedColormap(["#ff9999", "#89cff0"])

    ax2 = sns.heatmap(df_blue, cmap=cmap_blue, cbar=False, linewidths=2.5, linecolor='white', annot=False)

    plt.title('Heuristic vs. Heuristic: Blue Player Win/Loss Outcome', fontsize=16, pad=20)
    plt.xlabel('Red Player Heuristic', fontsize=12, labelpad=15)
    plt.ylabel('Blue Player Heuristic', fontsize=12, labelpad=15)
    plt.xticks(rotation=0)
    plt.yticks(rotation=0)

    for i in range(len(heuristics_labels)):
        for j in range(len(heuristics_labels)):
            if i == j: continue
            text = "Win" if df_blue.iloc[i, j] == 1 else "Loss"
            ax2.text(j + 0.5, i + 0.5, text, ha='center', va='center', color='black', fontsize=12)

    plt.tight_layout()
    plt.savefig('tournament_grid_blue_wins.png', dpi=300)
    plt.show()


def create_win_rate_chart():
    """
    Generates and saves a bar chart for the overall win rate of each heuristic.
    """
    data = {
        'Heuristic': [
            'Peripheral Control', 
            'Cascade Potential',
            'Orb Difference', 
            'Chain Reaction + Conversion Potential', 
            'Territory Control'
        ],
        'Wins': [7, 6, 4, 3, 2],
        'Losses': [1, 2, 4, 5, 6]
    }
    df = pd.DataFrame(data)
    df['Win Rate (%)'] = (df['Wins'] / (df['Wins'] + df['Losses'])) * 100

    plt.figure(figsize=(10, 6))
    sns.set_style("whitegrid")
    
    barplot = sns.barplot(x='Heuristic', y='Win Rate (%)', data=df.sort_values('Win Rate (%)', ascending=False), palette='viridis')
    
    plt.title('Overall Win Rate of Each Heuristic Across Tournament', fontsize=16)
    plt.xlabel('Agent', fontsize=12)
    plt.ylabel('Overall Win Rate (%)', fontsize=12)
    plt.xticks(rotation=45, ha='right')
    plt.ylim(0, 100)

    for p in barplot.patches:
        barplot.annotate(format(p.get_height(), '.1f') + '%', 
                       (p.get_x() + p.get_width() / 2., p.get_height()), 
                       ha = 'center', va = 'center', 
                       xytext = (0, -12), 
                       textcoords = 'offset points',
                       color='white',
                       weight='bold')
    
    plt.tight_layout()
    plt.savefig('win_rate_chart.png', dpi=300)
    plt.show()

def create_runtime_charts():
    """
    Generates and saves the runtime comparison charts.
    """
    # Data when Heuristic is Red player
    red_player_data = {
        'Heuristic (Red Player)': ['Peripheral Control', 'Orb Difference', 'Territory Control', 'Cascade Potential', 'Chain Reaction+'],
        'Average Game Time (s)': [74, 126, 106, 104, 75]
    }
    df_red = pd.DataFrame(red_player_data)

    # Data when Heuristic is Blue player
    blue_player_data = {
        'Heuristic (Blue Player)': ['Peripheral Control', 'Orb Difference', 'Territory Control', 'Cascade Potential', 'Chain Reaction+'],
        'Average Game Time (s)': [51, 140, 50, 164, 170]
    }
    df_blue = pd.DataFrame(blue_player_data)

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 6))
    sns.set_style("whitegrid")

    # Plot 1: Heuristic as Red
    sns.barplot(ax=ax1, x='Heuristic (Red Player)', y='Average Game Time (s)', data=df_red, palette='plasma')
    ax1.set_title('Runtime: All Heuristics (Red) vs. Random (Blue)')
    ax1.set_xlabel('Heuristic (Red Player)')
    ax1.set_ylabel('Average Game Time (s)')
    ax1.tick_params(axis='x', rotation=45)

    # Plot 2: Heuristic as Blue
    sns.barplot(ax=ax2, x='Heuristic (Blue Player)', y='Average Game Time (s)', data=df_blue, palette='cividis')
    ax2.set_title('Runtime: Random (Red) vs. All Heuristics (Blue)')
    ax2.set_xlabel('Heuristic (Blue Player)')
    ax2.set_ylabel('Average Game Time (s)')
    ax2.tick_params(axis='x', rotation=45)

    plt.tight_layout()
    plt.savefig('runtime_charts.png', dpi=300)
    plt.show()

if __name__ == '__main__':
    create_tournament_heatmaps()
    create_win_rate_chart()
    create_runtime_charts()
