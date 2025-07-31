#!/usr/bin/env python3
"""
Navigation Data Analyzer for Elysium Descent
Analyzes player movement patterns from nav.json
"""

import json
import math
import argparse
from typing import List, Dict, Tuple
from datetime import datetime

def load_nav_data(filename: str = "nav.json") -> Dict:
    """Load navigation data from JSON file."""
    try:
        with open(filename, 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"❌ Navigation file '{filename}' not found!")
        print("💡 Make sure to run the game and move around to generate navigation data.")
        return None
    except json.JSONDecodeError as e:
        print(f"❌ Error parsing JSON: {e}")
        return None

def calculate_distance(pos1: List[float], pos2: List[float]) -> float:
    """Calculate 3D distance between two positions."""
    return math.sqrt(sum((a - b) ** 2 for a, b in zip(pos1, pos2)))

def analyze_movement_patterns(nav_data: Dict) -> Dict:
    """Analyze player movement patterns."""
    positions = nav_data.get("positions", [])
    if len(positions) < 2:
        return {"error": "Not enough position data for analysis"}
    
    # Calculate movement statistics
    total_distance = 0.0
    max_distance_per_interval = 0.0
    speeds = []
    
    for i in range(1, len(positions)):
        prev_pos = positions[i-1]["position"]
        curr_pos = positions[i]["position"]
        time_diff = positions[i]["session_time"] - positions[i-1]["session_time"]
        
        distance = calculate_distance(prev_pos, curr_pos)
        total_distance += distance
        max_distance_per_interval = max(max_distance_per_interval, distance)
        
        if time_diff > 0:
            speed = distance / time_diff
            speeds.append(speed)
    
    avg_speed = sum(speeds) / len(speeds) if speeds else 0
    max_speed = max(speeds) if speeds else 0
    
    # Find most visited areas (clustering)
    clusters = find_position_clusters(positions)
    
    return {
        "total_distance_traveled": total_distance,
        "average_speed": avg_speed,
        "max_speed": max_speed,
        "max_distance_per_interval": max_distance_per_interval,
        "movement_clusters": clusters,
        "exploration_bounds": {
            "min_bounds": nav_data["statistics"]["min_bounds"],
            "max_bounds": nav_data["statistics"]["max_bounds"],
            "exploration_area": calculate_exploration_area(nav_data["statistics"])
        }
    }

def find_position_clusters(positions: List[Dict], cluster_radius: float = 5.0) -> List[Dict]:
    """Find clusters of positions where player spent time."""
    clusters = []
    
    for pos_data in positions:
        pos = pos_data["position"]
        
        # Find if this position belongs to an existing cluster
        added_to_cluster = False
        for cluster in clusters:
            if calculate_distance(pos, cluster["center"]) <= cluster_radius:
                cluster["positions"].append(pos)
                cluster["count"] += 1
                cluster["time_spent"] += 5.0  # 5 seconds per position
                # Update cluster center (simple average)
                cluster["center"] = [
                    sum(p[i] for p in cluster["positions"]) / len(cluster["positions"])
                    for i in range(3)
                ]
                added_to_cluster = True
                break
        
        if not added_to_cluster:
            clusters.append({
                "center": pos.copy(),
                "positions": [pos],
                "count": 1,
                "time_spent": 5.0
            })
    
    # Sort clusters by time spent (most visited first)
    clusters.sort(key=lambda x: x["time_spent"], reverse=True)
    return clusters[:10]  # Return top 10 clusters

def calculate_exploration_area(stats: Dict) -> float:
    """Calculate the total exploration area in square units."""
    min_bounds = stats["min_bounds"]
    max_bounds = stats["max_bounds"]
    
    width = max_bounds[0] - min_bounds[0]
    depth = max_bounds[2] - min_bounds[2]
    
    return width * depth

def print_analysis_report(nav_data: Dict, analysis: Dict):
    """Print a detailed analysis report."""
    print("\n" + "="*60)
    print("🗺️  NAVIGATION ANALYSIS REPORT")
    print("="*60)
    
    # Session info
    session_start = datetime.fromtimestamp(int(nav_data["session_start"]))
    stats = nav_data["statistics"]
    
    print(f"\n📊 SESSION INFORMATION:")
    print(f"   Start Time: {session_start.strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"   Duration: {stats['session_duration']:.1f} seconds ({stats['session_duration']/60:.1f} minutes)")
    print(f"   Total Navigation Points: {stats['total_points']}")
    print(f"   Average Position: [{stats['average_position'][0]:.2f}, {stats['average_position'][1]:.2f}, {stats['average_position'][2]:.2f}]")
    
    # Movement analysis
    if "error" not in analysis:
        print(f"\n🏃 MOVEMENT ANALYSIS:")
        print(f"   Total Distance Traveled: {analysis['total_distance_traveled']:.2f} units")
        print(f"   Average Speed: {analysis['average_speed']:.2f} units/second")
        print(f"   Maximum Speed: {analysis['max_speed']:.2f} units/second")
        print(f"   Max Distance in 5s: {analysis['max_distance_per_interval']:.2f} units")
        
        # Exploration area
        bounds = analysis['exploration_bounds']
        print(f"\n🌍 EXPLORATION AREA:")
        print(f"   Min Bounds: [{bounds['min_bounds'][0]:.2f}, {bounds['min_bounds'][1]:.2f}, {bounds['min_bounds'][2]:.2f}]")
        print(f"   Max Bounds: [{bounds['max_bounds'][0]:.2f}, {bounds['max_bounds'][1]:.2f}, {bounds['max_bounds'][2]:.2f}]")
        print(f"   Total Area Explored: {bounds['exploration_area']:.2f} square units")
        
        # Top visited areas
        clusters = analysis['movement_clusters']
        if clusters:
            print(f"\n🎯 TOP VISITED AREAS:")
            for i, cluster in enumerate(clusters[:5], 1):
                center = cluster['center']
                print(f"   {i}. Position [{center[0]:.2f}, {center[1]:.2f}, {center[2]:.2f}] - {cluster['time_spent']:.0f}s ({cluster['count']} visits)")
    else:
        print(f"\n❌ Analysis Error: {analysis['error']}")
    
    print("\n" + "="*60)

def export_heatmap_data(nav_data: Dict, filename: str = "heatmap_data.json"):
    """Export position data for heatmap visualization."""
    positions = nav_data.get("positions", [])
    
    heatmap_data = {
        "positions": [pos["position"] for pos in positions],
        "timestamps": [pos["session_time"] for pos in positions],
        "bounds": nav_data["statistics"]
    }
    
    with open(filename, 'w') as f:
        json.dump(heatmap_data, f, indent=2)
    
    print(f"📈 Heatmap data exported to {filename}")

def main():
    parser = argparse.ArgumentParser(description="Analyze Elysium Descent navigation data")
    parser.add_argument("--file", "-f", default="nav.json", help="Navigation JSON file (default: nav.json)")
    parser.add_argument("--export-heatmap", "-e", action="store_true", help="Export heatmap data")
    parser.add_argument("--watch", "-w", action="store_true", help="Watch for file changes and update analysis")
    
    args = parser.parse_args()
    
    nav_data = load_nav_data(args.file)
    if nav_data is None:
        return
    
    analysis = analyze_movement_patterns(nav_data)
    print_analysis_report(nav_data, analysis)
    
    if args.export_heatmap:
        export_heatmap_data(nav_data)
    
    if args.watch:
        print("\n👀 Watching for file changes... (Press Ctrl+C to stop)")
        import time
        try:
            last_modified = 0
            while True:
                try:
                    import os
                    current_modified = os.path.getmtime(args.file)
                    if current_modified > last_modified:
                        last_modified = current_modified
                        print(f"\n🔄 File updated at {datetime.now().strftime('%H:%M:%S')}")
                        nav_data = load_nav_data(args.file)
                        if nav_data:
                            analysis = analyze_movement_patterns(nav_data)
                            print_analysis_report(nav_data, analysis)
                except FileNotFoundError:
                    pass
                time.sleep(2)
        except KeyboardInterrupt:
            print("\n👋 Stopping file watch.")

if __name__ == "__main__":
    main() 