# Spherical Flocking Simulator

This project is a high-performance simulation of the Vicsek model for collective motion, extended to the surface of a
sphere. It provides a framework for studying the statistical physics of flocking behavior, including phase transitions,
cluster formation, and non-equilibrium dynamics.

The simulation is implemented in Rust for performance and correctness, while data analysis and visualization are
designed to be handled by external tools like Python.

## Features

- **Spherical Geometry:** Particles are constrained to the surface of a sphere, avoiding boundary effects and providing
  a topologically closed environment.
- **Robust Physics Engine:** Utilizes 3D Cartesian vectors for all internal calculations, avoiding singularities at the
  poles and simplifying vector mathematics.
- **Correct Dynamics:** Implements geodesic motion for particle updates and parallel transport for geometrically correct
  velocity comparisons.
- **Collision Avoidance:** Includes a short-range repulsive force to prevent particle overlap and ensure simulation
  stability.
- **Flexible Noise Model:** Supports configurable noise models (e.g., Uniform or Normal distribution) to study the
  order-disorder phase transition.
- **Ensemble-Based Workflow:** Designed for large-scale studies with tools to generate, simulate, and analyze large
  ensembles of initial conditions.
- **Modular and Testable:** A clean separation between the core simulation library and the command-line interface makes
  the code easy to maintain and test.

## Project Structure

The project is organized into a Rust library (`flocking_lib`) and a command-line binary (`flocking_cli`):

- `src/lib.rs`: The core library containing all the physics and data structures.
    - `vector.rs`: The fundamental `Vec3` type.
    - `particle.rs`: The `Particle` and `SphericalCoord` structs.
    - `simulation.rs`: The main simulation engine.
    - `ensemble.rs`: Logic for generating valid initial states.
    - `analysis.rs`: Functions for calculating order parameters and finding clusters.
    - `io.rs`: Handles all file saving and loading.
    - `cli.rs`: The `clap`-based definition of the command-line interface.
- `src/main.rs`: The main entry point that parses CLI arguments and calls the library.
- `plots/`: A directory intended for Python visualization scripts.
- `data/`: (Git-ignored) The default output directory for simulation data.

## Usage

The simulation is controlled via the `flocking_cli` command-line tool.

### Prerequisites

- Rust toolchain (install via [rustup.rs](https://rustup.rs/))
- Python 3 with `pandas` and `plotly` (for visualization)

### Build the Project

```bash
cargo build --release
```

The executable will be located at `target/release/flocking_cli`.

### Workflow Example

**1. Generate an Initial State**

First, generate a set of non-colliding particle coordinates. This creates a `initial_coords.json` file.

```bash
./target/release/flocking_cli generate --num-particles 512 --output initial_coords.json
```

(Optional: Manually inspect the generated JSON or write a quick Python script to plot it for a sanity check).

**2. Run a Simulation for Visualization**

Use the generated coordinates to run a simulation. This will produce a `viz_data.csv` file that can be read by a Python
script to create an interactive 3D plot.

```bash
./target/release/flocking_cli simulate --input initial_coords.json --output viz_data.csv --steps 2000 --noise 0.5
```

**3. Analyze Simulation Data**

(This mode would be used to analyze full snapshot files saved during a more intensive study).

```bash
./target/release/flocking_cli analyze --snapshot-dir ./data/snapshots/ --output analysis_results.csv
```

## Data Analysis and Visualization

The primary workflow is to generate data with the fast Rust binary and then analyze and plot it using Python. An example
visualization script `plots/visualize.py` can be used to read the `viz_data.csv` file and generate an interactive HTML
animation of the flock.

```bash
# Install Python dependencies
pip install pandas plotly

# Run the visualization script
python plots/visualize.py
```

This will create a `flock_visualization.html` file that can be opened in any web browser.