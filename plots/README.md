# Flocking Simulation Visualization

This directory contains Python scripts for visualizing bird flocking simulation data exported from the Rust simulation engine.

## Overview

The Rust simulation generates binary data files containing bird trajectories on a sphere. To analyze and visualize this data in Python, you need to:

1. **Export data from Rust to JSON format**
2. **Import and visualize the JSON data in Python**

## Rust Export Function

A new function `export_to_json()` has been added to `src/io/simulation.rs`:

```rust
use flocking_lib::io::simulation::export_to_json;
use std::path::Path;

// Export simulation data to JSON
export_to_json("my_simulation", &1, Path::new("./data/simulation/exported_simulation.json"))?;
```

This creates a JSON file with the following structure:
```json
{
  "metadata": {
    "simulation_id": 1,
    "tag": "my_simulation", 
    "ensemble_id": 42,
    "total_steps": 1000,
    "duration_seconds": 1.234,
    "created_at": 1628765432
  },
  "parameters": {
    "num_birds": 100,
    "radius": 1.0,
    "speed": 1.0,
    "dt": 0.01,
    "interaction_radius": 0.5,
    "eta": 0.1,
    "iterations": 1000
  },
  "frames": [
    {
      "step": 0,
      "timestamp": 0.0,
      "birds": [
        {
          "position": {"x": 1.0, "y": 0.0, "z": 0.0},
          "velocity": {"x": 0.0, "y": 1.0, "z": 0.0}
        }
      ]
    }
  ]
}
```

## Python Scripts

### Dependencies

Install required Python packages:
```bash
pip install plotly pandas numpy
```

### 1. Simple Example (`simple_example.py`)

Basic visualization showing birds on a sphere for one or two frames.

**Usage:**
```bash
python simple_example.py [json_file]
```

**Features:**
- Shows bird positions as red dots on a wireframe sphere
- Displays velocity vectors (for small flocks)
- Shows first and last frames for comparison

### 2. Full Visualization (`visualize_birds_sphere.py`)

Comprehensive visualization with animation and analysis plots.

**Usage:**
```bash
# Full visualization (animation + analysis)
python visualize_birds_sphere.py [json_file]

# Only animation
python visualize_birds_sphere.py --animation-only [json_file]

# Only analysis plots
python visualize_birds_sphere.py --analysis-only [json_file]
```

**Features:**
- **Animated 3D visualization** of bird movement on sphere
- **Trajectory analysis** with multiple plots:
  - 3D trajectory paths for sample birds
  - Velocity magnitude over time
  - Order parameter (flocking alignment measure)
  - Average distance from sphere center
- Interactive controls (play/pause animation)
- Comprehensive simulation information display

## Example Workflow

### 1. In Rust (export data):

```rust
// In your main.rs or simulation code
use flocking_lib::io::simulation::export_to_json;
use std::path::Path;

// After running a simulation with tag "test" and id 1
let output_path = Path::new("./data/simulation/test_simulation.json");
export_to_json("test", &1, &output_path)?;
```

### 2. In Python (visualize data):

```bash
# Navigate to plots directory
cd plots

# Simple visualization
python simple_example.py ../data/simulation/test_simulation.json

# Full visualization with animation
python visualize_birds_sphere.py ../data/simulation/test_simulation.json
```

## Data Analysis

The Python scripts provide several analysis tools:

1. **Order Parameter**: Measures how aligned the birds are (0 = random, 1 = perfectly aligned)
2. **Velocity Analysis**: Shows speed distribution and temporal evolution
3. **Spatial Distribution**: Tracks how birds move on the sphere surface
4. **Trajectory Paths**: Individual bird movement patterns

## Customization

You can modify the Python scripts to:

- Change visualization colors and styles
- Add new analysis metrics
- Export plots to files (PNG, PDF, etc.)
- Create custom animation sequences
- Filter data by time ranges or bird subsets

## Performance Notes

- For large simulations (many birds/frames), the animation might be slow
- The full visualization samples every 5th frame by default for performance
- Consider using `--analysis-only` for quick data overview
- JSON files can be large for long simulations

## Troubleshooting

**File not found error:**
- Make sure you've exported the data from Rust first
- Check the file path is correct
- Ensure the JSON file is valid

**Memory issues with large datasets:**
- Reduce the number of frames in the export
- Use analysis-only mode
- Sample fewer birds for trajectory plots

**Slow performance:**
- Reduce animation frame rate
- Decrease sphere resolution
- Use simple_example.py for quick viewing
