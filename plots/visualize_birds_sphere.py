#!/usr/bin/env python3
"""
Flocking Simulation Visualization on Sphere

This script demonstrates how to import and visualize bird movement data from the
Rust flocking simulation using Plotly for 3D sphere visualization.

Dependencies:
    pip install plotly pandas numpy

Usage:
    python visualize_birds_sphere.py [simulation_json_file]
"""

import json
import numpy as np
import plotly.graph_objects as go
import plotly.express as px
from plotly.subplots import make_subplots
import argparse
import sys
from pathlib import Path


def load_simulation_data(json_file):
    """Load simulation data from JSON file exported by Rust simulation."""
    try:
        with open(json_file, 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"Error: File '{json_file}' not found.")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON format - {e}")
        sys.exit(1)


def create_sphere_surface(radius, resolution=50):
    """Create sphere surface for visualization background."""
    # Create sphere coordinates
    phi = np.linspace(0, 2 * np.pi, resolution)
    theta = np.linspace(0, np.pi, resolution)
    phi, theta = np.meshgrid(phi, theta)
    
    x = radius * np.sin(theta) * np.cos(phi)
    y = radius * np.sin(theta) * np.sin(phi)
    z = radius * np.cos(theta)
    
    return x, y, z


def extract_trajectories(data):
    """Extract bird trajectories from simulation data."""
    num_birds = data['parameters']['num_birds']
    num_frames = len(data['frames'])
    
    # Initialize trajectory arrays
    trajectories = {
        'x': np.zeros((num_birds, num_frames)),
        'y': np.zeros((num_birds, num_frames)),
        'z': np.zeros((num_birds, num_frames)),
        'vx': np.zeros((num_birds, num_frames)),
        'vy': np.zeros((num_birds, num_frames)),
        'vz': np.zeros((num_birds, num_frames)),
        'times': [],
        'steps': []
    }
    
    # Extract data frame by frame
    for frame_idx, frame in enumerate(data['frames']):
        trajectories['times'].append(frame['timestamp'])
        trajectories['steps'].append(frame['step'])
        
        for bird_idx, bird in enumerate(frame['birds']):
            trajectories['x'][bird_idx, frame_idx] = bird['position']['x']
            trajectories['y'][bird_idx, frame_idx] = bird['position']['y']
            trajectories['z'][bird_idx, frame_idx] = bird['position']['z']
            trajectories['vx'][bird_idx, frame_idx] = bird['velocity']['x']
            trajectories['vy'][bird_idx, frame_idx] = bird['velocity']['y']
            trajectories['vz'][bird_idx, frame_idx] = bird['velocity']['z']
    
    return trajectories


def create_animated_visualization(data, trajectories):
    """Create an animated 3D visualization of birds on sphere."""
    radius = data['parameters']['radius']
    
    # Create sphere surface
    sphere_x, sphere_y, sphere_z = create_sphere_surface(radius)
    
    # Create figure
    fig = go.Figure()
    
    # Add transparent sphere surface
    fig.add_trace(go.Surface(
        x=sphere_x, y=sphere_y, z=sphere_z,
        opacity=0.3,
        colorscale='Blues',
        showscale=False,
        name='Sphere Surface'
    ))
    
    # Add initial bird positions
    frame_0 = data['frames'][0]
    x_pos = [bird['position']['x'] for bird in frame_0['birds']]
    y_pos = [bird['position']['y'] for bird in frame_0['birds']]
    z_pos = [bird['position']['z'] for bird in frame_0['birds']]
    
    # Add birds as scatter points
    fig.add_trace(go.Scatter3d(
        x=x_pos, y=y_pos, z=z_pos,
        mode='markers',
        marker=dict(
            size=8,
            color='red',
            opacity=0.8
        ),
        name='Birds'
    ))
    
    # Create animation frames
    frames = []
    for frame_idx, frame in enumerate(data['frames'][::5]):  # Sample every 5th frame for performance
        x_pos = [bird['position']['x'] for bird in frame['birds']]
        y_pos = [bird['position']['y'] for bird in frame['birds']]
        z_pos = [bird['position']['z'] for bird in frame['birds']]
        
        frame_data = go.Frame(
            data=[
                go.Surface(x=sphere_x, y=sphere_y, z=sphere_z),
                go.Scatter3d(x=x_pos, y=y_pos, z=z_pos)
            ],
            name=f"frame_{frame_idx}"
        )
        frames.append(frame_data)
    
    fig.frames = frames
    
    # Update layout
    fig.update_layout(
        title=f"Flocking Simulation: {data['metadata']['tag']} (ID: {data['metadata']['simulation_id']})",
        scene=dict(
            xaxis_title='X',
            yaxis_title='Y',
            zaxis_title='Z',
            aspectmode='cube',
            camera=dict(
                eye=dict(x=1.5, y=1.5, z=1.5)
            )
        ),
        updatemenus=[{
            'type': 'buttons',
            'showactive': False,
            'buttons': [
                {
                    'label': 'Play',
                    'method': 'animate',
                    'args': [None, {
                        'frame': {'duration': 100, 'redraw': True},
                        'fromcurrent': True,
                        'transition': {'duration': 50}
                    }]
                },
                {
                    'label': 'Pause',
                    'method': 'animate',
                    'args': [[None], {
                        'frame': {'duration': 0, 'redraw': False},
                        'mode': 'immediate',
                        'transition': {'duration': 0}
                    }]
                }
            ]
        }]
    )
    
    return fig


def create_trajectory_analysis(data, trajectories):
    """Create analysis plots for bird trajectories."""
    
    # Create subplots
    fig = make_subplots(
        rows=2, cols=2,
        subplot_titles=('Position Distribution', 'Velocity Magnitude Over Time', 
                       'Order Parameter', 'Average Distance from Center'),
        specs=[[{'type': 'scatter3d'}, {'type': 'scatter'}],
               [{'type': 'scatter'}, {'type': 'scatter'}]]
    )
    
    # 1. 3D trajectory plot (sample of birds)
    num_birds_to_plot = min(10, data['parameters']['num_birds'])
    colors = px.colors.qualitative.Set1[:num_birds_to_plot]
    
    for i in range(num_birds_to_plot):
        fig.add_trace(
            go.Scatter3d(
                x=trajectories['x'][i, :],
                y=trajectories['y'][i, :],
                z=trajectories['z'][i, :],
                mode='lines',
                line=dict(color=colors[i % len(colors)], width=3),
                name=f'Bird {i+1}'
            ),
            row=1, col=1
        )
    
    # 2. Velocity magnitude over time
    vel_magnitudes = np.sqrt(trajectories['vx']**2 + trajectories['vy']**2 + trajectories['vz']**2)
    mean_vel = np.mean(vel_magnitudes, axis=0)
    std_vel = np.std(vel_magnitudes, axis=0)
    
    fig.add_trace(
        go.Scatter(
            x=trajectories['times'],
            y=mean_vel,
            mode='lines',
            name='Mean Velocity',
            line=dict(color='blue')
        ),
        row=1, col=2
    )
    
    fig.add_trace(
        go.Scatter(
            x=trajectories['times'] + trajectories['times'][::-1],
            y=np.concatenate([mean_vel + std_vel, (mean_vel - std_vel)[::-1]]),
            fill='toself',
            fillcolor='rgba(0,100,80,0.2)',
            line=dict(color='rgba(255,255,255,0)'),
            name='±1σ',
            showlegend=False
        ),
        row=1, col=2
    )
    
    # 3. Order parameter (alignment measure)
    order_parameter = []
    for frame_idx in range(len(trajectories['times'])):
        # Calculate normalized velocities
        vx = trajectories['vx'][:, frame_idx]
        vy = trajectories['vy'][:, frame_idx]
        vz = trajectories['vz'][:, frame_idx]
        
        # Normalize each velocity vector
        speeds = np.sqrt(vx**2 + vy**2 + vz**2)
        speeds[speeds == 0] = 1  # Avoid division by zero
        
        vx_norm = vx / speeds
        vy_norm = vy / speeds
        vz_norm = vz / speeds
        
        # Calculate order parameter (magnitude of average normalized velocity)
        avg_vx = np.mean(vx_norm)
        avg_vy = np.mean(vy_norm)
        avg_vz = np.mean(vz_norm)
        
        order = np.sqrt(avg_vx**2 + avg_vy**2 + avg_vz**2)
        order_parameter.append(order)
    
    fig.add_trace(
        go.Scatter(
            x=trajectories['times'],
            y=order_parameter,
            mode='lines',
            name='Order Parameter',
            line=dict(color='green')
        ),
        row=2, col=1
    )
    
    # 4. Average distance from center
    distances = np.sqrt(trajectories['x']**2 + trajectories['y']**2 + trajectories['z']**2)
    mean_distance = np.mean(distances, axis=0)
    
    fig.add_trace(
        go.Scatter(
            x=trajectories['times'],
            y=mean_distance,
            mode='lines',
            name='Avg Distance from Center',
            line=dict(color='purple')
        ),
        row=2, col=2
    )
    
    # Add reference line for sphere radius
    fig.add_trace(
        go.Scatter(
            x=trajectories['times'],
            y=[data['parameters']['radius']] * len(trajectories['times']),
            mode='lines',
            name='Sphere Radius',
            line=dict(color='red', dash='dash')
        ),
        row=2, col=2
    )
    
    # Update layout
    fig.update_layout(
        title=f"Trajectory Analysis: {data['metadata']['tag']} (ID: {data['metadata']['simulation_id']})",
        height=800
    )
    
    return fig


def print_simulation_info(data):
    """Print basic information about the simulation."""
    metadata = data['metadata']
    params = data['parameters']
    
    print("=" * 60)
    print("SIMULATION INFORMATION")
    print("=" * 60)
    print(f"Tag: {metadata['tag']}")
    print(f"Simulation ID: {metadata['simulation_id']}")
    print(f"Ensemble ID: {metadata['ensemble_id']}")
    print(f"Total Steps: {metadata['total_steps']}")
    print(f"Duration: {metadata['duration_seconds']:.3f} seconds")
    print(f"Created At: {metadata['created_at']}")
    print()
    print("PARAMETERS:")
    print(f"  Number of Birds: {params['num_birds']}")
    print(f"  Sphere Radius: {params['radius']}")
    print(f"  Speed: {params['speed']}")
    print(f"  Time Step (dt): {params['dt']}")
    print(f"  Interaction Radius: {params['interaction_radius']}")
    print(f"  Noise (eta): {params['eta']}")
    print(f"  Iterations: {params['iterations']}")
    print()
    print(f"FRAMES: {len(data['frames'])} snapshots")
    print("=" * 60)


def main():
    """Main function to run the visualization."""
    parser = argparse.ArgumentParser(description='Visualize flocking simulation data')
    parser.add_argument('json_file', nargs='?', 
                       default='../data/simulation/exported_simulation.json',
                       help='Path to JSON file exported from Rust simulation')
    parser.add_argument('--analysis-only', action='store_true',
                       help='Show only analysis plots (no animation)')
    parser.add_argument('--animation-only', action='store_true',
                       help='Show only animated visualization')
    
    args = parser.parse_args()
    
    # Load data
    print(f"Loading simulation data from: {args.json_file}")
    data = load_simulation_data(args.json_file)
    
    # Print simulation information
    print_simulation_info(data)
    
    # Extract trajectories
    print("Extracting bird trajectories...")
    trajectories = extract_trajectories(data)
    
    # Create visualizations
    if not args.analysis_only:
        print("Creating animated 3D visualization...")
        anim_fig = create_animated_visualization(data, trajectories)
        anim_fig.show()
    
    if not args.animation_only:
        print("Creating trajectory analysis plots...")
        analysis_fig = create_trajectory_analysis(data, trajectories)
        analysis_fig.show()
    
    print("Visualization complete!")


if __name__ == "__main__":
    main()
