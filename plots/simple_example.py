#!/usr/bin/env python3
"""
Simple example showing how to load and visualize flocking simulation data.

This is a minimal example demonstrating the basic workflow:
1. Load JSON data exported from Rust simulation
2. Extract bird positions for a specific frame
3. Create a simple 3D scatter plot on sphere

Dependencies:
    pip install plotly numpy

Usage:
    python simple_example.py [json_file]
"""

import json
import numpy as np
import plotly.graph_objects as go
import sys


def load_data(json_file):
    """Load simulation data from JSON file."""
    with open(json_file, 'r') as f:
        return json.load(f)


def create_sphere_wireframe(radius, resolution=20):
    """Create wireframe sphere for reference."""
    phi = np.linspace(0, 2 * np.pi, resolution)
    theta = np.linspace(0, np.pi, resolution)
    
    # Longitude lines
    traces = []
    for p in phi[::4]:  # Every 4th longitude line
        theta_line = np.linspace(0, np.pi, resolution)
        x = radius * np.sin(theta_line) * np.cos(p)
        y = radius * np.sin(theta_line) * np.sin(p)
        z = radius * np.cos(theta_line)
        
        traces.append(go.Scatter3d(
            x=x, y=y, z=z,
            mode='lines',
            line=dict(color='lightgray', width=2),
            showlegend=False
        ))
    
    # Latitude lines
    for t in theta[::4]:  # Every 4th latitude line
        if t == 0 or t == np.pi:  # Skip poles
            continue
        phi_line = np.linspace(0, 2 * np.pi, resolution)
        x = radius * np.sin(t) * np.cos(phi_line)
        y = radius * np.sin(t) * np.sin(phi_line)
        z = radius * np.cos(t) * np.ones_like(phi_line)
        
        traces.append(go.Scatter3d(
            x=x, y=y, z=z,
            mode='lines',
            line=dict(color='lightgray', width=2),
            showlegend=False
        ))
    
    return traces


def plot_frame(data, frame_index=0):
    """Plot birds on sphere for a specific frame."""
    
    # Get simulation parameters
    radius = data['parameters']['radius']
    frame = data['frames'][frame_index]
    
    # Extract bird positions
    x_pos = [bird['position']['x'] for bird in frame['birds']]
    y_pos = [bird['position']['y'] for bird in frame['birds']]
    z_pos = [bird['position']['z'] for bird in frame['birds']]
    
    # Extract velocities for arrow visualization
    vx = [bird['velocity']['x'] for bird in frame['birds']]
    vy = [bird['velocity']['y'] for bird in frame['birds']]
    vz = [bird['velocity']['z'] for bird in frame['birds']]
    
    # Create figure
    fig = go.Figure()
    
    # Add sphere wireframe
    sphere_traces = create_sphere_wireframe(radius)
    for trace in sphere_traces:
        fig.add_trace(trace)
    
    # Add birds as points
    fig.add_trace(go.Scatter3d(
        x=x_pos, y=y_pos, z=z_pos,
        mode='markers',
        marker=dict(
            size=8,
            color='red',
            opacity=0.8
        ),
        name=f'Birds (Frame {frame_index})',
        text=[f'Bird {i}' for i in range(len(x_pos))],
        hovertemplate='<b>%{text}</b><br>' +
                     'Position: (%{x:.3f}, %{y:.3f}, %{z:.3f})<br>' +
                     '<extra></extra>'
    ))
    
    # Add velocity vectors (optional - can be overwhelming with many birds)
    if len(x_pos) <= 50:  # Only show arrows for small flocks
        # Scale velocity vectors for visibility
        scale = 0.1
        fig.add_trace(go.Scatter3d(
            x=np.repeat(x_pos, 2) + np.tile([0, scale], len(x_pos)) * np.repeat(vx, 2),
            y=np.repeat(y_pos, 2) + np.tile([0, scale], len(y_pos)) * np.repeat(vy, 2),
            z=np.repeat(z_pos, 2) + np.tile([0, scale], len(z_pos)) * np.repeat(vz, 2),
            mode='lines',
            line=dict(color='blue', width=3),
            name='Velocity Vectors',
            showlegend=False
        ))
    
    # Update layout
    fig.update_layout(
        title=f"Flocking Simulation - Frame {frame_index}<br>" +
              f"Time: {frame['timestamp']:.3f}s, Step: {frame['step']}<br>" +
              f"Birds: {len(x_pos)}, Radius: {radius}",
        scene=dict(
            xaxis_title='X',
            yaxis_title='Y',
            zaxis_title='Z',
            aspectmode='cube',
            camera=dict(
                eye=dict(x=1.5, y=1.5, z=1.5)
            )
        ),
        width=800,
        height=600
    )
    
    return fig


def main():
    """Main function."""
    
    # Get JSON file path
    if len(sys.argv) > 1:
        json_file = sys.argv[1]
    else:
        json_file = '../data/simulation/exported_simulation.json'
    
    print(f"Loading data from: {json_file}")
    
    try:
        # Load simulation data
        data = load_data(json_file)
        
        # Print basic info
        print(f"Simulation: {data['metadata']['tag']} (ID: {data['metadata']['simulation_id']})")
        print(f"Birds: {data['parameters']['num_birds']}")
        print(f"Frames: {len(data['frames'])}")
        print(f"Sphere radius: {data['parameters']['radius']}")
        
        # Plot first frame
        fig = plot_frame(data, frame_index=0)
        fig.show()
        
        # If there are multiple frames, also show the last frame
        if len(data['frames']) > 1:
            fig_last = plot_frame(data, frame_index=-1)
            fig_last.show()
        
        print("Visualization complete!")
        
    except FileNotFoundError:
        print(f"Error: File '{json_file}' not found.")
        print("Make sure to export simulation data from Rust first using:")
        print("  io::simulation::export_to_json(&tag, &id, &output_path)")
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
