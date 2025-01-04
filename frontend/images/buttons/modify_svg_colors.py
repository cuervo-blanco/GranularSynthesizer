import os

def modify_svg_color(input_file, output_file, fill_color):
    """
    Modify the fill color in an SVG file and save as a new file.
    Args:
        input_file (str): Path to the input SVG file.
        output_file (str): Path to the output SVG file.
        fill_color (str): New fill color to apply (e.g., '#ff0000').
    """
    with open(input_file, 'r') as file:
        content = file.read()

    # Replace the 'fill="currentColor"' or similar fill attributes
    modified_content = content.replace('fill="currentColor"', f'fill="{fill_color}"')

    # Save the modified SVG
    with open(output_file, 'w') as file:
        file.write(modified_content)


# Define your color mappings
color_mappings = {
    'hover': '#ffffff',      # White for hover
    'pressed': '#aaaaaa',    # Light gray for pressed
    'inactive': '#333333',   # Dark gray for inactive
    'record': '#770000'      # Dark red for record
}

# Path to your SVG files
svg_files = [
    ("play-button.svg", "play"),
    ("stop-button.svg", "stop"),
    ("record-button.svg", "record"),
    ("pause-button.svg", "pause"),
    ("load-button.svg", "load"),
]

# Generate modified SVG files
for input_svg, base_name in svg_files:
    for state, color in color_mappings.items():
        output_svg = f"{base_name}_{state}.svg"
        modify_svg_color(input_svg, output_svg, color)

print("SVG files with modified colors have been created.")

