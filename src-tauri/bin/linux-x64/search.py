import sys
import json
from snaphoundpy import SnapHound

# Ensure there are enough arguments
if len(sys.argv) < 3:
    print("Usage: script.py <priority_paths_json> <paths_json>")
    sys.exit(1)

# Parse JSON arguments from command line
priority_paths = json.loads(sys.argv[1])  # First argument after script name
paths = json.loads(sys.argv[2])  # Second argument after script name

# Initialize SnapHound
snaphoundpy = SnapHound(paths=paths, priority_paths=priority_paths)

# Perform search
snaphoundpy.search_with_text("The END")
