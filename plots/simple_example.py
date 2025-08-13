import json
import numpy as np
import plotly.graph_objects as go
import plotly.express as px
from plotly.subplots import make_subplots

def load_data(filepath):
    """Load bird data from JSON file"""
    with open(filepath, 'r') as f:
        data = json.load(f)
    return data

def check_sphere_constraint(positions, radius):
    """Check if all positions lie on the sphere surface"""
    distances = []
    for bird in positions:
        pos = bird['position']
        distance = np.sqrt(pos['x']**2 + pos['y']**2 + pos['z']**2)
        distances.append(distance)
    
    distances = np.array(distances)
    mean_dist = np.mean(distances)
    std_dist = np.std(distances)
    
    print(f"Expected radius: {radius}")
    print(f"Mean distance from origin: {mean_dist:.6f}")
    print(f"Standard deviation: {std_dist:.6f}")
    print(f"Min distance: {np.min(distances):.6f}")
    print(f"Max distance: {np.max(distances):.6f}")
    
    return distances

def create_sphere_surface(radius, resolution=50):
    """Create a sphere surface for visualization"""
    u = np.linspace(0, 2 * np.pi, resolution)
    v = np.linspace(0, np.pi, resolution)
    x = radius * np.outer(np.cos(u), np.sin(v))
    y = radius * np.outer(np.sin(u), np.sin(v))
    z = radius * np.outer(np.ones(np.size(u)), np.cos(v))
    return x, y, z

def visualize_birds_on_sphere(data):
    """Create 3D visualization of birds on sphere"""
    birds = data['birds']
    params = data['params']
    radius = params['radius']
    
    # Extract positions and velocities
    positions = np.array([[bird['position']['x'], bird['position']['y'], bird['position']['z']] for bird in birds])
    velocities = np.array([[bird['velocity']['x'], bird['velocity']['y'], bird['velocity']['z']] for bird in birds])
    
    # Check sphere constraint
    print("Checking sphere constraint:")
    distances = check_sphere_constraint(birds, radius)
    
    # Create sphere surface
    sphere_x, sphere_y, sphere_z = create_sphere_surface(radius)
    
    # Create figure
    fig = go.Figure()
    
    # Add sphere surface
    fig.add_trace(go.Surface(
        x=sphere_x, y=sphere_y, z=sphere_z,
        opacity=0.3,
        colorscale='Blues',
        showscale=False,
        name='Sphere Surface'
    ))
    
    # Add bird positions
    fig.add_trace(go.Scatter3d(
        x=positions[:, 0],
        y=positions[:, 1],
        z=positions[:, 2],
        mode='markers',
        marker=dict(
            size=8,
            color='red',
            symbol='circle'
        ),
        name='Bird Positions',
        text=[f'Bird {i+1}' for i in range(len(birds))]
    ))
    
    # Add velocity vectors
    for i, (pos, vel) in enumerate(zip(positions, velocities)):
        # Scale velocity vectors for better visibility
        scale = 0.2
        fig.add_trace(go.Scatter3d(
            x=[pos[0], pos[0] + vel[0] * scale],
            y=[pos[1], pos[1] + vel[1] * scale],
            z=[pos[2], pos[2] + vel[2] * scale],
            mode='lines',
            line=dict(color='green', width=3),
            showlegend=False,
            name=f'Velocity {i+1}'
        ))
    
    # Update layout
    fig.update_layout(
        title=f'Vicsek Model: {params["num_birds"]} Birds on Sphere (radius={radius})',
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

def create_distance_histogram(data):
    """Create histogram of distances from origin"""
    birds = data['birds']
    params = data['params']
    radius = params['radius']
    
    distances = check_sphere_constraint(birds, radius)
    
    fig = go.Figure()
    fig.add_trace(go.Histogram(
        x=distances,
        nbinsx=20,
        name='Distance Distribution'
    ))
    
    # Add expected radius line
    fig.add_vline(x=radius, line_dash="dash", line_color="red", 
                  annotation_text=f"Expected radius: {radius}")
    
    fig.update_layout(
        title='Distribution of Bird Distances from Origin',
        xaxis_title='Distance from Origin',
        yaxis_title='Count',
        width=600,
        height=400
    )
    
    return fig

def main():
    # Load data
    data_file = 'data/ensemble/sample_data.json'
    data = load_data(data_file)
    
    print(f"Loaded data with {data['params']['num_birds']} birds")
    print(f"Simulation parameters: {data['params']}")
    print()
    
    # Create 3D visualization
    fig_3d = visualize_birds_on_sphere(data)
    fig_3d.show()
    
    print()
    
    # Create distance histogram
    fig_hist = create_distance_histogram(data)
    fig_hist.show()

if __name__ == "__main__":
    main()